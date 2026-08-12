//! Tool catalog and dispatch for HTTP / MCP.

use serde::Serialize;
use serde_json::{json, Value};
use slate_comfy::ComfyClient;

use crate::config::EngineConfig;

/// Shared engine runtime context for tool handlers.
#[derive(Debug, Clone)]
pub struct EngineCtx {
    pub config: EngineConfig,
}

impl EngineCtx {
    pub fn new(config: EngineConfig) -> Self {
        crate::config::apply_env(&config);
        Self { config }
    }

    pub fn from_env() -> Self {
        Self::new(crate::config::load_config())
    }
}

/// One entry in the tool catalog returned by `GET /tools` and MCP `tools/list`.
#[derive(Debug, Clone, Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Catalog of tools exposed by the engine (more land in later tasks).
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
    ]
}

/// Dispatch a named tool. Returns JSON value or error string.
pub async fn invoke(tool: &str, args: Value, ctx: &EngineCtx) -> Result<Value, String> {
    match tool {
        "slate_health" => slate_health(ctx).await,
        "slate_list_projects" => slate_list_projects(),
        "slate_get_project" => slate_get_project(args),
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

    Ok(json!({
        "engine": true,
        "comfy": {
            "ok": comfy_ok,
            "url": url,
        },
        "brain": brain_json,
    }))
}

fn slate_list_projects() -> Result<Value, String> {
    let metas = slate_domain::list_projects().map_err(|e| e.to_string())?;
    serde_json::to_value(metas).map_err(|e| e.to_string())
}

fn slate_get_project(args: Value) -> Result<Value, String> {
    let project_id = args
        .get("projectId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing required arg: projectId".to_string())?;
    match slate_domain::open_project(project_id).map_err(|e| e.to_string())? {
        Some(p) => serde_json::to_value(p).map_err(|e| e.to_string()),
        None => Err("Project not found".to_string()),
    }
}
