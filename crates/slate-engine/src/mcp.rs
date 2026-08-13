//! Stdio JSON-RPC 2.0 MCP server matching `mcp/slate-mcp.mjs` line protocol.
//!
//! Logs go to stderr only — stdout is reserved for JSON-RPC replies.

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::tools::{self, EngineCtx};

/// Run the MCP stdio loop until stdin EOF.
pub async fn serve(ctx: EngineCtx) -> Result<(), String> {
    eprintln!("slate-engine mcp: ready (stdio JSON-RPC)");

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut stdout = tokio::io::stdout();
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader
            .read_line(&mut line)
            .await
            .map_err(|e| format!("stdin read: {e}"))?;
        if n == 0 {
            break;
        }

        let msg: Value = match serde_json::from_str(line.trim()) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(Value::Null);

        match method {
            "initialize" | "tools/list" | "tools/call" => {
                match handle_known(method, &params, &ctx).await {
                    Ok(result) => {
                        // Known methods always reply (id may be null).
                        write_reply(&mut stdout, id.unwrap_or(Value::Null), result).await?;
                    }
                    Err(message) => {
                        if let Some(id) = id {
                            write_error(&mut stdout, id, message).await?;
                        }
                    }
                }
            }
            // Match slate-mcp.mjs: unknown methods with id → empty result; else silence.
            _ => {
                if let Some(id) = id {
                    write_reply(&mut stdout, id, json!({})).await?;
                }
            }
        }
    }

    Ok(())
}

async fn handle_known(method: &str, params: &Value, ctx: &EngineCtx) -> Result<Value, String> {
    match method {
        "initialize" => {
            let protocol_version = params
                .get("protocolVersion")
                .and_then(|v| v.as_str())
                .unwrap_or("2024-11-05");
            Ok(json!({
                "protocolVersion": protocol_version,
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "slate", "version": env!("CARGO_PKG_VERSION") }
            }))
        }
        "tools/list" => Ok(json!({ "tools": tools::catalog() })),
        "tools/call" => {
            let name = params
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "missing tool name".to_string())?;
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let result = tools::invoke(name, args, ctx).await?;
            let text = serde_json::to_string_pretty(&result).map_err(|e| e.to_string())?;
            Ok(json!({
                "content": [{ "type": "text", "text": text }]
            }))
        }
        other => Err(format!("internal: unexpected method {other}")),
    }
}

async fn write_reply(
    stdout: &mut tokio::io::Stdout,
    id: Value,
    result: Value,
) -> Result<(), String> {
    let msg = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    });
    write_line(stdout, &msg).await
}

async fn write_error(
    stdout: &mut tokio::io::Stdout,
    id: Value,
    message: String,
) -> Result<(), String> {
    let msg = json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": -32000, "message": message },
    });
    write_line(stdout, &msg).await
}

async fn write_line(stdout: &mut tokio::io::Stdout, msg: &Value) -> Result<(), String> {
    let mut bytes = serde_json::to_vec(msg).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    stdout
        .write_all(&bytes)
        .await
        .map_err(|e| format!("stdout write: {e}"))?;
    stdout
        .flush()
        .await
        .map_err(|e| format!("stdout flush: {e}"))?;
    Ok(())
}
