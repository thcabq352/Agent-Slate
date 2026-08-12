//! Quality-gate contracts for vision judging (Phase 0).
//!
//! The full auto accept/retry loop is Phase 2; these types are the shared
//! schema for health, tools, and future factory integration.

use serde::{Deserialize, Serialize};

/// Per-criterion scores in \[0.0, 1.0\].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityScores {
    /// Overall visual polish (noise, focus, composition).
    pub visual_quality: f64,
    /// Match to prior shots / bible continuity.
    pub continuity: f64,
    /// Freedom from artifacts (hands, text, warping). Higher = fewer artifacts.
    pub artifacts: f64,
    /// How well the image matches the prompt intent.
    pub prompt_fidelity: f64,
}

impl QualityScores {
    /// Mean of the four criteria.
    pub fn mean(&self) -> f64 {
        (self.visual_quality + self.continuity + self.artifacts + self.prompt_fidelity) / 4.0
    }
}

/// Structured judge result for a single take / media file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityVerdict {
    /// Whether the take should be accepted without human override.
    pub accept: bool,
    pub scores: QualityScores,
    /// Overall 0–1 score (usually mean of criteria).
    pub overall: f64,
    /// Short human-readable issues.
    #[serde(default)]
    pub issues: Vec<String>,
    /// Hints for a retry (seed, prompt tighten, guidance, etc.).
    #[serde(default)]
    pub retry_hints: Vec<String>,
    /// Free-text summary from the VL model.
    #[serde(default)]
    pub summary: String,
    /// Model tag used for this verdict.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub judge_model: Option<String>,
    /// Path that was judged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_path: Option<String>,
}

/// Defaults for the quality gate (Phase 0 contracts).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityGateConfig {
    /// Minimum overall score to auto-accept (inclusive).
    pub pass_threshold: f64,
    /// Max automatic retries after a reject (not counting the first attempt).
    pub max_retries: u32,
    /// Prefer this Ollama/OpenAI-compat model for vision.
    pub judge_model: String,
    /// OpenAI-compat base URL ending in `/v1` (Ollama default).
    pub judge_endpoint: String,
}

impl Default for QualityGateConfig {
    fn default() -> Self {
        Self {
            pass_threshold: 0.7,
            max_retries: 2,
            judge_model: crate::vision::DEFAULT_JUDGE_MODEL.to_string(),
            judge_endpoint: crate::vision::DEFAULT_OLLAMA_ENDPOINT.to_string(),
        }
    }
}

impl QualityVerdict {
    /// Build accept/reject from scores + threshold.
    pub fn from_scores(
        scores: QualityScores,
        threshold: f64,
        issues: Vec<String>,
        retry_hints: Vec<String>,
        summary: String,
    ) -> Self {
        let overall = scores.mean();
        Self {
            accept: overall >= threshold,
            scores,
            overall,
            issues,
            retry_hints,
            summary,
            judge_model: None,
            media_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_and_threshold() {
        let s = QualityScores {
            visual_quality: 0.8,
            continuity: 0.8,
            artifacts: 0.8,
            prompt_fidelity: 0.8,
        };
        let v = QualityVerdict::from_scores(s, 0.7, vec![], vec![], "ok".into());
        assert!(v.accept);
        assert!((v.overall - 0.8).abs() < 1e-9);
    }

    #[test]
    fn reject_below_threshold() {
        let s = QualityScores {
            visual_quality: 0.5,
            continuity: 0.5,
            artifacts: 0.5,
            prompt_fidelity: 0.5,
        };
        let v = QualityVerdict::from_scores(s, 0.7, vec!["soft".into()], vec!["raise steps".into()], "meh".into());
        assert!(!v.accept);
        assert_eq!(v.issues.len(), 1);
    }
}
