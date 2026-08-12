//! Local vision / judge model resolution (Ollama-first).
//!
//! Prefer `qwen3.5:9b` (vision on this host). Do not ship weights — only resolve
//! tags that are already installed on the user's Ollama (or OpenAI-compat) server.

use crate::local::{detect_local, normalize_endpoint};
use serde::{Deserialize, Serialize};

/// Default Ollama OpenAI-compat base (`/v1`).
pub const DEFAULT_OLLAMA_ENDPOINT: &str = "http://127.0.0.1:11434/v1";

/// Preferred VL judge model (must be installed via `ollama pull`; not bundled).
pub const DEFAULT_JUDGE_MODEL: &str = "qwen3.5:9b";

/// Fallback order when preferred tag is missing (first match in server's model list wins).
pub const JUDGE_MODEL_FALLBACKS: &[&str] = &[
    "qwen3.5:9b",
    "qwen3-vl:8b",
    "qwen3-vl:30b",
    "qwen3.6:35b",
    "llava",
    "llava:latest",
];

/// How the judge model was chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JudgeResolveSource {
    /// Explicit config / env / request override and present on server.
    Configured,
    /// Preferred default tag found on server.
    Preferred,
    /// A known fallback tag found on server.
    Fallback,
    /// Heuristic: first listed model whose id looks vision-capable.
    Heuristic,
    /// No suitable model.
    None,
}

/// Result of resolving a local VL judge model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JudgeModelStatus {
    /// True when endpoint is up and a model was selected.
    pub ready: bool,
    /// Resolved OpenAI-compat endpoint ending in `/v1`, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    /// Selected model id/tag, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Preferred default when none configured.
    pub preferred_model: String,
    pub source: JudgeResolveSource,
    /// Models reported by the local server.
    #[serde(default)]
    pub available_models: Vec<String>,
    /// Human-readable hint when not ready.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

fn looks_like_vision_model(id: &str) -> bool {
    let l = id.to_ascii_lowercase();
    l.contains("vl")
        || l.contains("vision")
        || l.contains("llava")
        || l.contains("minicpm-v")
        || l.contains("qwen3.5")
        || l.contains("qwen2.5-vl")
        || l.contains("qwen2-vl")
        || l.contains("gemma3")
        || l.contains("pixtral")
}

fn list_contains_model(models: &[String], want: &str) -> bool {
    let want_l = want.to_ascii_lowercase();
    models.iter().any(|m| {
        let m_l = m.to_ascii_lowercase();
        m_l == want_l || m_l.starts_with(&format!("{want_l}:")) || m_l.starts_with(&want_l)
    })
}

fn pick_exact<'a>(models: &'a [String], want: &str) -> Option<&'a str> {
    let want_l = want.to_ascii_lowercase();
    models
        .iter()
        .find(|m| m.to_ascii_lowercase() == want_l)
        .map(|s| s.as_str())
        .or_else(|| {
            models
                .iter()
                .find(|m| {
                    let m_l = m.to_ascii_lowercase();
                    m_l.starts_with(&format!("{want_l}:")) || m_l == want_l
                })
                .map(|s| s.as_str())
        })
}

/// Resolve which VL model to use for judging.
///
/// Order:
/// 1. `configured_model` if set and present on the server
/// 2. Preferred `qwen3.5:9b` if present
/// 3. Entries in [`JUDGE_MODEL_FALLBACKS`] in order
/// 4. First model whose name looks vision-capable
pub fn resolve_judge_model(
    available_models: &[String],
    configured_model: Option<&str>,
) -> (Option<String>, JudgeResolveSource) {
    if available_models.is_empty() {
        return (None, JudgeResolveSource::None);
    }

    if let Some(cfg) = configured_model.map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(hit) = pick_exact(available_models, cfg) {
            return (Some(hit.to_string()), JudgeResolveSource::Configured);
        }
        // Configured but missing: fall through to preferred/fallbacks
    }

    if let Some(hit) = pick_exact(available_models, DEFAULT_JUDGE_MODEL) {
        return (Some(hit.to_string()), JudgeResolveSource::Preferred);
    }

    for fb in JUDGE_MODEL_FALLBACKS {
        if *fb == DEFAULT_JUDGE_MODEL {
            continue;
        }
        if let Some(hit) = pick_exact(available_models, fb) {
            return (Some(hit.to_string()), JudgeResolveSource::Fallback);
        }
    }

    if let Some(hit) = available_models.iter().find(|m| looks_like_vision_model(m)) {
        return (Some(hit.clone()), JudgeResolveSource::Heuristic);
    }

    (None, JudgeResolveSource::None)
}

fn not_ready_hint(endpoint: Option<&str>, preferred: &str, models: &[String]) -> String {
    match endpoint {
        None => format!(
            "No local model server found. Start Ollama (`ollama serve`) and pull a vision model: ollama pull {preferred}"
        ),
        Some(_) if models.is_empty() => format!(
            "Local server is up but has no models. Run: ollama pull {preferred}"
        ),
        Some(_) => format!(
            "No vision judge model found. Prefer `{preferred}`. Install with: ollama pull {preferred}  (or set SLATE_JUDGE_MODEL to an installed VL tag)"
        ),
    }
}

/// Probe local/Ollama and resolve the judge VL model.
///
/// `preferred_endpoint`: override base URL (OpenAI-compat `/v1` form).  
/// `configured_model`: override model tag (e.g. from `SLATE_JUDGE_MODEL`).
pub async fn judge_vision_status(
    preferred_endpoint: Option<&str>,
    configured_model: Option<&str>,
) -> JudgeModelStatus {
    let preferred = DEFAULT_JUDGE_MODEL.to_string();

    // Prefer Ollama endpoint first when no override.
    let (endpoint, models) = if let Some(ep) = preferred_endpoint.filter(|s| !s.is_empty()) {
        detect_local(Some(ep)).await
    } else {
        // Try Ollama explicitly first, then generic detect.
        let ollama = detect_local(Some(DEFAULT_OLLAMA_ENDPOINT)).await;
        if ollama.0.is_some() {
            ollama
        } else {
            detect_local(None).await
        }
    };

    let (model, source) = resolve_judge_model(&models, configured_model);
    let ready = endpoint.is_some() && model.is_some();
    let hint = if ready {
        None
    } else {
        Some(not_ready_hint(
            endpoint.as_deref(),
            &preferred,
            &models,
        ))
    };

    JudgeModelStatus {
        ready,
        endpoint: endpoint.map(|e| normalize_endpoint(&e)),
        model,
        preferred_model: preferred,
        source,
        available_models: models,
        hint,
    }
}

/// True if `model` appears in the local server's list (after optional resolve).
pub fn model_is_available(models: &[String], model: &str) -> bool {
    list_contains_model(models, model)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_configured_when_present() {
        let models = vec![
            "qwen3.5:9b".into(),
            "qwen3-vl:30b".into(),
            "gemma4:latest".into(),
        ];
        let (m, src) = resolve_judge_model(&models, Some("qwen3-vl:30b"));
        assert_eq!(m.as_deref(), Some("qwen3-vl:30b"));
        assert_eq!(src, JudgeResolveSource::Configured);
    }

    #[test]
    fn prefers_qwen35_9b_by_default() {
        let models = vec![
            "qwen3-vl:30b".into(),
            "qwen3.5:9b".into(),
            "nomic-embed-text:latest".into(),
        ];
        let (m, src) = resolve_judge_model(&models, None);
        assert_eq!(m.as_deref(), Some("qwen3.5:9b"));
        assert_eq!(src, JudgeResolveSource::Preferred);
    }

    #[test]
    fn falls_back_when_preferred_missing() {
        let models = vec!["qwen3-vl:30b".into(), "llava:latest".into()];
        let (m, src) = resolve_judge_model(&models, None);
        assert_eq!(m.as_deref(), Some("qwen3-vl:30b"));
        assert_eq!(src, JudgeResolveSource::Fallback);
    }

    #[test]
    fn configured_missing_falls_through_to_preferred() {
        let models = vec!["qwen3.5:9b".into()];
        let (m, src) = resolve_judge_model(&models, Some("qwen3-vl:8b"));
        assert_eq!(m.as_deref(), Some("qwen3.5:9b"));
        assert_eq!(src, JudgeResolveSource::Preferred);
    }

    #[test]
    fn empty_list_none() {
        let (m, src) = resolve_judge_model(&[], None);
        assert!(m.is_none());
        assert_eq!(src, JudgeResolveSource::None);
    }
}
