//! Phase 2 — post-generate quality gate using local VL (Ollama / OpenAI-compat).
//!
//! After a take is written, score it with the configured judge model and either
//! accept or request a retry (bounded). Dry-run and missing vision skip judging.

use std::path::Path;

use serde_json::Value;
use slate_brain::{
    brain_run, extract_json, judge_vision_status, BrainBackend, BrainRequest, BrainTier,
    QualityGateConfig, QualityScores, QualityVerdict, DEFAULT_JUDGE_MODEL,
};

use crate::config::EngineConfig;

const JUDGE_SYSTEM: &str = r#"You are a strict cinematography quality judge for AI-generated stills.
You receive one image plus the intended prompt and continuity context.
Score each criterion from 0.0 to 1.0 (1.0 = excellent).
Respond with ONLY valid JSON matching this schema (no markdown fences):
{
  "visual_quality": 0.0,
  "continuity": 0.0,
  "artifacts": 0.0,
  "prompt_fidelity": 0.0,
  "issues": ["short issue", ...],
  "retry_hints": ["actionable adjustment for next generation", ...],
  "summary": "one or two sentences"
}
Rules:
- artifacts: 1.0 means clean (few/no artifacts); 0.0 means ruined by artifacts.
- continuity: if no prior continuity context, score 0.75 if the image is self-consistent.
- Be concrete in issues and retry_hints (e.g. "increase subject scale", "darker background", "fix warped hands").
"#;

/// Outcome of the gate for one media file (possibly after retries are decided by caller).
#[derive(Debug, Clone)]
pub struct GateOutcome {
    pub verdict: QualityVerdict,
    /// True if VL was skipped (dry-run / vision offline).
    pub skipped: bool,
    pub skip_reason: Option<String>,
}

fn clamp01(v: f64) -> f64 {
    if v.is_nan() {
        0.0
    } else {
        v.clamp(0.0, 1.0)
    }
}

fn f64_field(obj: &Value, keys: &[&str], default: f64) -> f64 {
    for k in keys {
        if let Some(n) = obj.get(*k).and_then(|v| v.as_f64()) {
            return clamp01(n);
        }
        if let Some(n) = obj.get(*k).and_then(|v| v.as_i64()) {
            return clamp01(n as f64);
        }
    }
    default
}

fn string_list(obj: &Value, key: &str) -> Vec<String> {
    obj.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.trim().to_string()))
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Parse a VL model JSON blob into a [`QualityVerdict`].
pub fn parse_verdict_json(value: &Value, threshold: f64) -> Result<QualityVerdict, String> {
    let obj = value
        .as_object()
        .ok_or_else(|| "judge response is not a JSON object".to_string())?;

    let scores = QualityScores {
        visual_quality: f64_field(value, &["visual_quality", "visualQuality", "quality"], 0.5),
        continuity: f64_field(value, &["continuity"], 0.5),
        artifacts: f64_field(value, &["artifacts", "artifact_score"], 0.5),
        prompt_fidelity: f64_field(
            value,
            &["prompt_fidelity", "promptFidelity", "fidelity"],
            0.5,
        ),
    };

    let issues = string_list(value, "issues");
    let retry_hints = string_list(value, "retry_hints");
    let retry_hints = if retry_hints.is_empty() {
        string_list(value, "retryHints")
    } else {
        retry_hints
    };
    let summary = obj
        .get("summary")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    Ok(QualityVerdict::from_scores(
        scores,
        threshold,
        issues,
        retry_hints,
        summary,
    ))
}

fn skipped_accept(reason: &str, media_path: Option<&Path>, model: Option<&str>) -> GateOutcome {
    let mut v = QualityVerdict::from_scores(
        QualityScores {
            visual_quality: 1.0,
            continuity: 1.0,
            artifacts: 1.0,
            prompt_fidelity: 1.0,
        },
        0.0,
        vec![],
        vec![],
        reason.to_string(),
    );
    v.accept = true;
    v.judge_model = model.map(|s| s.to_string());
    v.media_path = media_path.map(|p| p.display().to_string());
    GateOutcome {
        verdict: v,
        skipped: true,
        skip_reason: Some(reason.to_string()),
    }
}

/// Run the VL judge on a local image file.
pub async fn judge_media(
    config: &EngineConfig,
    media_path: &Path,
    prompt: &str,
    continuity: &str,
) -> Result<GateOutcome, String> {
    if config.dry_run {
        return Ok(skipped_accept(
            "quality gate skipped (SLATE_DRY_RUN)",
            Some(media_path),
            None,
        ));
    }

    // Marker dry-run files are text — skip VL.
    let is_txt = media_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("txt"))
        .unwrap_or(false);
    if is_txt {
        return Ok(skipped_accept(
            "quality gate skipped (non-image take)",
            Some(media_path),
            None,
        ));
    }

    let judge_path = match crate::media::media_for_judge(media_path) {
        Ok(p) => p,
        Err(e) => {
            return Ok(skipped_accept(
                &format!("quality gate skipped (frame extract: {e})"),
                Some(media_path),
                None,
            ));
        }
    };
    let media_path = judge_path.as_path();

    let gate = config.quality_gate();
    let status = judge_vision_status(
        Some(config.judge_endpoint.as_str()),
        Some(config.judge_model.as_str()),
    )
    .await;

    if !status.ready {
        let hint = status
            .hint
            .unwrap_or_else(|| "vision judge not ready".into());
        return Ok(skipped_accept(
            &format!("quality gate skipped: {hint}"),
            Some(media_path),
            None,
        ));
    }

    let model = status
        .model
        .clone()
        .unwrap_or_else(|| DEFAULT_JUDGE_MODEL.to_string());
    let endpoint = status
        .endpoint
        .clone()
        .unwrap_or_else(|| config.judge_endpoint.clone());

    let continuity = if continuity.trim().is_empty() {
        "(none provided — score self-consistency only)"
    } else {
        continuity
    };

    let user_prompt = format!(
        "PROMPT INTENT:\n{prompt}\n\nCONTINUITY CONTEXT:\n{continuity}\n\n\
         Score the attached image. Return JSON only."
    );

    let req = BrainRequest {
        id: format!("judge-{}", uuid_simple()),
        task: "quality-gate".into(),
        system: JUDGE_SYSTEM.into(),
        prompt: user_prompt,
        images: vec![media_path.to_path_buf()],
        tier: BrainTier::Standard,
        expect_json: true,
        local_endpoint: Some(endpoint),
        local_model: Some(model.clone()),
    };

    let result = brain_run(req, BrainBackend::Local).await;
    if !result.ok {
        // Soft-skip: keep the take rather than failing the whole generate.
        return Ok(skipped_accept(
            &format!(
                "quality gate skipped (judge brain error: {})",
                result
                    .error
                    .unwrap_or_else(|| "vision judge brain call failed".into())
            ),
            Some(media_path),
            Some(&model),
        ));
    }

    let value = if let Some(j) = result.json {
        j
    } else {
        match extract_json(&result.text) {
            Ok(j) => j,
            Err(e) => {
                return Ok(skipped_accept(
                    &format!("quality gate skipped (judge JSON: {e})"),
                    Some(media_path),
                    Some(&model),
                ));
            }
        }
    };

    // Models sometimes return a JSON string or wrap scores one level deep.
    let value = if let Some(s) = value.as_str() {
        extract_json(s).unwrap_or(value)
    } else if value.get("visual_quality").is_none() && value.get("scores").is_some() {
        value.get("scores").cloned().unwrap_or(value)
    } else {
        value
    };

    let mut verdict = match parse_verdict_json(&value, gate.pass_threshold) {
        Ok(v) => v,
        Err(e) => {
            return Ok(skipped_accept(
                &format!("quality gate skipped (verdict schema: {e})"),
                Some(media_path),
                Some(&model),
            ));
        }
    };
    verdict.judge_model = Some(model);
    verdict.media_path = Some(media_path.display().to_string());

    Ok(GateOutcome {
        verdict,
        skipped: false,
        skip_reason: None,
    })
}

/// Format a compact notes line for the take record.
pub fn format_take_notes(path: &Path, outcome: &GateOutcome) -> String {
    let v = &outcome.verdict;
    let status = if outcome.skipped {
        "skip"
    } else if v.accept {
        "pass"
    } else {
        "fail"
    };
    let issues = if v.issues.is_empty() {
        String::new()
    } else {
        format!(" | issues: {}", v.issues.join("; "))
    };
    format!(
        "{} | quality:{status} overall={:.2} vq={:.2} cont={:.2} art={:.2} fid={:.2}{} | {}",
        path.display(),
        v.overall,
        v.scores.visual_quality,
        v.scores.continuity,
        v.scores.artifacts,
        v.scores.prompt_fidelity,
        issues,
        v.summary.replace('\n', " ")
    )
}

pub fn rating_for_verdict(v: &QualityVerdict, skipped: bool) -> slate_domain::TakeRating {
    use slate_domain::TakeRating;
    if skipped {
        return TakeRating::Good;
    }
    if v.accept {
        if v.overall >= 0.85 {
            TakeRating::Circled
        } else {
            TakeRating::Good
        }
    } else {
        TakeRating::NoGood
    }
}

/// Merge retry hints into the positive prompt for a re-roll (bounded length).
pub fn apply_retry_hints_to_prompt(prompt: &str, hints: &[String]) -> String {
    if hints.is_empty() {
        return prompt.to_string();
    }
    let joined = hints
        .iter()
        .take(4)
        .map(|h| h.trim())
        .filter(|h| !h.is_empty())
        .collect::<Vec<_>>()
        .join("; ");
    if joined.is_empty() {
        return prompt.to_string();
    }
    format!("{prompt}\n\n# Quality pickup\n{joined}\n")
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("{t:x}")
}

/// Expose gate config defaults for tests / tools.
pub fn gate_config_from_engine(config: &EngineConfig) -> QualityGateConfig {
    config.quality_gate()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_verdict_accepts_snake_and_camel() {
        let v = json!({
            "visual_quality": 0.9,
            "continuity": 0.8,
            "artifacts": 0.85,
            "promptFidelity": 0.7,
            "issues": ["slight haze"],
            "retryHints": ["increase contrast"],
            "summary": "usable night plate"
        });
        let verdict = parse_verdict_json(&v, 0.7).unwrap();
        assert!(verdict.accept);
        assert!((verdict.scores.prompt_fidelity - 0.7).abs() < 1e-9);
        assert_eq!(verdict.retry_hints.len(), 1);
    }

    #[test]
    fn parse_verdict_rejects_low_mean() {
        let v = json!({
            "visual_quality": 0.2,
            "continuity": 0.2,
            "artifacts": 0.2,
            "prompt_fidelity": 0.2,
            "issues": ["wrecked"],
            "retry_hints": ["start over"],
            "summary": "no"
        });
        let verdict = parse_verdict_json(&v, 0.7).unwrap();
        assert!(!verdict.accept);
    }

    #[test]
    fn apply_hints_appends_section() {
        let out = apply_retry_hints_to_prompt("# Subject\nhero\n", &["push in tighter".into()]);
        assert!(out.contains("Quality pickup"));
        assert!(out.contains("push in tighter"));
    }
}
