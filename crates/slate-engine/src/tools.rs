//! Tool catalog and dispatch for HTTP / MCP.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use serde_json::{json, Value};
use slate_comfy::ComfyClient;

use crate::config::{self, EngineConfig};
use crate::factory::{self, FilmFactoryArgs};

/// Mid-flight / last factory job status for `slate_status`.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobStatus {
    pub active: bool,
    pub step: String,
    pub project_id: Option<String>,
    pub message: String,
    /// Scene continuity one-liner for agents/UI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuity_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_shot_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_plan: Option<String>,
}

/// Shared engine runtime context for tool handlers.
#[derive(Debug, Clone)]
pub struct EngineCtx {
    pub config: EngineConfig,
    /// Set by `slate_cancel`; checked between factory shots.
    pub cancel: Arc<AtomicBool>,
    /// Last / active job status.
    pub job: Arc<Mutex<JobStatus>>,
}

impl EngineCtx {
    pub fn new(config: EngineConfig) -> Self {
        config::apply_env(&config);
        Self {
            config,
            cancel: Arc::new(AtomicBool::new(false)),
            job: Arc::new(Mutex::new(JobStatus::default())),
        }
    }

    pub fn from_env() -> Self {
        Self::new(config::load_config())
    }

    /// Build a context for integration tests (temp data dir, resolved packs, dry-run from env).
    pub async fn for_test(data_dir: impl AsRef<Path>) -> Self {
        let data_dir = data_dir.as_ref().to_path_buf();
        let dry_run = std::env::var("SLATE_DRY_RUN")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
            .unwrap_or(true);
        let packs_dir = resolve_packs_dir_for_test();
        let comfy_base_url = std::env::var("SLATE_COMFY_URL")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| slate_comfy::DEFAULT_COMFY_BASE.to_string());
        let mut base = config::load_config();
        base.data_dir = data_dir;
        base.comfy_base_url = comfy_base_url;
        base.packs_dir = packs_dir;
        base.brain_default = "local".into();
        base.bind = "127.0.0.1".into();
        base.dry_run = dry_run;
        Self::new(base)
    }
}

fn resolve_packs_dir_for_test() -> PathBuf {
    if let Ok(p) = std::env::var("SLATE_PACKS_DIR") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    // slate-engine crate → workspace root/workflows/packs
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        manifest.join("../../workflows/packs"),
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("workflows/packs"),
        PathBuf::from("workflows/packs"),
    ];
    for c in candidates {
        if c.join("default-still").join("manifest.json").is_file() {
            return c.canonicalize().unwrap_or(c);
        }
    }
    manifest.join("../../workflows/packs")
}

/// One entry in the tool catalog returned by `GET /tools` and MCP `tools/list`.
#[derive(Debug, Clone, Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Catalog of tools exposed by the engine.
pub fn catalog() -> Vec<ToolInfo> {
    vec![
        ToolInfo {
            name: "slate_health".into(),
            description: "Report engine, ComfyUI, and brain availability.".into(),
            input_schema: json!({ "type": "object", "properties": {} }),
        },
        ToolInfo {
            name: "slate_list_projects".into(),
            description: "List Slate projects with scene and shot counts.".into(),
            input_schema: json!({ "type": "object", "properties": {} }),
        },
        ToolInfo {
            name: "slate_get_project".into(),
            description: "Get a full Slate project document (args: projectId).".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "projectId": { "type": "string" }
                },
                "required": ["projectId"]
            }),
        },
        ToolInfo {
            name: "slate_film_factory".into(),
            description: "Synchronous one-scene film factory: brief → project → prompts → Comfy takes (blocking; long timeout).".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "brief": { "type": "string", "description": "Plain-language scene brief" },
                    "pack_id": { "type": "string", "description": "Comfy pack id (default default-still)" },
                    "brain": { "type": "string", "enum": ["claude", "codex", "local"] },
                    "shot_count": { "type": "integer", "minimum": 4, "maximum": 8 },
                    "project_name": { "type": "string" }
                },
                "required": ["brief"]
            }),
        },
        ToolInfo {
            name: "slate_generate_shot".into(),
            description: "Re-roll one shot through a Comfy pack with quality-gate retries (blocking).".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "projectId": { "type": "string" },
                    "shotId": { "type": "string" },
                    "pack_id": { "type": "string" }
                },
                "required": ["projectId", "shotId"]
            }),
        },
        ToolInfo {
            name: "slate_judge_take".into(),
            description: "Score a media file with the local VL judge (qwen3.5:9b preferred). Args: mediaPath, prompt?, continuity?.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "mediaPath": { "type": "string" },
                    "prompt": { "type": "string" },
                    "continuity": { "type": "string" }
                },
                "required": ["mediaPath"]
            }),
        },
        ToolInfo {
            name: "slate_first_ad".into(),
            description: "First AD conversational turn: plan/mutate a project (args: projectId, message, history?, brain?).".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "projectId": { "type": "string" },
                    "message": { "type": "string" },
                    "history": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "role": { "type": "string" },
                                "text": { "type": "string" }
                            }
                        }
                    },
                    "brain": { "type": "string", "enum": ["claude", "codex", "local"] }
                },
                "required": ["projectId", "message"]
            }),
        },
        ToolInfo {
            name: "slate_note_write".into(),
            description: "Write an atomic note (continuity / quality / plan) for a project.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "projectId": { "type": "string" },
                    "kind": {
                        "type": "string",
                        "description": "continuity | shot_decision | quality_feedback | scene_plan | general"
                    },
                    "title": { "type": "string" },
                    "body": { "type": "string" },
                    "tags": { "type": "array", "items": { "type": "string" } },
                    "sceneId": { "type": "string" },
                    "shotId": { "type": "string" }
                },
                "required": ["projectId", "kind", "title", "body"]
            }),
        },
        ToolInfo {
            name: "slate_note_search".into(),
            description: "Search atomic notes for a project (query/kind/scene/shot filters).".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "projectId": { "type": "string" },
                    "query": { "type": "string" },
                    "kind": { "type": "string" },
                    "sceneId": { "type": "string" },
                    "shotId": { "type": "string" },
                    "limit": { "type": "integer" }
                },
                "required": ["projectId"]
            }),
        },
        ToolInfo {
            name: "slate_list_takes".into(),
            description: "List take media for a project (optional shotId filter).".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "projectId": { "type": "string" },
                    "shotId": { "type": "string" }
                },
                "required": ["projectId"]
            }),
        },
        ToolInfo {
            name: "slate_cancel".into(),
            description: "Cancel the active film factory run (stops between shots).".into(),
            input_schema: json!({ "type": "object", "properties": {} }),
        },
        ToolInfo {
            name: "slate_status".into(),
            description: "Return active/last film factory job status.".into(),
            input_schema: json!({ "type": "object", "properties": {} }),
        },
    ]
}

/// Dispatch a named tool. Returns JSON value or error string.
pub async fn invoke(tool: &str, args: Value, ctx: &EngineCtx) -> Result<Value, String> {
    match tool {
        "slate_health" => slate_health(ctx).await,
        "slate_list_projects" => slate_list_projects(),
        "slate_get_project" => slate_get_project(args),
        "slate_film_factory" => slate_film_factory(ctx, args).await,
        "slate_generate_shot" => slate_generate_shot(ctx, args).await,
        "slate_judge_take" => slate_judge_take(ctx, args).await,
        "slate_first_ad" => slate_first_ad(ctx, args).await,
        "slate_note_write" => slate_note_write(args),
        "slate_note_search" => slate_note_search(args),
        "slate_list_takes" => slate_list_takes(args),
        "slate_cancel" => slate_cancel(ctx),
        "slate_status" => slate_status(ctx),
        other => Err(format!("Unknown tool: {other}")),
    }
}

async fn slate_health(ctx: &EngineCtx) -> Result<Value, String> {
    let url = ctx.config.comfy_base_url.clone();
    let comfy_ok = match ComfyClient::new(&url) {
        Ok(client) => client.health().await.is_ok(),
        Err(_) => false,
    };

    let brain = slate_brain::brain_status(None).await;
    let brain_json = serde_json::to_value(&brain).map_err(|e| e.to_string())?;

    let judge = slate_brain::judge_vision_status(
        Some(ctx.config.judge_endpoint.as_str()),
        Some(ctx.config.judge_model.as_str()),
    )
    .await;
    let judge_json = serde_json::to_value(&judge).map_err(|e| e.to_string())?;
    let gate = ctx.config.quality_gate();
    let gate_json = serde_json::to_value(&gate).map_err(|e| e.to_string())?;

    Ok(json!({
        "engine": true,
        "comfy": {
            "ok": comfy_ok,
            "url": url,
        },
        "brain": brain_json,
        "vision": judge_json,
        "qualityGate": gate_json,
        "dryRun": ctx.config.dry_run,
    }))
}

fn slate_list_projects() -> Result<Value, String> {
    let metas = slate_domain::list_projects().map_err(|e| e.to_string())?;
    serde_json::to_value(metas).map_err(|e| e.to_string())
}

fn slate_get_project(args: Value) -> Result<Value, String> {
    let project_id = args
        .get("projectId")
        .or_else(|| args.get("project_id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing required arg: projectId".to_string())?;
    match slate_domain::open_project(project_id).map_err(|e| e.to_string())? {
        Some(p) => serde_json::to_value(p).map_err(|e| e.to_string()),
        None => Err("Project not found".to_string()),
    }
}

async fn slate_film_factory(ctx: &EngineCtx, args: Value) -> Result<Value, String> {
    // Accept camelCase aliases for tool callers.
    let brief = args
        .get("brief")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing required arg: brief".to_string())?
        .to_string();
    let pack_id = args
        .get("pack_id")
        .or_else(|| args.get("packId"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let brain = args
        .get("brain")
        .and_then(|v| v.as_str())
        .and_then(|s| match s.to_lowercase().as_str() {
            "claude" => Some(slate_domain::BrainBackend::Claude),
            "codex" => Some(slate_domain::BrainBackend::Codex),
            "local" => Some(slate_domain::BrainBackend::Local),
            _ => None,
        });
    let shot_count = args
        .get("shot_count")
        .or_else(|| args.get("shotCount"))
        .and_then(|v| v.as_u64())
        .map(|n| n as u8);
    let project_name = args
        .get("project_name")
        .or_else(|| args.get("projectName"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let result = factory::run_film_factory(
        ctx,
        FilmFactoryArgs {
            brief,
            pack_id,
            brain,
            shot_count,
            project_name,
        },
    )
    .await;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

async fn slate_generate_shot(ctx: &EngineCtx, args: Value) -> Result<Value, String> {
    let project_id = args
        .get("projectId")
        .or_else(|| args.get("project_id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing required arg: projectId".to_string())?;
    let shot_id = args
        .get("shotId")
        .or_else(|| args.get("shot_id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing required arg: shotId".to_string())?;
    let pack_id = args
        .get("pack_id")
        .or_else(|| args.get("packId"))
        .and_then(|v| v.as_str());

    let outcome = factory::generate_one_shot(ctx, project_id, shot_id, pack_id).await?;
    serde_json::to_value(outcome).map_err(|e| e.to_string())
}

async fn slate_judge_take(ctx: &EngineCtx, args: Value) -> Result<Value, String> {
    use crate::quality_gate::judge_media;
    use std::path::PathBuf;

    let media_path = args
        .get("mediaPath")
        .or_else(|| args.get("media_path"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing required arg: mediaPath".to_string())?;
    let prompt = args
        .get("prompt")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let continuity = args
        .get("continuity")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let path = PathBuf::from(media_path);
    if !path.is_file() {
        return Err(format!("media file not found: {media_path}"));
    }

    let outcome = judge_media(&ctx.config, &path, &prompt, &continuity).await?;
    serde_json::to_value(json!({
        "skipped": outcome.skipped,
        "skipReason": outcome.skip_reason,
        "verdict": outcome.verdict,
    }))
    .map_err(|e| e.to_string())
}

async fn slate_first_ad(ctx: &EngineCtx, args: Value) -> Result<Value, String> {
    use crate::first_ad::{FirstAdArgs, FirstAdChatMsg, run_first_ad};

    let project_id = args
        .get("projectId")
        .or_else(|| args.get("project_id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing required arg: projectId".to_string())?
        .to_string();
    let message = args
        .get("message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing required arg: message".to_string())?
        .to_string();

    let history = args
        .get("history")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let role = m.get("role")?.as_str()?.to_string();
                    let text = m.get("text")?.as_str()?.to_string();
                    Some(FirstAdChatMsg { role, text })
                })
                .collect()
        })
        .unwrap_or_default();

    let brain = args
        .get("brain")
        .and_then(|v| v.as_str())
        .and_then(|s| match s {
            "claude" => Some(slate_domain::BrainBackend::Claude),
            "codex" => Some(slate_domain::BrainBackend::Codex),
            "local" => Some(slate_domain::BrainBackend::Local),
            _ => None,
        });

    let result = run_first_ad(
        ctx,
        FirstAdArgs {
            project_id,
            message,
            history,
            brain,
        },
    )
    .await;

    // Surface planning on job status for agents/UI.
    if let Ok(mut g) = ctx.job.lock() {
        g.step = "first_ad".into();
        g.message = result.reply.chars().take(160).collect();
        g.project_id = Some(result.continuity.project_id.clone());
        g.continuity_summary = Some(result.continuity.summary_one_line());
        g.scene_plan = Some(result.scene_plan_summary.clone());
        g.last_shot_id = result.focus_shot_id.clone();
        g.active = false;
    }

    if !result.ok {
        return Err(result
            .error
            .unwrap_or_else(|| result.reply.clone()));
    }
    serde_json::to_value(result).map_err(|e| e.to_string())
}

fn slate_note_write(args: Value) -> Result<Value, String> {
    use crate::notes::{write_note, NoteWriteArgs};

    let project_id = args
        .get("projectId")
        .or_else(|| args.get("project_id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing required arg: projectId".to_string())?
        .to_string();
    let kind = args
        .get("kind")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing required arg: kind".to_string())?
        .to_string();
    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing required arg: title".to_string())?
        .to_string();
    let body = args
        .get("body")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing required arg: body".to_string())?
        .to_string();
    let tags = args
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let scene_id = args
        .get("sceneId")
        .or_else(|| args.get("scene_id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let shot_id = args
        .get("shotId")
        .or_else(|| args.get("shot_id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let note = write_note(NoteWriteArgs {
        project_id,
        kind,
        title,
        body,
        tags,
        scene_id,
        shot_id,
    })?;
    serde_json::to_value(note).map_err(|e| e.to_string())
}

fn slate_note_search(args: Value) -> Result<Value, String> {
    use crate::notes::{search_notes, NoteSearchArgs};

    let project_id = args
        .get("projectId")
        .or_else(|| args.get("project_id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing required arg: projectId".to_string())?
        .to_string();
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let kind = args
        .get("kind")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let scene_id = args
        .get("sceneId")
        .or_else(|| args.get("scene_id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let shot_id = args
        .get("shotId")
        .or_else(|| args.get("shot_id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let limit = args.get("limit").and_then(|v| v.as_u64()).map(|n| n as usize);

    let hits = search_notes(NoteSearchArgs {
        project_id,
        query,
        kind,
        scene_id,
        shot_id,
        limit,
    })?;
    serde_json::to_value(hits).map_err(|e| e.to_string())
}

fn slate_list_takes(args: Value) -> Result<Value, String> {
    let project_id = args
        .get("projectId")
        .or_else(|| args.get("project_id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing required arg: projectId".to_string())?;
    let shot_id = args
        .get("shotId")
        .or_else(|| args.get("shot_id"))
        .and_then(|v| v.as_str());
    factory::list_takes(project_id, shot_id)
}

fn slate_cancel(ctx: &EngineCtx) -> Result<Value, String> {
    ctx.cancel.store(true, Ordering::SeqCst);
    Ok(json!({
        "ok": true,
        "message": "cancel requested; factory stops between shots"
    }))
}

fn slate_status(ctx: &EngineCtx) -> Result<Value, String> {
    let job = ctx
        .job
        .lock()
        .map(|g| g.clone())
        .unwrap_or_default();
    let cancelled = ctx.cancel.load(Ordering::SeqCst);
    Ok(json!({
        "active": job.active,
        "step": job.step,
        "projectId": job.project_id,
        "message": job.message,
        "continuitySummary": job.continuity_summary,
        "lastShotId": job.last_shot_id,
        "scenePlan": job.scene_plan,
        "cancelRequested": cancelled,
    }))
}
