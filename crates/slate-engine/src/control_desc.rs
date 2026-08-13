//! Control descriptor — loopback port + bearer token on disk for HTTP clients.
//!
//! Electron writes a *different* file (`electron-control.json`, app `slate-electron`)
//! from `src/main/control.ts`. Do not share a path with the studio MCP bridge.

use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// `app` field written into the engine descriptor. Clients must match this
/// so they never invoke the Electron control server by mistake.
pub const APP_NAME: &str = "slate-engine";

/// Filename under the slate config dir (not `control.json` — that collided
/// with Electron).
pub const DESCRIPTOR_FILE: &str = "engine-control.json";

/// On-disk control descriptor written when the HTTP server starts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlDescriptor {
    pub v: u32,
    pub app: String,
    pub port: u16,
    pub token: String,
    pub pid: u32,
}

/// Path to `engine-control.json`:
/// - Windows: `%APPDATA%\slate\engine-control.json`
/// - Unix: `~/.config/slate/engine-control.json`
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
    base.join("slate").join(DESCRIPTOR_FILE)
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
        app: APP_NAME.to_string(),
        port,
        token: token.to_string(),
        pid: std::process::id(),
    };
    let json =
        serde_json::to_string_pretty(&desc).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(&path, json.as_bytes())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }

    Ok(path)
}
