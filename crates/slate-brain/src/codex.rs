//! Codex CLI adapter — one-shot `codex exec` with `--output-last-message`.
//! Port of `src/main/brain.ts` `buildCodexCall` / codex result file handling.

use crate::cli::{brain_path, resolve_cli, which_cli, CLI_TIMEOUT};
use crate::extract_json::extract_json;
use crate::types::{BrainRequest, BrainResult};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// Probe Codex CLI availability / version.
pub async fn which_codex() -> Option<String> {
    which_cli("codex", &["--version"]).await
}

/// Build Codex argv + stdin prompt.
/// `exec --skip-git-repo-check --output-last-message <file> [-i img...] -`
pub fn build_codex_args(req: &BrainRequest, last_message_file: &Path) -> (Vec<String>, String) {
    let mut args = vec![
        "exec".into(),
        "--skip-git-repo-check".into(),
        "--output-last-message".into(),
        last_message_file.to_string_lossy().into_owned(),
    ];
    for img in &req.images {
        args.push("-i".into());
        args.push(img.to_string_lossy().into_owned());
    }
    args.push("-".into());
    let prompt = format!("{}\n\n---\n\n{}", req.system, req.prompt);
    (args, prompt)
}

fn last_message_path(req_id: &str) -> PathBuf {
    std::env::temp_dir().join(format!("slate-codex-{req_id}.txt"))
}

fn codex_result(last_message_file: &Path, raw_stdout: &str) -> String {
    if let Ok(msg) = std::fs::read_to_string(last_message_file) {
        let _ = std::fs::remove_file(last_message_file);
        let trimmed = msg.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    } else {
        let _ = std::fs::remove_file(last_message_file);
    }
    raw_stdout.trim().to_string()
}

async fn run_codex_once(req: &BrainRequest, extra_nudge: Option<&str>) -> Result<String, String> {
    let last_message_file = last_message_path(&req.id);
    // Ensure a clean slate for this run.
    let _ = std::fs::remove_file(&last_message_file);

    let (args, input) = build_codex_args(req, &last_message_file);
    let input = match extra_nudge {
        Some(n) => format!("{input}\n\n{n}"),
        None => input,
    };

    let cmd = resolve_cli("codex");
    let mut child = Command::new(&cmd)
        .args(&args)
        .env("PATH", brain_path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Could not launch codex: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .await
            .map_err(|e| format!("codex stdin: {e}"))?;
        drop(stdin);
    }

    let output = tokio::time::timeout(CLI_TIMEOUT, child.wait_with_output())
        .await
        .map_err(|_| {
            let _ = std::fs::remove_file(&last_message_file);
            "codex timed out after 600s".to_string()
        })?
        .map_err(|e| {
            let _ = std::fs::remove_file(&last_message_file);
            format!("codex wait: {e}")
        })?;

    let out = String::from_utf8_lossy(&output.stdout).into_owned();
    let err_out = String::from_utf8_lossy(&output.stderr).into_owned();

    if !output.status.success() && out.trim().is_empty() {
        // Still try last-message file before failing.
        let from_file = codex_result(&last_message_file, "");
        if !from_file.is_empty() {
            return Ok(from_file);
        }
        return Err(if err_out.trim().is_empty() {
            format!("codex exited with code {:?}", output.status.code())
        } else {
            err_out.trim().to_string()
        });
    }

    Ok(codex_result(&last_message_file, &out))
}

/// Run a brain request via Codex CLI.
pub async fn run_codex(req: &BrainRequest) -> BrainResult {
    let started = Instant::now();
    let elapsed = || started.elapsed().as_millis() as u64;

    match run_codex_once(req, None).await {
        Ok(mut text) => {
            let mut json = None;
            if req.expect_json {
                match extract_json(&text) {
                    Ok(v) => json = Some(v),
                    Err(_) => {
                        match run_codex_once(
                            req,
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
