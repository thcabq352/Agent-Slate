mod claude;
mod codex;
mod extract_json;
pub mod judge;
pub mod local;
pub mod run;
pub mod status;
mod types;
pub mod vision;

pub use claude::{build_claude_args, parse_claude_output, which_claude};
pub use codex::{build_codex_args, which_codex};
pub use extract_json::extract_json;
pub use judge::{QualityGateConfig, QualityScores, QualityVerdict};
pub use local::{detect_local, run_local};
pub use run::brain_run;
pub use status::{brain_status, BackendAvailability, BrainStatus, LocalStatus};
pub use types::*;
pub use vision::{
    judge_vision_status, resolve_judge_model, JudgeModelStatus, JudgeResolveSource,
    DEFAULT_JUDGE_MODEL, DEFAULT_OLLAMA_ENDPOINT, JUDGE_MODEL_FALLBACKS,
};
