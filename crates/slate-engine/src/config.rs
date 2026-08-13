//! Engine configuration from environment variables.

use std::env;
use std::path::PathBuf;

use slate_brain::{QualityGateConfig, DEFAULT_JUDGE_MODEL, DEFAULT_OLLAMA_ENDPOINT};
use slate_comfy::DEFAULT_COMFY_BASE;

/// Runtime configuration for the slate-engine process.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Project store root (Documents/Slate or `SLATE_DATA_DIR`).
    pub data_dir: PathBuf,
    /// ComfyUI HTTP base URL (`SLATE_COMFY_URL`, default `http://127.0.0.1:8188`).
    pub comfy_base_url: String,
    /// Workflow packs directory (`SLATE_PACKS_DIR`, else next to the engine binary).
    pub packs_dir: PathBuf,
    /// Default brain backend name: `cursor` | `grok-4.5` | `grok-4.6` | `codex` | `local` (`SLATE_BRAIN`).
    pub brain_default: String,
    /// HTTP bind host (loopback only by default).
    pub bind: String,
    /// When true, skip Comfy GPU work (`SLATE_DRY_RUN`).
    pub dry_run: bool,
    /// Preferred VL judge model tag (`SLATE_JUDGE_MODEL`, default `qwen3.5:9b`).
    pub judge_model: String,
    /// OpenAI-compat endpoint for the judge (`SLATE_JUDGE_ENDPOINT`, default Ollama).
    pub judge_endpoint: String,
    /// Auto-accept threshold 0–1 (`SLATE_JUDGE_PASS_THRESHOLD`, default 0.7).
    pub judge_pass_threshold: f64,
    /// Max auto retries after reject (`SLATE_JUDGE_MAX_RETRIES`, default 2).
    pub judge_max_retries: u32,
}

impl EngineConfig {
    /// Quality-gate slice of config (Phase 0 contract).
    pub fn quality_gate(&self) -> QualityGateConfig {
        QualityGateConfig {
            pass_threshold: self.judge_pass_threshold,
            max_retries: self.judge_max_retries,
            judge_model: self.judge_model.clone(),
            judge_endpoint: self.judge_endpoint.clone(),
        }
    }
}

fn env_truthy(key: &str) -> bool {
    env::var(key)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
        .unwrap_or(false)
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

    let packs_dir = slate_comfy::resolve_packs_dir();

    let brain_default = env::var("SLATE_BRAIN")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "local".to_string());

    let dry_run = env_truthy("SLATE_DRY_RUN");

    let judge_model = env::var("SLATE_JUDGE_MODEL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_JUDGE_MODEL.to_string());

    let judge_endpoint = env::var("SLATE_JUDGE_ENDPOINT")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_OLLAMA_ENDPOINT.to_string());

    let judge_pass_threshold = env::var("SLATE_JUDGE_PASS_THRESHOLD")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|t| (0.0..=1.0).contains(t))
        .unwrap_or(0.7);

    let judge_max_retries = env::var("SLATE_JUDGE_MAX_RETRIES")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(2);

    EngineConfig {
        data_dir,
        comfy_base_url,
        packs_dir,
        brain_default,
        bind: "127.0.0.1".to_string(),
        dry_run,
        judge_model,
        judge_endpoint,
        judge_pass_threshold,
        judge_max_retries,
    }
}

/// Apply config side-effects so domain/comfy helpers that read env stay in sync.
pub fn apply_env(config: &EngineConfig) {
    env::set_var("SLATE_DATA_DIR", &config.data_dir);
    if config.dry_run {
        env::set_var("SLATE_DRY_RUN", "1");
    }
    env::set_var("SLATE_JUDGE_MODEL", &config.judge_model);
    env::set_var("SLATE_JUDGE_ENDPOINT", &config.judge_endpoint);
}
