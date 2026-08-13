//! ComfyUI HTTP client — health, queue, history poll, download, pack load, generate.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use rand::Rng;
use serde_json::{json, Value};

use crate::inject::inject_workflow;
use crate::manifest::{load_manifest, PackManifest};
use crate::{Error, Result};

/// Default ComfyUI API base URL (Video Buddy–aligned loopback).
pub const DEFAULT_COMFY_BASE: &str = "http://127.0.0.1:8188";

/// Env var: when set to `1`, skip HTTP and write a marker file instead.
pub const SLATE_DRY_RUN_ENV: &str = "SLATE_DRY_RUN";

/// Default history poll interval.
const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// HTTP client for a local (or remote) ComfyUI API server.
#[derive(Debug, Clone)]
pub struct ComfyClient {
    pub base_url: String,
    http: reqwest::Client,
}

/// Reference to a file in ComfyUI output storage (`/view` query params).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComfyFileRef {
    pub filename: String,
    pub subfolder: String,
    pub file_type: String,
}

impl ComfyClient {
    /// Build a client against `base_url` (trailing slashes stripped).
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        let base_url = base_url.into().trim_end_matches('/').to_string();
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .no_proxy()
            .build()
            .map_err(|e| Error::Http(e.to_string()))?;
        Ok(Self { base_url, http })
    }

    /// Client pointed at [`DEFAULT_COMFY_BASE`].
    pub fn default_local() -> Result<Self> {
        Self::new(DEFAULT_COMFY_BASE)
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn loopback_alt(base: &str) -> Option<String> {
        if base.contains("127.0.0.1") {
            Some(base.replace("127.0.0.1", "localhost"))
        } else if let Some(rest) = base.strip_prefix("http://localhost") {
            Some(format!("http://127.0.0.1{rest}"))
        } else if let Some(rest) = base.strip_prefix("https://localhost") {
            Some(format!("https://127.0.0.1{rest}"))
        } else {
            None
        }
    }

    async fn health_at(http: &reqwest::Client, base: &str) -> Result<()> {
        let stats = http
            .get(format!("{base}/system_stats"))
            .timeout(Duration::from_secs(3))
            .send()
            .await;
        match stats {
            Ok(resp) if resp.status().is_success() => Ok(()),
            Ok(resp) => {
                let status = resp.status();
                let queue = http
                    .get(format!("{base}/queue"))
                    .timeout(Duration::from_secs(3))
                    .send()
                    .await
                    .map_err(|e| Error::Http(e.to_string()))?;
                if queue.status().is_success() {
                    Ok(())
                } else {
                    Err(Error::Http(format!(
                        "health failed: system_stats={status}, queue={}",
                        queue.status()
                    )))
                }
            }
            Err(e) => {
                let queue = http
                    .get(format!("{base}/queue"))
                    .timeout(Duration::from_secs(3))
                    .send()
                    .await;
                match queue {
                    Ok(resp) if resp.status().is_success() => Ok(()),
                    Ok(resp) => Err(Error::Http(format!(
                        "health failed: system_stats err={e}, queue={}",
                        resp.status()
                    ))),
                    Err(e2) => Err(Error::Http(format!(
                        "health unreachable: system_stats={e}, queue={e2}"
                    ))),
                }
            }
        }
    }

    /// GET `/system_stats`, falling back to `/queue`. Accepts any 2xx.
    /// Also retries the other loopback host (`127.0.0.1` ↔ `localhost`) because
    /// Comfy on Windows often binds IPv4-only while `localhost` is IPv6.
    pub async fn health(&self) -> Result<()> {
        match Self::health_at(&self.http, &self.base_url).await {
            Ok(()) => Ok(()),
            Err(first) => {
                if let Some(alt) = Self::loopback_alt(&self.base_url) {
                    if Self::health_at(&self.http, &alt).await.is_ok() {
                        return Ok(());
                    }
                }
                Err(first)
            }
        }
    }

    /// POST `/prompt` with the API-format workflow graph; returns `prompt_id`.
    pub async fn queue_prompt(&self, workflow: Value) -> Result<String> {
        let client_id = format!("slate-{}", random_hex(8));
        let body = json!({
            "prompt": workflow,
            "client_id": client_id,
        });
        let resp = self
            .http
            .post(self.url("/prompt"))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Http(e.to_string()))?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| Error::Http(e.to_string()))?;
        if !status.is_success() {
            return Err(Error::Http(format!(
                "POST /prompt failed ({status}): {text}"
            )));
        }
        let v: Value = serde_json::from_str(&text)?;
        if let Some(err) = v.get("error") {
            return Err(Error::Comfy(format!("prompt error: {err}")));
        }
        if let Some(node_errors) = v.get("node_errors") {
            if let Some(obj) = node_errors.as_object() {
                if !obj.is_empty() {
                    return Err(Error::Comfy(format!("node_errors: {node_errors}")));
                }
            }
        }
        v.get("prompt_id")
            .and_then(|p| p.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| Error::Comfy(format!("missing prompt_id in response: {text}")))
    }

    /// POST `/interrupt` — stop the currently executing prompt (best-effort).
    pub async fn interrupt(&self) -> Result<()> {
        let resp = self
            .http
            .post(self.url("/interrupt"))
            .send()
            .await
            .map_err(|e| Error::Http(e.to_string()))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(Error::Http(format!(
                "POST /interrupt failed ({})",
                resp.status()
            )))
        }
    }

    /// POST `/queue` with `{ "clear": true }` — drop pending prompts.
    pub async fn clear_queue(&self) -> Result<()> {
        let resp = self
            .http
            .post(self.url("/queue"))
            .json(&json!({ "clear": true }))
            .send()
            .await
            .map_err(|e| Error::Http(e.to_string()))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(Error::Http(format!(
                "POST /queue clear failed ({})",
                resp.status()
            )))
        }
    }

    /// Poll `GET /history/{prompt_id}` until the entry appears or `timeout` elapses.
    ///
    /// Returns the history **entry** object (contains `outputs`, not the outer map).
    pub async fn wait_history(
        &self,
        prompt_id: &str,
        timeout: Duration,
        cancel: Option<&AtomicBool>,
    ) -> Result<Value> {
        let deadline = Instant::now() + timeout;
        let path = format!("/history/{prompt_id}");
        loop {
            let resp = self
                .http
                .get(self.url(&path))
                .send()
                .await
                .map_err(|e| Error::Http(e.to_string()))?;
            let status = resp.status();
            let text = resp.text().await.map_err(|e| Error::Http(e.to_string()))?;
            if !status.is_success() {
                return Err(Error::Http(format!(
                    "GET /history/{prompt_id} failed ({status}): {text}"
                )));
            }
            let map: Value = serde_json::from_str(&text)?;
            if let Some(entry) = map.get(prompt_id) {
                // Prefer entries that already expose outputs (or completed status).
                let has_outputs = entry
                    .get("outputs")
                    .and_then(|o| o.as_object())
                    .map(|o| !o.is_empty())
                    .unwrap_or(false);
                let completed = entry
                    .pointer("/status/completed")
                    .and_then(|c| c.as_bool())
                    .unwrap_or(false);
                if has_outputs || completed {
                    return Ok(entry.clone());
                }
            }
            if cancel.map(|c| c.load(Ordering::SeqCst)).unwrap_or(false) {
                let _ = self.interrupt().await;
                return Err(Error::Comfy("cancelled".into()));
            }
            if Instant::now() >= deadline {
                return Err(Error::Timeout(format!(
                    "timed out after {timeout:?} waiting for history of {prompt_id}"
                )));
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    /// Download a Comfy output file via `GET /view` into `dest`.
    pub async fn download_file(&self, r: &ComfyFileRef, dest: &Path) -> Result<()> {
        let resp = self
            .http
            .get(self.url("/view"))
            .query(&[
                ("filename", r.filename.as_str()),
                ("subfolder", r.subfolder.as_str()),
                ("type", r.file_type.as_str()),
            ])
            .send()
            .await
            .map_err(|e| Error::Http(e.to_string()))?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Http(format!("GET /view failed ({status}): {text}")));
        }
        let bytes = resp.bytes().await.map_err(|e| Error::Http(e.to_string()))?;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| Error::Io {
                path: parent.display().to_string(),
                source: e,
            })?;
        }
        fs::write(dest, &bytes).map_err(|e| Error::Io {
            path: dest.display().to_string(),
            source: e,
        })?;
        Ok(())
    }
}

/// Collect media file refs from one history node's output object.
fn files_from_node(node_out: &Value) -> Vec<ComfyFileRef> {
    let mut out = Vec::new();
    let Some(node_obj) = node_out.as_object() else {
        return out;
    };
    // Comfy still graphs: "images"; SaveVideo often serializes as images + animated.
    for key in ["images", "gifs", "videos"] {
        let Some(arr) = node_obj.get(key).and_then(|a| a.as_array()) else {
            continue;
        };
        for item in arr {
            let filename = item
                .get("filename")
                .and_then(|f| f.as_str())
                .unwrap_or("")
                .to_string();
            if filename.is_empty() {
                continue;
            }
            let subfolder = item
                .get("subfolder")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            let file_type = item
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("output")
                .to_string();
            out.push(ComfyFileRef {
                filename,
                subfolder,
                file_type,
            });
        }
    }
    out
}

/// Walk a history entry's `outputs` and collect image (and common media) file refs.
pub fn collect_output_files(history: &Value) -> Vec<ComfyFileRef> {
    collect_output_files_preferring(history, None)
}

/// Prefer files from `node_id` (pack `outputs.media.node_id`); fall back to every node.
pub fn collect_output_files_preferring(history: &Value, node_id: Option<&str>) -> Vec<ComfyFileRef> {
    let Some(outputs) = history.get("outputs").and_then(|o| o.as_object()) else {
        return Vec::new();
    };
    if let Some(id) = node_id {
        if let Some(node) = outputs.get(id) {
            let preferred = files_from_node(node);
            if !preferred.is_empty() {
                return preferred;
            }
        }
    }
    let mut out = Vec::new();
    for (_id, node) in outputs {
        out.extend(files_from_node(node));
    }
    out
}

/// Load `packs_dir/<pack_id>/manifest.json` + `workflow.api.json`.
pub fn load_pack(packs_dir: &Path, pack_id: &str) -> Result<(PackManifest, Value)> {
    let pack_dir = packs_dir.join(pack_id);
    let manifest_path = pack_dir.join("manifest.json");
    let workflow_path = pack_dir.join("workflow.api.json");
    let manifest = load_manifest(&manifest_path)?;
    let text = fs::read_to_string(&workflow_path).map_err(|e| Error::Io {
        path: workflow_path.display().to_string(),
        source: e,
    })?;
    let workflow: Value = serde_json::from_str(&text)?;
    Ok((manifest, workflow))
}

/// Load pack → inject values → queue → wait → download the pack's declared media output.
///
/// When `SLATE_DRY_RUN=1`, skips HTTP and writes `dry-run.txt` under `dest_dir`.
pub async fn generate_to_file(
    client: &ComfyClient,
    packs_dir: &Path,
    pack_id: &str,
    values: &HashMap<String, Value>,
    dest_dir: &Path,
    cancel: Option<&AtomicBool>,
) -> Result<PathBuf> {
    fs::create_dir_all(dest_dir).map_err(|e| Error::Io {
        path: dest_dir.display().to_string(),
        source: e,
    })?;

    if is_dry_run() {
        let path = dest_dir.join("dry-run.txt");
        let body = format!("SLATE_DRY_RUN pack_id={pack_id}\n");
        fs::write(&path, body.as_bytes()).map_err(|e| Error::Io {
            path: path.display().to_string(),
            source: e,
        })?;
        return Ok(path);
    }

    let (manifest, workflow) = load_pack(packs_dir, pack_id)?;
    let injected = inject_workflow(workflow, &manifest, values)?;
    if cancel.map(|c| c.load(Ordering::SeqCst)).unwrap_or(false) {
        return Err(Error::Comfy("cancelled".into()));
    }
    let prompt_id = client.queue_prompt(injected).await?;
    let history = client
        .wait_history(&prompt_id, Duration::from_secs(600), cancel)
        .await?;
    let preferred = manifest.outputs.get("media").map(|o| o.node_id.as_str());
    let files = collect_output_files_preferring(&history, preferred);
    let first = files.first().ok_or_else(|| {
        Error::Comfy(format!(
            "no output files for prompt_id={prompt_id} (wanted node {})",
            preferred.unwrap_or("(any)")
        ))
    })?;

    let dest = dest_dir.join(&first.filename);
    client.download_file(first, &dest).await?;
    Ok(dest)
}

fn is_dry_run() -> bool {
    env::var(SLATE_DRY_RUN_ENV)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
        .unwrap_or(false)
}

fn random_hex(n_bytes: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..n_bytes)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_alt_swaps_hosts() {
        assert_eq!(
            ComfyClient::loopback_alt("http://127.0.0.1:8188").as_deref(),
            Some("http://localhost:8188")
        );
        assert_eq!(
            ComfyClient::loopback_alt("http://localhost:8188").as_deref(),
            Some("http://127.0.0.1:8188")
        );
    }
}
