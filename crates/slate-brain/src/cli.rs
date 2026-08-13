//! Shared PATH / CLI resolution for subscription brains (Grok Build, Cursor CLI, Codex).
//! Port of `src/main/brain.ts` `resolveCli` / `brainEnv` / `which`.

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

/// Factory / agent call timeout (matches TS + design ~600s).
pub const CLI_TIMEOUT: Duration = Duration::from_secs(600);
const WHICH_TIMEOUT: Duration = Duration::from_secs(15);

/// ChatGPT desktop bundled codex — shared with Electron `brain.ts`.
#[cfg(target_os = "macos")]
pub(crate) const CODEX_BUNDLED: &str =
    "/Applications/ChatGPT.app/Contents/Resources/codex";

/// Extra dirs prepended to PATH when spawning CLIs (Electron-style minimal PATH).
pub(crate) fn cli_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = home_dir() {
        dirs.push(home.join(".grok").join("bin"));
    }
    dirs.push(PathBuf::from("/opt/homebrew/bin"));
    dirs.push(PathBuf::from("/usr/local/bin"));
    dirs.push(PathBuf::from("/snap/bin"));
    if let Some(home) = home_dir() {
        dirs.push(home.join(".local").join("bin"));
        dirs.push(home.join("bin"));
        dirs.push(home.join(".npm-global").join("bin"));
        dirs.push(home.join("scoop").join("shims"));
    }
    // Windows npm global: %APPDATA%\npm
    #[cfg(windows)]
    if let Ok(appdata) = std::env::var("APPDATA") {
        dirs.push(PathBuf::from(appdata).join("npm"));
    }
    #[cfg(windows)]
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        dirs.push(PathBuf::from(&local).join("cursor-agent"));
        dirs.push(
            PathBuf::from(&local)
                .join("Programs")
                .join("cursor")
                .join("resources")
                .join("app")
                .join("bin"),
        );
        dirs.push(
            PathBuf::from(local)
                .join("Microsoft")
                .join("WinGet")
                .join("Links"),
        );
    }
    #[cfg(windows)]
    {
        dirs.push(PathBuf::from(r"C:\ffmpeg\bin"));
        dirs.push(PathBuf::from(r"C:\Program Files\ffmpeg\bin"));
        dirs.push(PathBuf::from(r"C:\ProgramData\chocolatey\bin"));
        dirs.push(PathBuf::from(r"C:\Program Files\Git\usr\bin"));
    }
    dirs.push(PathBuf::from("/usr/bin"));
    dirs
}

fn home_dir() -> Option<PathBuf> {
    if let Ok(h) = std::env::var("USERPROFILE") {
        if !h.is_empty() {
            return Some(PathBuf::from(h));
        }
    }
    if let Ok(h) = std::env::var("HOME") {
        if !h.is_empty() {
            return Some(PathBuf::from(h));
        }
    }
    None
}

/// Candidate executable names for `name` on this platform.
pub(crate) fn cli_candidates(name: &str) -> Vec<String> {
    #[cfg(windows)]
    {
        // Never the extensionless Git-Bash shim — Node/Windows spawn EINVAL.
        vec![
            format!("{name}.exe"),
            format!("{name}.cmd"),
            format!("{name}.bat"),
        ]
    }
    #[cfg(not(windows))]
    {
        vec![name.to_string()]
    }
}

/// ChatGPT desktop bundled Codex (macOS app + Windows installers).
pub(crate) fn resolve_codex_bundled() -> Option<PathBuf> {
    let mut cands: Vec<PathBuf> = Vec::new();
    #[cfg(target_os = "macos")]
    cands.push(PathBuf::from(CODEX_BUNDLED));
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let local = PathBuf::from(local).join("Programs");
        cands.push(local.join("ChatGPT").join("resources").join("codex.exe"));
        cands.push(local.join("ChatGPT").join("resources").join("codex"));
        cands.push(
            local
                .join("ChatGPT")
                .join("resources")
                .join("app")
                .join("codex.exe"),
        );
        cands.push(local.join("chatgpt").join("resources").join("codex.exe"));
    }
    if let Ok(pf) = std::env::var("ProgramFiles") {
        let pf = PathBuf::from(pf).join("ChatGPT").join("resources");
        cands.push(pf.join("codex.exe"));
        cands.push(pf.join("codex"));
    }
    cands.into_iter().find(|p| p.is_file())
}

/// Resolve absolute path to a CLI, or bare name if not found (PATH fallback).
pub(crate) fn resolve_cli(name: &str) -> String {
    if name == "codex" {
        if let Some(bundled) = resolve_codex_bundled() {
            return bundled.to_string_lossy().into_owned();
        }
    }

    let candidates = cli_candidates(name);
    for dir in cli_dirs() {
        for cand in &candidates {
            let p = dir.join(cand);
            if p.is_file() {
                return p.to_string_lossy().into_owned();
            }
        }
    }

    // Also scan PATH entries for Windows .cmd / bare name.
    if let Ok(path) = std::env::var("PATH") {
        for entry in std::env::split_paths(&path) {
            for cand in &candidates {
                let p = entry.join(cand);
                if p.is_file() {
                    return p.to_string_lossy().into_owned();
                }
            }
        }
    }

    // Hope the OS can find it via PATH as-is.
    name.to_string()
}

/// Cursor CLI: spawn bundled `node.exe` + `index.js` (Windows) so we never
/// hit the Git-Bash shim at `~/.local/bin/cursor-agent` (spawn EINVAL).
pub(crate) fn resolve_cursor_launch() -> (String, Vec<String>) {
    #[cfg(windows)]
    {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let versions = PathBuf::from(local).join("cursor-agent").join("versions");
            if let Ok(rd) = std::fs::read_dir(&versions) {
                let mut dirs: Vec<_> = rd
                    .flatten()
                    .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                    .filter(|e| {
                        e.file_name()
                            .to_string_lossy()
                            .chars()
                            .next()
                            .is_some_and(|c| c.is_ascii_digit())
                    })
                    .collect();
                dirs.sort_by_key(|e| e.file_name());
                dirs.reverse();
                for e in dirs {
                    let dir = e.path();
                    let node = dir.join("node.exe");
                    let index = dir.join("index.js");
                    if node.is_file() && index.is_file() {
                        return (
                            node.to_string_lossy().into_owned(),
                            vec![index.to_string_lossy().into_owned()],
                        );
                    }
                }
            }
        }
    }
    (resolve_cli("cursor-agent"), vec![])
}

/// Official Grok Build (`~/.grok/bin/grok`). Never the colliding `agent` binary.
pub(crate) fn resolve_grok_launch() -> Option<String> {
    let candidates = cli_candidates("grok");
    if let Some(home) = home_dir() {
        let bin = home.join(".grok").join("bin");
        for cand in &candidates {
            let p = bin.join(cand);
            if p.is_file() {
                return Some(p.to_string_lossy().into_owned());
            }
        }
    }
    for dir in cli_dirs() {
        for cand in &candidates {
            let p = dir.join(cand);
            if p.is_file() {
                return Some(p.to_string_lossy().into_owned());
            }
        }
    }
    if let Ok(path) = std::env::var("PATH") {
        for entry in std::env::split_paths(&path) {
            for cand in &candidates {
                let p = entry.join(cand);
                if p.is_file() {
                    return Some(p.to_string_lossy().into_owned());
                }
            }
        }
    }
    None
}

/// PATH with CLI_DIRS prepended so child processes inherit Homebrew / user bins.
pub(crate) fn brain_path() -> String {
    let extra: Vec<String> = cli_dirs()
        .into_iter()
        .map(|d| d.to_string_lossy().into_owned())
        .collect();
    let existing: Vec<String> = std::env::var_os("PATH")
        .map(|p| {
            std::env::split_paths(&p)
                .map(|x| x.to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    let mut parts = extra;
    for s in existing {
        if !parts.iter().any(|p| p == &s) {
            parts.push(s);
        }
    }
    std::env::join_paths(parts.iter().map(PathBuf::from))
        .map(|os| os.to_string_lossy().into_owned())
        .unwrap_or_else(|_| parts.join(if cfg!(windows) { ";" } else { ":" }))
}

/// Run `cmd --version` (or given args) and return first stdout line, or None.
pub(crate) async fn which_cli(name: &str, args: &[&str]) -> Option<String> {
    let (resolved, prefix) = if name == "cursor-agent" {
        resolve_cursor_launch()
    } else if name == "grok" {
        match resolve_grok_launch() {
            Some(p) => (p, vec![]),
            None => return None,
        }
    } else {
        (resolve_cli(name), vec![])
    };
    let mut argv: Vec<String> = prefix;
    argv.extend(args.iter().map(|s| (*s).to_string()));
    let mut command = Command::new(&resolved);
    command
        .args(&argv)
        .env("PATH", brain_path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    let fut = async {
        let output = command.output().await.ok()?;
        if !output.status.success() && output.stdout.is_empty() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let first = stdout.lines().next().unwrap_or("").trim();
        if first.is_empty() {
            Some("available".to_string())
        } else {
            Some(first.to_string())
        }
    };

    match tokio::time::timeout(WHICH_TIMEOUT, fut).await {
        Ok(v) => v,
        Err(_) => None,
    }
}
