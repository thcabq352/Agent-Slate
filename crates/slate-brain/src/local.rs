//! Local OpenAI-compatible brain adapter (Ollama, LM Studio, vLLM, llama.cpp…).
//! Port of `src/main/brain.ts` local path: `/v1/models` + `/v1/chat/completions`.

use crate::extract_json::extract_json;
use crate::types::{BrainRequest, BrainResult};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde_json::{json, Value};
use std::path::Path;
use std::time::{Duration, Instant};

/// Common localhost OpenAI-compat endpoints (same order as TS).
const LOCAL_CANDIDATES: &[&str] = &[
    "http://localhost:11434/v1", // Ollama
    "http://localhost:1234/v1",  // LM Studio
    "http://localhost:8000/v1",  // vLLM
    "http://localhost:8080/v1",  // llama.cpp / KoboldCpp
];

const PROBE_TIMEOUT: Duration = Duration::from_millis(1500);

/// Normalize a user or default endpoint to `http(s)://…/v1` (no trailing slash).
pub fn normalize_endpoint(url: &str) -> String {
    let mut u = url.trim().trim_end_matches('/').to_string();
    if !u.starts_with("http://") && !u.starts_with("https://") {
        u = format!("http://{u}");
    }
    if !u.ends_with("/v1") {
        u = format!("{u}/v1");
    }
    u
}

/// Parse OpenAI chat completion JSON body → trimmed assistant content.
pub fn parse_chat_response(body: &str) -> Result<String, String> {
    let value: Value =
        serde_json::from_str(body).map_err(|e| format!("invalid chat completion JSON: {e}"))?;

    if let Some(msg) = value
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
    {
        return Err(msg.to_string());
    }

    let text = value
        .pointer("/choices/0/message/content")
        .and_then(|c| c.as_str())
        .ok_or_else(|| "Local model returned an empty response.".to_string())?;

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("Local model returned an empty response.".into());
    }
    Ok(trimmed.to_string())
}

fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(|e| e.to_string())
}

async fn probe_local(client: &reqwest::Client, endpoint: &str) -> Option<Vec<String>> {
    let url = format!("{endpoint}/models");
    let res = client
        .get(&url)
        .header("Authorization", "Bearer slate")
        .timeout(PROBE_TIMEOUT)
        .send()
        .await
        .ok()?;
    if !res.status().is_success() {
        return None;
    }
    let body: Value = res.json().await.ok()?;
    let models = body
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("id").and_then(|id| id.as_str()).map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Some(models)
}

/// Find a live local server: preferred endpoint first, else common ports.
/// Returns `(endpoint, model_ids)`.
pub async fn detect_local(preferred: Option<&str>) -> (Option<String>, Vec<String>) {
    let client = match http_client() {
        Ok(c) => c,
        Err(_) => return (None, vec![]),
    };

    let candidates: Vec<String> = if let Some(p) = preferred {
        vec![normalize_endpoint(p)]
    } else {
        LOCAL_CANDIDATES
            .iter()
            .map(|s| normalize_endpoint(s))
            .collect()
    };

    for ep in candidates {
        if let Some(models) = probe_local(&client, &ep).await {
            return (Some(ep), models);
        }
    }
    (None, vec![])
}

fn image_mime(path: &Path) -> Option<&'static str> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => Some("image/png"),
        Some("jpg") | Some("jpeg") => Some("image/jpeg"),
        Some("webp") => Some("image/webp"),
        Some("gif") => Some("image/gif"),
        _ => None,
    }
}

fn local_messages(req: &BrainRequest) -> Value {
    let user_content: Value = if req.images.is_empty() {
        Value::String(req.prompt.clone())
    } else {
        let mut parts = vec![json!({"type": "text", "text": req.prompt})];
        for img in &req.images {
            let Some(mime) = image_mime(img) else {
                continue;
            };
            let Ok(bytes) = std::fs::read(img) else {
                continue;
            };
            let b64 = B64.encode(bytes);
            parts.push(json!({
                "type": "image_url",
                "image_url": { "url": format!("data:{mime};base64,{b64}") }
            }));
        }
        Value::Array(parts)
    };

    json!([
        { "role": "system", "content": req.system },
        { "role": "user", "content": user_content }
    ])
}

async fn run_local_once(
    client: &reqwest::Client,
    req: &BrainRequest,
    endpoint: &str,
    model: &str,
    extra_nudge: Option<&str>,
) -> Result<String, String> {
    let mut messages = local_messages(req);
    if let Some(nudge) = extra_nudge {
        if let Some(arr) = messages.as_array_mut() {
            arr.push(json!({"role": "user", "content": nudge}));
        }
    }

    let url = format!("{endpoint}/chat/completions");
    let res = client
        .post(&url)
        .header("Authorization", "Bearer slate")
        .header("Content-Type", "application/json")
        .json(&json!({
            "model": model,
            "messages": messages,
            "stream": false
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        let status = res.status();
        let detail = res.text().await.unwrap_or_default();
        let detail: String = detail.chars().take(300).collect();
        return Err(format!(
            "Local model server responded {status} at {endpoint}. {detail}"
        ));
    }

    let body = res.text().await.map_err(|e| e.to_string())?;
    parse_chat_response(&body)
}

/// Run a single brain request against a local OpenAI-compatible server.
pub async fn run_local(req: &BrainRequest) -> BrainResult {
    let started = Instant::now();
    let elapsed = || started.elapsed().as_millis() as u64;

    let (endpoint, models) = detect_local(req.local_endpoint.as_deref()).await;
    let Some(endpoint) = endpoint else {
        return BrainResult {
            id: req.id.clone(),
            ok: false,
            text: String::new(),
            json: None,
            error: Some(
                "No local model server found. Start Ollama, LM Studio, vLLM, or llama.cpp (or set a custom endpoint in Project Settings → Brain), then retry."
                    .into(),
            ),
            elapsed_ms: elapsed(),
        };
    };

    let model = req
        .local_model
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| models.first().cloned());

    let Some(model) = model else {
        return BrainResult {
            id: req.id.clone(),
            ok: false,
            text: String::new(),
            json: None,
            error: Some(format!(
                "Local server at {endpoint} has no models loaded. Pull or load a model, then retry."
            )),
            elapsed_ms: elapsed(),
        };
    };

    let client = match http_client() {
        Ok(c) => c,
        Err(e) => {
            return BrainResult {
                id: req.id.clone(),
                ok: false,
                text: String::new(),
                json: None,
                error: Some(e),
                elapsed_ms: elapsed(),
            };
        }
    };

    match run_local_once(&client, req, &endpoint, &model, None).await {
        Ok(mut text) => {
            let mut json = None;
            if req.expect_json {
                match extract_json(&text) {
                    Ok(v) => json = Some(v),
                    Err(_) => {
                        match run_local_once(
                            &client,
                            req,
                            &endpoint,
                            &model,
                            Some(
                                "IMPORTANT: Respond with ONLY the requested JSON. No prose, no code fences.",
                            ),
                        )
                        .await
                        {
                            Ok(retry_text) => {
                                text = retry_text;
                                match extract_json(&text) {
                                    Ok(v) => json = Some(v),
                                    Err(e) => {
                                        return BrainResult {
                                            id: req.id.clone(),
                                            ok: false,
                                            text,
                                            json: None,
                                            error: Some(e),
                                            elapsed_ms: elapsed(),
                                        };
                                    }
                                }
                            }
                            Err(e) => {
                                return BrainResult {
                                    id: req.id.clone(),
                                    ok: false,
                                    text: String::new(),
                                    json: None,
                                    error: Some(e),
                                    elapsed_ms: elapsed(),
                                };
                            }
                        }
                    }
                }
            }
            BrainResult {
                id: req.id.clone(),
                ok: true,
                text,
                json,
                error: None,
                elapsed_ms: elapsed(),
            }
        }
        Err(e) => BrainResult {
            id: req.id.clone(),
            ok: false,
            text: String::new(),
            json: None,
            error: Some(e),
            elapsed_ms: elapsed(),
        },
    }
}
