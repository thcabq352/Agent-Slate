mod cli;
mod cursor;
mod grok;
mod codex;
mod extract_json;
pub mod judge;
pub mod local;
pub mod run;
pub mod status;
mod types;
pub mod vision;

pub use cursor::{build_cursor_args, cursor_cli_model, parse_cursor_output, which_cursor};
pub use grok::{
    build_grok_build_args, grok_auth_looks_signed_in, grok_build_cli_model, grok_build_ready,
    parse_grok_build_output, which_grok,
};
pub use extract_json::extract_json;
pub use judge::{QualityGateConfig, QualityScores, QualityVerdict};
pub use local::{detect_local, parse_model_ids, run_local};
pub use run::brain_run;
pub use status::{brain_status, BackendAvailability, BrainStatus, LocalStatus};
pub use types::*;
pub use vision::{
    judge_vision_status, resolve_judge_model, JudgeModelStatus, JudgeResolveSource,
    DEFAULT_JUDGE_MODEL, DEFAULT_OLLAMA_ENDPOINT, JUDGE_MODEL_FALLBACKS,
};
