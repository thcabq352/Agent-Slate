//! Brain availability status (local probe now; Claude/Codex CLI in Task 7).

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

/// Probe local OpenAI-compat server; Claude/Codex left unavailable until Task 7.
pub async fn brain_status(local_endpoint: Option<&str>) -> BrainStatus {
    let (endpoint, models) = detect_local(local_endpoint).await;
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
        // Task 7 wires CLI detection.
        claude: BackendAvailability {
            available: false,
            version: None,
        },
        codex: BackendAvailability {
            available: false,
            version: None,
        },
        local,
    }
}
