//! Engine configuration from environment variables.

use std::env;
use std::path::PathBuf;

use slate_comfy::DEFAULT_COMFY_BASE;

/// Runtime configuration for the slate-engine process.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Project store root (Documents/Slate or `SLATE_DATA_DIR`).
    pub data_dir: PathBuf,
    /// ComfyUI HTTP base URL (`SLATE_COMFY_URL`, default `http://127.0.0.1:8188`).
    pub comfy_base_url: String,
    /// Workflow packs directory (`SLATE_PACKS_DIR`).
    pub packs_dir: PathBuf,
    /// Default brain backend name: `claude` | `codex` | `local` (`SLATE_BRAIN`).
    pub brain_default: String,
    /// HTTP bind host (loopback only by default).
    pub bind: String,
    /// When true, skip Comfy GPU work (`SLATE_DRY_RUN`).
    pub dry_run: bool,
}

/// Load config from environment. Missing vars get sensible defaults.
pub fn load_config() -> EngineConfig {
    let data_dir = env::var("SLATE_DATA_DIR")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(slate_domain::projects_root);

    let comfy_base_url = env::var("SLATE_COMFY_URL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_COMFY_BASE.to_string());

    let packs_dir = env::var("SLATE_PACKS_DIR")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            // Prefer cwd/workflows/packs, else relative to executable location later.
            env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("workflows")
                .join("packs")
        });

    let brain_default = env::var("SLATE_BRAIN")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "local".to_string());

    let dry_run = env::var("SLATE_DRY_RUN")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
        .unwrap_or(false);

    EngineConfig {
        data_dir,
        comfy_base_url,
        packs_dir,
        brain_default,
        bind: "127.0.0.1".to_string(),
        dry_run,
    }
}

/// Apply config side-effects so domain/comfy helpers that read env stay in sync.
pub fn apply_env(config: &EngineConfig) {
    env::set_var("SLATE_DATA_DIR", &config.data_dir);
    if config.dry_run {
        env::set_var("SLATE_DRY_RUN", "1");
    }
}
