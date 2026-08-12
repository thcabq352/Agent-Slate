mod claude;
mod codex;
mod extract_json;
pub mod local;
pub mod run;
pub mod status;
mod types;

pub use claude::{build_claude_args, parse_claude_output, which_claude};
pub use codex::{build_codex_args, which_codex};
pub use extract_json::extract_json;
pub use local::{detect_local, run_local};
pub use run::brain_run;
pub use status::{brain_status, BackendAvailability, BrainStatus, LocalStatus};
pub use types::*;
