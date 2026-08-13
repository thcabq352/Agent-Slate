//! Grok Build CLI adapter — `grok -p` with OAuth from `grok login`.
//! Preferred over Cursor for grok-4.5 / grok-4.6. Never spawn the colliding `agent` binary.

use crate::cli::{brain_path, resolve_grok_launch, which_cli, CLI_TIMEOUT};
use crate::extract_json::extract_json;
use crate::types::{BrainBackend, BrainRequest, BrainResult};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;

pub const GROK_PROMPT_FILE: &str = "slate-brain-prompt.txt";

/// Probe official Grok Build CLI (`grok --version`).
pub async fn which_grok() -> Option<String> {
    which_cli("grok", &["--version"]).await
}

pub fn grok_build_cli_model(backend: BrainBackend) -> &'static str {
    match backend {
        BrainBackend::Grok45 => "grok-4.5",
        BrainBackend::Grok46 => "grok-4.6",
        BrainBackend::Cursor | BrainBackend::Codex | BrainBackend::Local => "grok-build",
    }
}

pub fn grok_workspace_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("slate-grok-brain");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn grok_auth_path() -> Option<PathBuf> {
    let home = std::env::var("USERPROFILE")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("HOME").ok().filter(|s| !s.is_empty()))?;
    Some(PathBuf::from(home).join(".grok").join("auth.json"))
}

/// True when `~/.grok/auth.json` looks like a `grok login` session. Never returns tokens.
pub fn grok_build_oauth_present() -> bool {
    let Some(path) = grok_auth_path() else {
        return false;
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return false;
    };
    grok_auth_looks_signed_in(&v)
}

pub fn grok_auth_looks_signed_in(v: &Value) -> bool {
    if v.get("https://accounts.x.ai/sign-in")
        .and_then(|e| e.get("key"))
        .and_then(|k| k.as_str())
        .is_some_and(|s| !s.is_empty())
    {
        return true;
    }
    if v.get("tokens")
        .and_then(|t| t.get("access_token"))
        .and_then(|t| t.as_str())
        .is_some_and(|s| !s.is_empty())
    {
        return true;
    }
    let Some(obj) = v.as_object() else {
        return false;
    };
    for val in obj.values() {
        let Some(entry) = val.as_object() else {
            continue;
        };
        let key = entry.get("key").and_then(|k| k.as_str()).unwrap_or("");
        if key.is_empty() {
            continue;
        }
        let issuer = entry
            .get("oidc_issuer")
            .or_else(|| entry.get("issuer"))
            .and_then(|i| i.as_str())
            .unwrap_or("");
        if issuer.contains("x.ai") || entry.contains_key("refresh_token") {
            return true;
        }
    }
    false
}

pub fn grok_build_ready() -> bool {
    resolve_grok_launch().is_some() && grok_build_oauth_present()
}

pub fn grok_build_headless_prompt() -> String {
    format!("Read {GROK_PROMPT_FILE} in this directory and follow it exactly. Reply with only the answer — no preamble.")
}

/// `grok -p … --output-format json --always-approve --cwd <scratch> -m <model>`
pub fn build_grok_build_args(workspace: &Path, backend: BrainBackend) -> Vec<String> {
    vec![
        "-p".into(),
        grok_build_headless_prompt(),
        "--output-format".into(),
        "json".into(),
        "--always-approve".into(),
        "--cwd".into(),
        workspace.to_string_lossy().into_owned(),
        "-m".into(),
        grok_build_cli_model(backend).into(),
        "--tools".into(),
        "read_file".into(),
        "--max-turns".into(),
        "8".into(),
    ]
}

fn is_auth_error(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    lower.contains("authenticat")
        || lower.contains("oauth")
        || lower.contains("401")
        || lower.contains("revoked")
        || lower.contains("not logged")
        || lower.contains("not signed")
        || lower.contains("unauthenticated")
        || lower.contains("unauthorized")
        || lower.contains("grok login")
        || lower.contains("login")
}

fn auth_err(detail: &str) -> String {
    format!(
        "Grok Build is not signed in. Open a terminal, run: grok login  — approve xAI OAuth, then retry. ({detail})"
    )
}

/// Parse Grok Build `--output-format json` stdout: prefer `.text`.
pub fn parse_grok_build_output(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    match serde_json::from_str::<Value>(trimmed) {
        Ok(v) => {
            let err_str = v.get("error").and_then(|e| e.as_str()).unwrap_or("");
            let is_error = v.get("is_error").and_then(|b| b.as_bool()).unwrap_or(false);
            if is_error || !err_str.is_empty() {
                let msg = if !err_str.is_empty() {
                    err_str
                } else {
                    v.get("result")
                        .and_then(|r| r.as_str())
                        .unwrap_or("Grok Build returned an error.")
                };
                if is_auth_error(msg) {
                    return Err(auth_err(msg));
                }
                return Err(msg.to_string());
            }
            if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                return Ok(text.to_string());
            }
            if let Some(result) = v.get("result").and_then(|r| r.as_str()) {
                return Ok(result.to_string());
            }
            if let Some(message) = v.get("message").and_then(|r| r.as_str()) {
                return Ok(message.to_string());
            }
            Ok(trimmed.to_string())
        }
        Err(_) => {
            if is_auth_error(trimmed) {
                return Err(auth_err(trimmed));
            }
            Ok(trimmed.to_string())
        }
    }
}

fn write_prompt_file(req: &BrainRequest, workspace: &Path) -> Result<(), String> {
    let mut prompt = format!("{}\n\n---\n\n{}", req.system, req.prompt);
    if !req.images.is_empty() {
        prompt.push_str(
            "\n\nReference media frames to view (read each file before answering):\n",
        );
        let lines: Vec<String> = req
            .images
            .iter()
            .map(|p| format!("- {}", p.display()))
            .collect();
        prompt.push_str(&lines.join("\n"));
    }
    std::fs::write(workspace.join(GROK_PROMPT_FILE), prompt)
        .map_err(|e| format!("Grok Build prompt file: {e}"))
}

async fn run_grok_once(req: &BrainRequest, backend: BrainBackend, extra_nudge: Option<&str>) -> Result<String, String> {
    let workspace = grok_workspace_dir();
    write_prompt_file(req, &workspace)?;
    if let Some(nudge) = extra_nudge {
        let path = workspace.join(GROK_PROMPT_FILE);
        let mut body = std::fs::read_to_string(&path).unwrap_or_default();
        body.push_str("\n\n");
        body.push_str(nudge);
        let _ = std::fs::write(path, body);
    }
    let cmd = resolve_grok_launch().ok_or_else(|| {
        "Grok Build CLI not found. Install it, run grok login, then retry.".to_string()
    })?;
    let args = build_grok_build_args(&workspace, backend);
    let child = Command::new(&cmd)
        .args(&args)
        .env("PATH", brain_path())
        .current_dir(&workspace)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Could not launch grok ({cmd}): {e}"))?;

    let output = tokio::time::timeout(CLI_TIMEOUT, child.wait_with_output())
        .await
        .map_err(|_| "Grok Build timed out after 600s".to_string())?
        .map_err(|e| format!("grok wait: {e}"))?;

    let out = String::from_utf8_lossy(&output.stdout).into_owned();
    let err_out = String::from_utf8_lossy(&output.stderr).into_owned();

    if !output.status.success() && out.trim().is_empty() {
        let msg = if err_out.trim().is_empty() {
            format!("grok exited with code {:?}", output.status.code())
        } else {
            err_out.trim().to_string()
        };
        if is_auth_error(&msg) {
            return Err(auth_err(&msg));
        }
        return Err(msg);
    }

    parse_grok_build_output(&out)
}

/// Run a grok-4.5 / grok-4.6 request via official Grok Build OAuth.
pub async fn run_grok_build(req: &BrainRequest, backend: BrainBackend) -> BrainResult {
    let started = Instant::now();
    let elapsed = || started.elapsed().as_millis() as u64;

    match run_grok_once(req, backend, None).await {
        Ok(mut text) => {
            let mut json = None;
            if req.expect_json {
                match extract_json(&text) {
                    Ok(v) => json = Some(v),
                    Err(_) => {
                        match run_grok_once(
                            req,
                            backend,
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
