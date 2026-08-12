//! Control descriptor — loopback port + bearer token on disk for MCP / clients.
//! Mirrors Electron `src/main/control.ts`.

use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// On-disk control descriptor written when the HTTP server starts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlDescriptor {
    pub v: u32,
    pub app: String,
    pub port: u16,
    pub token: String,
    pub pid: u32,
}

/// Path to `control.json`:
/// - Windows: `%APPDATA%\slate\control.json`
/// - Unix: `~/.config/slate/control.json`
pub fn descriptor_path() -> PathBuf {
    let base = if cfg!(windows) {
        dirs::data_dir()
            .or_else(|| {
                std::env::var_os("APPDATA")
                    .map(PathBuf::from)
                    .or_else(|| dirs::home_dir().map(|h| h.join("AppData").join("Roaming")))
            })
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        dirs::config_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
            .unwrap_or_else(|| PathBuf::from("."))
    };
    base.join("slate").join("control.json")
}

/// Write `{ v, app, port, token, pid }` to [`descriptor_path`], creating parent dirs.
/// Sets file mode `0o600` on Unix when possible.
pub fn write_control_descriptor(port: u16, token: &str) -> io::Result<PathBuf> {
    let path = descriptor_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let desc = ControlDescriptor {
        v: 1,
        app: "slate".to_string(),
        port,
        token: token.to_string(),
        pid: std::process::id(),
    };
    let json = serde_json::to_string_pretty(&desc)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(&path, json.as_bytes())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }

    Ok(path)
}
