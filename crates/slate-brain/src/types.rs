//! Brain request/result types — field names mirror `src/shared/types.ts` (camelCase).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// Which engine powers agent tasks. `local` talks to any OpenAI-compatible
/// localhost server (Ollama, LM Studio, vLLM, llama.cpp, KoboldCpp, Jan…).
/// Grok 4.5 / 4.6 prefer Grok Build (`grok login`), then Cursor (`cursor-agent`).
/// Composer stays on Cursor CLI. The `claude` alias maps old project JSON onto Cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BrainBackend {
    #[serde(alias = "claude")]
    Cursor,
    #[serde(rename = "grok-4.5", alias = "grok45")]
    Grok45,
    #[serde(rename = "grok-4.6", alias = "grok46")]
    Grok46,
    Codex,
    Local,
}

impl BrainBackend {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "cursor" | "claude" => Some(Self::Cursor),
            "grok-4.5" | "grok45" | "grok4.5" => Some(Self::Grok45),
            "grok-4.6" | "grok46" | "grok4.6" => Some(Self::Grok46),
            "codex" => Some(Self::Codex),
            "local" => Some(Self::Local),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cursor => "cursor",
            Self::Grok45 => "grok-4.5",
            Self::Grok46 => "grok-4.6",
            Self::Codex => "codex",
            Self::Local => "local",
        }
    }

    pub fn uses_cursor_cli(self) -> bool {
        matches!(self, Self::Cursor | Self::Grok45 | Self::Grok46)
    }

    pub fn is_grok_brain(self) -> bool {
        matches!(self, Self::Grok45 | Self::Grok46)
    }
}

/// Model quality / cost tier for Cursor Composer (and similar) backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BrainTier {
    Fast,
    Standard,
    Top,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrainRequest {
    pub id: String,
    pub task: String,
    pub system: String,
    pub prompt: String,
    #[serde(default)]
    pub images: Vec<PathBuf>,
    pub tier: BrainTier,
    #[serde(default)]
    pub expect_json: bool,
    /// Local backend only: server base URL override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_endpoint: Option<String>,
    /// Local backend only: model id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrainResult {
    pub id: String,
    pub ok: bool,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub json: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub elapsed_ms: u64,
}
