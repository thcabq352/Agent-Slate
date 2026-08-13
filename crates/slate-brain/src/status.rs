//! Brain availability status (local probe + Cursor/Grok Build/Codex CLI `--version`).

use crate::codex::which_codex;
use crate::cursor::which_cursor;
use crate::grok::{grok_build_oauth_present, which_grok};
use crate::local::detect_local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendAvailability {
    pub available: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalStatus {
    pub available: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

/// Which brains are installed / reachable. Mirrors `BrainStatus` in TS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrainStatus {
    pub cursor: BackendAvailability,
    #[serde(default)]
    pub grok: BackendAvailability,
    pub codex: BackendAvailability,
    pub local: LocalStatus,
}

/// Probe Cursor, Grok Build, Codex, and local OpenAI-compat backends in parallel.
pub async fn brain_status(local_endpoint: Option<&str>) -> BrainStatus {
    let (cursor_v, grok_v, codex_v, local_probe) = tokio::join!(
        which_cursor(),
        which_grok(),
        which_codex(),
        detect_local(local_endpoint),
    );
    let grok_oauth = grok_build_oauth_present();

    let (endpoint, models) = local_probe;
    let local = match endpoint {
        Some(ep) => {
            let host = ep
                .strip_prefix("https://")
                .or_else(|| ep.strip_prefix("http://"))
                .unwrap_or(&ep);
            LocalStatus {
                available: true,
                version: Some(format!("{} model(s) @ {host}", models.len())),
                endpoint: Some(ep),
            }
        }
        None => LocalStatus {
            available: false,
            version: None,
            endpoint: None,
        },
    };

    BrainStatus {
        cursor: BackendAvailability {
            available: cursor_v.is_some(),
            version: cursor_v,
        },
        grok: BackendAvailability {
            available: grok_v.is_some() && grok_oauth,
            version: grok_v,
        },
        codex: BackendAvailability {
            available: codex_v.is_some(),
            version: codex_v,
        },
        local,
    }
}
