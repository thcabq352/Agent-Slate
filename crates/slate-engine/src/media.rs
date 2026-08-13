//! Local ffmpeg helpers — video frame grab + simple cut assemble.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use serde_json::{json, Value};

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

fn install_hint() -> &'static str {
    if cfg!(windows) {
        "Install ffmpeg (`winget install Gyan.FFmpeg`) or set SLATE_FFMPEG to ffmpeg.exe"
    } else if cfg!(target_os = "macos") {
        "Install ffmpeg (`brew install ffmpeg`) or set SLATE_FFMPEG"
    } else {
        "Install ffmpeg (`sudo apt install ffmpeg`) or set SLATE_FFMPEG"
    }
}

fn is_ffmpeg_file(path: &Path) -> bool {
    path.is_file()
        && path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.eq_ignore_ascii_case("ffmpeg") || n.eq_ignore_ascii_case("ffmpeg.exe"))
}

fn env_ffmpeg() -> Option<PathBuf> {
    for key in ["SLATE_FFMPEG", "FFMPEG"] {
        if let Ok(raw) = env::var(key) {
            if raw.is_empty() {
                continue;
            }
            let p = PathBuf::from(&raw);
            if is_ffmpeg_file(&p) {
                return Some(p);
            }
            let nested = if cfg!(windows) {
                p.join("ffmpeg.exe")
            } else {
                p.join("ffmpeg")
            };
            if is_ffmpeg_file(&nested) {
                return Some(nested);
            }
        }
    }
    None
}

fn candidate_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/snap/bin"),
    ];
    if let Ok(home) = env::var("USERPROFILE").or_else(|_| env::var("HOME")) {
        let home = PathBuf::from(home);
        dirs.push(home.join(".local").join("bin"));
        dirs.push(home.join("bin"));
        dirs.push(home.join("scoop").join("shims"));
    }
    dirs.push(PathBuf::from(r"C:\ffmpeg\bin"));
    dirs.push(PathBuf::from(r"C:\Program Files\ffmpeg\bin"));
    dirs.push(PathBuf::from(r"C:\Program Files (x86)\ffmpeg\bin"));
    dirs.push(PathBuf::from(r"C:\Program Files\Gyan\ffmpeg\bin"));
    dirs.push(PathBuf::from(r"C:\ProgramData\chocolatey\bin"));
    dirs.push(PathBuf::from(r"C:\Program Files\Git\usr\bin"));
    if let Ok(local) = env::var("LOCALAPPDATA") {
        dirs.push(
            PathBuf::from(local)
                .join("Microsoft")
                .join("WinGet")
                .join("Links"),
        );
    }
    if let Ok(path) = env::var("PATH") {
        for entry in env::split_paths(&path) {
            dirs.push(entry);
        }
    }
    dirs
}

fn find_named(dir: &Path, depth: u32) -> Option<PathBuf> {
    let names = ["ffmpeg.exe", "ffmpeg"];
    for name in names {
        let p = dir.join(name);
        if is_ffmpeg_file(&p) {
            return Some(p);
        }
    }
    if depth == 0 {
        return None;
    }
    let rd = std::fs::read_dir(dir).ok()?;
    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
            if let Some(found) = find_named(&p, depth - 1) {
                return Some(found);
            }
        }
    }
    None
}

fn probe_winget() -> Option<PathBuf> {
    let local = env::var("LOCALAPPDATA").ok()?;
    let root = PathBuf::from(local)
        .join("Microsoft")
        .join("WinGet")
        .join("Packages");
    let rd = std::fs::read_dir(root).ok()?;
    for ent in rd.flatten() {
        let name = ent.file_name();
        if !name.to_string_lossy().to_ascii_lowercase().contains("ffmpeg") {
            continue;
        }
        if let Some(p) = find_named(&ent.path(), 3) {
            return Some(p);
        }
    }
    None
}

fn probe_ffmpeg() -> Option<PathBuf> {
    let exe = if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" };
    for dir in candidate_dirs() {
        let p = dir.join(exe);
        if is_ffmpeg_file(&p) {
            return Some(p);
        }
    }
    probe_winget()
}

/// Resolved ffmpeg path (SLATE_FFMPEG, then common install dirs, then PATH).
pub fn resolve_ffmpeg() -> Option<PathBuf> {
    if let Some(p) = env_ffmpeg() {
        return Some(p);
    }
    static CACHED: OnceLock<Option<PathBuf>> = OnceLock::new();
    CACHED.get_or_init(probe_ffmpeg).clone()
}

fn ffmpeg_bin() -> Result<PathBuf, String> {
    resolve_ffmpeg().ok_or_else(|| format!("ffmpeg not found. {}", install_hint()))
}

/// Health payload for `slate_health`.
pub fn ffmpeg_status() -> Value {
    match resolve_ffmpeg() {
        Some(p) => json!({
            "ok": true,
            "path": p,
            "hint": Value::Null,
        }),
        None => json!({
            "ok": false,
            "path": Value::Null,
            "hint": install_hint(),
        }),
    }
}

/// Grab the first frame of a video into `dest` (png). Returns `dest`.
pub fn extract_first_frame(video: &Path, dest: &Path) -> Result<PathBuf, String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    }
    let status = Command::new(ffmpeg_bin()?)
        .args(["-y", "-hide_banner", "-loglevel", "error", "-i"])
        .arg(video)
        .args(["-frames:v", "1", "-q:v", "2"])
        .arg(dest)
        .status()
        .map_err(|e| format!("ffmpeg not found ({e}). {}", install_hint()))?;
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
    let status = Command::new(ffmpeg_bin()?)
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
        .map_err(|e| format!("ffmpeg not found ({e}). {}", install_hint()))?;
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

    let status = Command::new(ffmpeg_bin()?)
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
        .map_err(|e| format!("ffmpeg not found ({e}). {}", install_hint()))?;
    if !status.success() {
        // Re-encode if copy-concat fails (mixed codecs).
        let status = Command::new(ffmpeg_bin()?)
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
            .map_err(|e| format!("ffmpeg not found ({e}). {}", install_hint()))?;
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

    #[test]
    fn slate_ffmpeg_env_wins() {
        let dir = tempfile::tempdir().unwrap();
        let fake = if cfg!(windows) {
            dir.path().join("ffmpeg.exe")
        } else {
            dir.path().join("ffmpeg")
        };
        std::fs::write(&fake, b"").unwrap();
        std::env::set_var("SLATE_FFMPEG", &fake);
        let resolved = env_ffmpeg().expect("env ffmpeg");
        assert_eq!(resolved, fake);
        std::env::remove_var("SLATE_FFMPEG");
    }
}
