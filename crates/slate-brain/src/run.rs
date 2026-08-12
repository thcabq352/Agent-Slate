//! Brain run dispatch: local HTTP adapter + Claude/Codex stubs (Task 7).

use crate::local::run_local;
use crate::types::{BrainBackend, BrainRequest, BrainResult};
use std::time::Instant;

/// Run `req` on the selected backend.
///
/// Claude and Codex return `ok: false` with `"not implemented"` until Task 7.
pub async fn brain_run(req: BrainRequest, backend: BrainBackend) -> BrainResult {
    match backend {
        BrainBackend::Local => run_local(&req).await,
        BrainBackend::Claude | BrainBackend::Codex => {
            let started = Instant::now();
            BrainResult {
                id: req.id,
                ok: false,
                text: String::new(),
                json: None,
                error: Some("not implemented".into()),
                elapsed_ms: started.elapsed().as_millis() as u64,
            }
        }
    }
}
