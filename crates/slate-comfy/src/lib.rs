//! slate-comfy — ComfyUI pack manifests, workflow inject, and HTTP client.
//!
//! Task 8: load pack manifests and inject logical values into API-format graphs.
//! Task 9: HTTP client, dry-run generate, default-still pack.

mod client;
mod inject;
mod manifest;

pub use client::{
    collect_output_files, generate_to_file, load_pack, ComfyClient, ComfyFileRef,
    DEFAULT_COMFY_BASE, SLATE_DRY_RUN_ENV,
};
pub use inject::inject_workflow;
pub use manifest::{load_manifest, InputMap, OutputMap, PackLimits, PackManifest};

/// Crate-level result alias.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors from manifest load, workflow inject, and Comfy HTTP.
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
    #[error("http: {0}")]
    Http(String),
    #[error("comfy: {0}")]
    Comfy(String),
    #[error("timeout: {0}")]
    Timeout(String),
}
