//! Cursor CLI adapter — print mode with JSON output, OAuth via `cursor-agent login`.
//! Uses `cursor-agent` (not `agent`) to avoid Grok CLI's colliding `agent` binary.
//! Composer always uses this path. Grok 4.5 / 4.6 use it only when Grok Build is not ready.

use crate::cli::{brain_path, resolve_cursor_launch, which_cli, CLI_TIMEOUT};
use crate::extract_json::extract_json;
use crate::types::{BrainBackend, BrainRequest, BrainResult, BrainTier};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// Probe Cursor Agent CLI availability / version.
pub async fn which_cursor() -> Option<String> {
    which_cli("cursor-agent", &["--version"]).await
}

/// Cursor CLI `--model` slugs. Never pass Claude ids (watermarked generations).
pub fn cursor_cli_model(backend: BrainBackend, tier: BrainTier) -> &'static str {
    match backend {
        BrainBackend::Grok45 => match tier {
            BrainTier::Fast => "cursor-grok-4.5-high-fast",
            BrainTier::Standard | BrainTier::Top => "cursor-grok-4.5-high",
        },
        BrainBackend::Grok46 => match tier {
            BrainTier::Fast => "cursor-grok-4.6-xhigh-fast",
            BrainTier::Standard => "cursor-grok-4.6-high",
            BrainTier::Top => "cursor-grok-4.6-xhigh",
        },
        BrainBackend::Cursor | BrainBackend::Codex | BrainBackend::Local => match tier {
            BrainTier::Fast => "composer-2.5-fast",
            BrainTier::Standard | BrainTier::Top => "composer-2.5",
        },
    }
}

/// Scratch workspace so print-mode does not operate on the Agent-Slate repo.
pub fn cursor_workspace_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("slate-cursor-brain");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Build Cursor CLI argv + stdin prompt.
/// `-p --output-format json --mode ask --trust --workspace <dir> --model <slug>`
pub fn build_cursor_args(
    req: &BrainRequest,
    workspace: &Path,
    backend: BrainBackend,
) -> (Vec<String>, String) {
    let args = vec![
        "-p".into(),
        "--output-format".into(),
        "json".into(),
        "--mode".into(),
        "ask".into(),
        "--trust".into(),
        "--workspace".into(),
        workspace.to_string_lossy().into_owned(),
        "--model".into(),
        cursor_cli_model(backend, req.tier).into(),
    ];

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
    (args, prompt)
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
        || lower.contains("login")
        || lower.contains("log in")
        || lower.contains("logged in")
}

/// Parse Cursor CLI `--output-format json` stdout: read `.result`, surface auth errors.
pub fn parse_cursor_output(raw: &str) -> Result<String, String> {
    let parsed: Result<Value, _> = serde_json::from_str(raw.trim());
    match parsed {
        Ok(v) => {
            if v.get("is_error").and_then(|b| b.as_bool()).unwrap_or(false) {
                let msg = v
                    .get("result")
                    .and_then(|r| r.as_str())
                    .or_else(|| v.get("error").and_then(|r| r.as_str()))
                    .unwrap_or("Cursor CLI returned an error.");
                if is_auth_error(msg) {
                    return Err(format!(
                        "Cursor CLI is not signed in. Open a terminal, run: cursor-agent login  — approve in the browser (Cursor OAuth), then retry. ({msg})"
                    ));
                }
                return Err(msg.to_string());
            }
            if let Some(result) = v.get("result").and_then(|r| r.as_str()) {
                return Ok(result.to_string());
            }
            Ok(raw.trim().to_string())
        }
        Err(_) => {
            let trimmed = raw.trim();
            if is_auth_error(trimmed) {
                return Err(format!(
                    "Cursor CLI is not signed in. Open a terminal, run: cursor-agent login  — approve in the browser (Cursor OAuth), then retry. ({trimmed})"
                ));
            }
            Ok(trimmed.to_string())
        }
    }
}

async fn run_cursor_once(
    req: &BrainRequest,
    backend: BrainBackend,
    extra_nudge: Option<&str>,
) -> Result<String, String> {
    let workspace = cursor_workspace_dir();
    let (args, input) = build_cursor_args(req, &workspace, backend);
    let input = match extra_nudge {
        Some(n) => format!("{input}\n\n{n}"),
        None => input,
    };
    let (cmd, prefix) = resolve_cursor_launch();
    let mut argv = prefix;
    argv.extend(args);
    let mut child = Command::new(&cmd)
        .args(&argv)
        .env("PATH", brain_path())
        .env("CURSOR_INVOKED_AS", "cursor-agent")
        .current_dir(&workspace)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Could not launch cursor-agent ({cmd}): {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .await
            .map_err(|e| format!("cursor-agent stdin: {e}"))?;
        drop(stdin);
    }

    let output = tokio::time::timeout(CLI_TIMEOUT, child.wait_with_output())
        .await
        .map_err(|_| "cursor-agent timed out after 600s".to_string())?
        .map_err(|e| format!("cursor-agent wait: {e}"))?;

    let out = String::from_utf8_lossy(&output.stdout).into_owned();
    let err_out = String::from_utf8_lossy(&output.stderr).into_owned();

    if !output.status.success() && out.trim().is_empty() {
        let msg = if err_out.trim().is_empty() {
            format!("cursor-agent exited with code {:?}", output.status.code())
        } else {
            err_out.trim().to_string()
        };
        if is_auth_error(&msg) {
            return Err(format!(
                "Cursor CLI is not signed in. Open a terminal, run: cursor-agent login  — approve in the browser (Cursor OAuth), then retry. ({msg})"
            ));
        }
        return Err(msg);
    }

    parse_cursor_output(&out)
}

/// Run a brain request via Cursor CLI (`cursor-agent`).
pub async fn run_cursor(req: &BrainRequest, backend: BrainBackend) -> BrainResult {
    let started = Instant::now();
    let elapsed = || started.elapsed().as_millis() as u64;

    match run_cursor_once(req, backend, None).await {
        Ok(mut text) => {
            let mut json = None;
            if req.expect_json {
                match extract_json(&text) {
                    Ok(v) => json = Some(v),
                    Err(_) => {
                        match run_cursor_once(
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
