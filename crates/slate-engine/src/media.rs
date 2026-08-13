//! Local ffmpeg helpers — video frame grab + simple cut assemble.

use std::path::{Path, PathBuf};
use std::process::Command;

/// True if `path` looks like a video container.
pub fn is_video_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            matches!(
                e.to_ascii_lowercase().as_str(),
                "mp4" | "webm" | "mkv" | "mov" | "avi" | "gif"
            )
        })
        .unwrap_or(false)
}

/// True if `path` looks like a still image.
pub fn is_image_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            matches!(
                e.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "webp" | "bmp"
            )
        })
        .unwrap_or(false)
}

fn ffmpeg_bin() -> &'static str {
    "ffmpeg"
}

/// Grab the first frame of a video into `dest` (png). Returns `dest`.
pub fn extract_first_frame(video: &Path, dest: &Path) -> Result<PathBuf, String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    }
    let status = Command::new(ffmpeg_bin())
        .args(["-y", "-hide_banner", "-loglevel", "error", "-i"])
        .arg(video)
        .args(["-frames:v", "1", "-q:v", "2"])
        .arg(dest)
        .status()
        .map_err(|e| format!("ffmpeg not on PATH: {e}"))?;
    if !status.success() {
        return Err(format!(
            "ffmpeg extract failed ({status}) for {}",
            video.display()
        ));
    }
    if !dest.is_file() {
        return Err(format!("ffmpeg wrote no frame at {}", dest.display()));
    }
    Ok(dest.to_path_buf())
}

/// If `path` is video, extract a sibling `_frame.png` for the VL judge; else return as-is.
pub fn media_for_judge(path: &Path) -> Result<PathBuf, String> {
    if !is_video_path(path) {
        return Ok(path.to_path_buf());
    }
    let dest = path.with_file_name(format!(
        "{}_judge.png",
        path.file_stem().and_then(|s| s.to_str()).unwrap_or("take")
    ));
    if dest.is_file() {
        return Ok(dest);
    }
    extract_first_frame(path, &dest)
}

fn still_clip(src: &Path, dest: &Path, seconds: f64) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    }
    let dur = format!("{seconds:.2}");
    let status = Command::new(ffmpeg_bin())
        .args([
            "-y",
            "-hide_banner",
            "-loglevel",
            "error",
            "-loop",
            "1",
            "-i",
        ])
        .arg(src)
        .args([
            "-t", &dur, "-r", "24", "-pix_fmt", "yuv420p", "-c:v", "libx264",
        ])
        .arg(dest)
        .status()
        .map_err(|e| format!("ffmpeg not on PATH: {e}"))?;
    if !status.success() {
        return Err(format!("ffmpeg still-clip failed ({status})"));
    }
    Ok(())
}

/// Concat takes (video as-is; stills become 2s holds) into `dest` mp4.
pub fn assemble_cut(paths: &[PathBuf], dest: &Path) -> Result<PathBuf, String> {
    if paths.is_empty() {
        return Err("no takes to assemble".into());
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    }
    let work = dest.parent().unwrap_or(Path::new(".")).join("_assemble");
    std::fs::create_dir_all(&work).map_err(|e| format!("mkdir: {e}"))?;

    let mut clips = Vec::new();
    for (i, p) in paths.iter().enumerate() {
        if !p.is_file() {
            continue;
        }
        if is_video_path(p) {
            clips.push(p.clone());
            continue;
        }
        if is_image_path(p) {
            let clip = work.join(format!("still_{i:02}.mp4"));
            still_clip(p, &clip, 2.0)?;
            clips.push(clip);
        }
    }
    if clips.is_empty() {
        return Err("no usable take files (png/mp4)".into());
    }
    if clips.len() == 1 {
        std::fs::copy(&clips[0], dest).map_err(|e| format!("copy cut: {e}"))?;
        return Ok(dest.to_path_buf());
    }

    let list = work.join("concat.txt");
    let mut body = String::new();
    for c in &clips {
        let abs = c
            .canonicalize()
            .unwrap_or_else(|_| c.clone())
            .display()
            .to_string()
            .replace('\\', "/")
            .replace('\'', r"'\''");
        body.push_str(&format!("file '{abs}'\n"));
    }
    std::fs::write(&list, body).map_err(|e| format!("write concat list: {e}"))?;

    let status = Command::new(ffmpeg_bin())
        .args([
            "-y",
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
        ])
        .arg(&list)
        .args(["-c", "copy"])
        .arg(dest)
        .status()
        .map_err(|e| format!("ffmpeg not on PATH: {e}"))?;
    if !status.success() {
        // Re-encode if copy-concat fails (mixed codecs).
        let status = Command::new(ffmpeg_bin())
            .args([
                "-y",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "concat",
                "-safe",
                "0",
                "-i",
            ])
            .arg(&list)
            .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac"])
            .arg(dest)
            .status()
            .map_err(|e| format!("ffmpeg not on PATH: {e}"))?;
        if !status.success() {
            return Err(format!("ffmpeg concat failed ({status})"));
        }
    }
    Ok(dest.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_extensions() {
        assert!(is_video_path(Path::new("a.MP4")));
        assert!(is_image_path(Path::new("b.png")));
        assert!(!is_video_path(Path::new("c.txt")));
    }
}
