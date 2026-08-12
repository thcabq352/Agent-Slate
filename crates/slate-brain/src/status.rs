//! Brain availability status (local probe + Claude/Codex CLI `--version`).

use crate::claude::which_claude;
use crate::codex::which_codex;
use crate::local::detect_local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub claude: BackendAvailability,
    pub codex: BackendAvailability,
    pub local: LocalStatus,
}

/// Probe Claude, Codex, and local OpenAI-compat backends in parallel.
pub async fn brain_status(local_endpoint: Option<&str>) -> BrainStatus {
    let (claude_v, codex_v, local_probe) = tokio::join!(
        which_claude(),
        which_codex(),
        detect_local(local_endpoint),
    );

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
        claude: BackendAvailability {
            available: claude_v.is_some(),
            version: claude_v,
        },
        codex: BackendAvailability {
            available: codex_v.is_some(),
            version: codex_v,
        },
        local,
    }
}
