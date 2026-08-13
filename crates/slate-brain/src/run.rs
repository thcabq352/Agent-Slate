//! Brain run dispatch: local HTTP + Grok Build + Cursor CLI + Codex CLI adapters.

use crate::codex::run_codex;
use crate::cursor::run_cursor;
use crate::grok::{grok_build_ready, run_grok_build};
use crate::local::run_local;
use crate::types::{BrainBackend, BrainRequest, BrainResult};

/// Run `req` on the selected backend.
pub async fn brain_run(req: BrainRequest, backend: BrainBackend) -> BrainResult {
    match backend {
        BrainBackend::Local => run_local(&req).await,
        BrainBackend::Grok45 | BrainBackend::Grok46 => {
            if grok_build_ready() {
                run_grok_build(&req, backend).await
            } else {
                run_cursor(&req, backend).await
            }
        }
        BrainBackend::Cursor => run_cursor(&req, backend).await,
        BrainBackend::Codex => run_codex(&req).await,
    }
}
