//! Brain run dispatch: local HTTP + Claude Code + Codex CLI adapters.

use crate::claude::run_claude;
use crate::codex::run_codex;
use crate::local::run_local;
use crate::types::{BrainBackend, BrainRequest, BrainResult};

/// Run `req` on the selected backend.
pub async fn brain_run(req: BrainRequest, backend: BrainBackend) -> BrainResult {
    match backend {
        BrainBackend::Local => run_local(&req).await,
        BrainBackend::Claude => run_claude(&req).await,
        BrainBackend::Codex => run_codex(&req).await,
    }
}
