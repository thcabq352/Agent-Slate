//! slate-comfy — ComfyUI pack manifests and workflow field injection.
//!
//! Task 8: load pack manifests and inject logical values into API-format graphs.
//! Task 9 will add the HTTP client and default-still pack.

mod inject;
mod manifest;

pub use inject::inject_workflow;
pub use manifest::{load_manifest, InputMap, OutputMap, PackLimits, PackManifest};

/// Crate-level result alias.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors from manifest load and workflow inject.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error reading {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("inject: {0}")]
    Inject(String),
}
