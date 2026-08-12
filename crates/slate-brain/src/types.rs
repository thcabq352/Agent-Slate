//! Brain request/result types — field names mirror `src/shared/types.ts` (camelCase).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// Which engine powers agent tasks. `local` talks to any OpenAI-compatible
/// localhost server (Ollama, LM Studio, vLLM, llama.cpp, KoboldCpp, Jan…).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BrainBackend {
    Claude,
    Codex,
    Local,
}

/// Model quality / cost tier for Claude (and similar) backends.
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
