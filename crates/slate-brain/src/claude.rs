//! Claude Code CLI adapter — print mode with JSON output.
//! Port of `src/main/brain.ts` `buildClaudeCall` / `parseClaudeOutput` / `which`.

use crate::extract_json::extract_json;
use crate::types::{BrainRequest, BrainResult, BrainTier};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// Factory / agent call timeout (matches TS + design ~600s).
pub const CLI_TIMEOUT: Duration = Duration::from_secs(600);
const WHICH_TIMEOUT: Duration = Duration::from_secs(15);

/// ChatGPT desktop bundled codex (macOS) — shared with codex module.
#[cfg(target_os = "macos")]
pub(crate) const CODEX_BUNDLED: &str =
    "/Applications/ChatGPT.app/Contents/Resources/codex";

/// Extra dirs prepended/appended to PATH when spawning CLIs (Electron-style minimal PATH).
pub(crate) fn cli_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
    ];
    if let Some(home) = home_dir() {
        dirs.push(home.join(".local").join("bin"));
        dirs.push(home.join("bin"));
        dirs.push(home.join(".npm-global").join("bin"));
    }
    // Windows npm global: %APPDATA%\npm
    #[cfg(windows)]
    if let Ok(appdata) = std::env::var("APPDATA") {
        dirs.push(PathBuf::from(appdata).join("npm"));
    }
    dirs.push(PathBuf::from("/usr/bin"));
    dirs
}

fn home_dir() -> Option<PathBuf> {
    if let Ok(h) = std::env::var("USERPROFILE") {
        if !h.is_empty() {
            return Some(PathBuf::from(h));
        }
    }
    if let Ok(h) = std::env::var("HOME") {
        if !h.is_empty() {
            return Some(PathBuf::from(h));
        }
    }
    None
}

/// Candidate executable names for `name` on this platform.
pub(crate) fn cli_candidates(name: &str) -> Vec<String> {
    #[cfg(windows)]
    {
        vec![
            format!("{name}.cmd"),
            format!("{name}.exe"),
            name.to_string(),
        ]
    }
    #[cfg(not(windows))]
    {
        vec![name.to_string()]
    }
}

/// Resolve absolute path to a CLI, or bare name if not found (PATH fallback).
pub(crate) fn resolve_cli(name: &str) -> String {
    #[cfg(target_os = "macos")]
    if name == "codex" {
        let bundled = PathBuf::from(CODEX_BUNDLED);
        if bundled.is_file() {
            return CODEX_BUNDLED.to_string();
        }
    }

    let candidates = cli_candidates(name);
    for dir in cli_dirs() {
        for cand in &candidates {
            let p = dir.join(cand);
            if p.is_file() {
                return p.to_string_lossy().into_owned();
            }
        }
    }

    // Also scan PATH entries for Windows .cmd / bare name.
    if let Ok(path) = std::env::var("PATH") {
        for entry in std::env::split_paths(&path) {
            for cand in &candidates {
                let p = entry.join(cand);
                if p.is_file() {
                    return p.to_string_lossy().into_owned();
                }
            }
        }
    }

    // Hope the OS can find it via PATH as-is.
    #[cfg(windows)]
    {
        // Prefer .cmd when spawning by bare name on Windows (npm shims).
        format!("{name}.cmd")
    }
    #[cfg(not(windows))]
    {
        name.to_string()
    }
}

/// PATH with CLI_DIRS appended so child processes inherit Homebrew / user bins.
pub(crate) fn brain_path() -> String {
    let mut parts: Vec<String> = std::env::var_os("PATH")
        .map(|p| {
            std::env::split_paths(&p)
                .map(|x| x.to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    for d in cli_dirs() {
        let s = d.to_string_lossy().into_owned();
        if !parts.iter().any(|p| p == &s) {
            parts.push(s);
        }
    }
    std::env::join_paths(parts.iter().map(PathBuf::from))
        .map(|os| os.to_string_lossy().into_owned())
        .unwrap_or_else(|_| parts.join(if cfg!(windows) { ";" } else { ":" }))
}

/// Run `cmd --version` (or given args) and return first stdout line, or None.
pub(crate) async fn which_cli(name: &str, args: &[&str]) -> Option<String> {
    let resolved = resolve_cli(name);
    let mut command = Command::new(&resolved);
    command
        .args(args)
        .env("PATH", brain_path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    // On Windows, running .cmd requires cmd.exe unless CREATE_NO_WINDOW; tokio handles .cmd.

    let fut = async {
        let output = command.output().await.ok()?;
        if !output.status.success() && output.stdout.is_empty() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let first = stdout.lines().next().unwrap_or("").trim();
        if first.is_empty() {
            Some("available".to_string())
        } else {
            Some(first.to_string())
        }
    };

    match tokio::time::timeout(WHICH_TIMEOUT, fut).await {
        Ok(v) => v,
        Err(_) => None,
    }
}

/// Probe Claude Code CLI availability / version.
pub async fn which_claude() -> Option<String> {
    which_cli("claude", &["--version"]).await
}

/// Map tier → Claude model alias (null/top uses user default).
fn claude_tier_model(tier: BrainTier) -> Option<&'static str> {
    match tier {
        BrainTier::Fast => Some("haiku"),
        BrainTier::Standard => Some("sonnet"),
        BrainTier::Top => None,
    }
}

/// Build Claude Code argv + stdin prompt. Includes `-p`, `--output-format json`,
/// optional `--model haiku|sonnet`, `--append-system-prompt`.
pub fn build_claude_args(req: &BrainRequest) -> (Vec<String>, String) {
    let mut args = vec!["-p".into(), "--output-format".into(), "json".into()];
    if let Some(model) = claude_tier_model(req.tier) {
        args.push("--model".into());
        args.push(model.into());
    }
    if !req.images.is_empty() {
        // Pre-approve Read so the model can open reference frames without a prompt.
        args.push("--allowedTools".into());
        args.push("Read".into());
    }
    args.push("--append-system-prompt".into());
    args.push(req.system.clone());

    let mut prompt = req.prompt.clone();
    if !req.images.is_empty() {
        prompt.push_str(
            "\n\nReference media frames to view (use the Read tool on each before answering):\n",
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
    // authenticat|oauth|401|logged? ?in|revoked (case-insensitive)
    let lower = msg.to_ascii_lowercase();
    lower.contains("authenticat")
        || lower.contains("oauth")
        || lower.contains("401")
        || lower.contains("revoked")
        || lower.contains("logged in")
        || lower.contains("logedin") // logged?in without space edge
        || {
            // logged? ?in — "login" / "logged in" / "log in"
            lower.contains("login") || lower.contains("log in") || lower.contains("loggedin")
        }
}

/// Parse Claude Code `--output-format json` stdout: read `.result`, surface auth errors.
pub fn parse_claude_output(raw: &str) -> Result<String, String> {
    let parsed: Result<Value, _> = serde_json::from_str(raw);
    match parsed {
        Ok(v) => {
            if v.get("is_error").and_then(|b| b.as_bool()).unwrap_or(false) {
                let msg = v
                    .get("result")
                    .and_then(|r| r.as_str())
                    .unwrap_or("Claude Code returned an error.");
                if is_auth_error(msg) {
                    return Err(format!(
                        "Claude Code's sign-in has expired or been revoked. Open Terminal, run: claude auth login  — approve in the browser, then retry. ({msg})"
                    ));
                }
                return Err(msg.to_string());
            }
            if let Some(result) = v.get("result").and_then(|r| r.as_str()) {
                return Ok(result.to_string());
            }
            Ok(raw.trim().to_string())
        }
        Err(_) => Ok(raw.trim().to_string()),
    }
}

async fn run_claude_once(req: &BrainRequest, extra_nudge: Option<&str>) -> Result<String, String> {
    let (args, input) = build_claude_args(req);
    let input = match extra_nudge {
        Some(n) => format!("{input}\n\n{n}"),
        None => input,
    };
    let cmd = resolve_cli("claude");
    let mut child = Command::new(&cmd)
        .args(&args)
        .env("PATH", brain_path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Could not launch claude: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .await
            .map_err(|e| format!("claude stdin: {e}"))?;
        drop(stdin);
    }

    let output = tokio::time::timeout(CLI_TIMEOUT, child.wait_with_output())
        .await
        .map_err(|_| "claude timed out after 600s".to_string())?
        .map_err(|e| format!("claude wait: {e}"))?;

    let out = String::from_utf8_lossy(&output.stdout).into_owned();
    let err_out = String::from_utf8_lossy(&output.stderr).into_owned();

    if !output.status.success() && out.trim().is_empty() {
        return Err(if err_out.trim().is_empty() {
            format!("claude exited with code {:?}", output.status.code())
        } else {
            err_out.trim().to_string()
        });
    }

    parse_claude_output(&out)
}

/// Run a brain request via Claude Code CLI.
pub async fn run_claude(req: &BrainRequest) -> BrainResult {
    let started = Instant::now();
    let elapsed = || started.elapsed().as_millis() as u64;

    match run_claude_once(req, None).await {
        Ok(mut text) => {
            let mut json = None;
            if req.expect_json {
                match extract_json(&text) {
                    Ok(v) => json = Some(v),
                    Err(_) => {
                        match run_claude_once(
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
