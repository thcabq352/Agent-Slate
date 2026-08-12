//! Loopback HTTP control server (axum) — `GET /tools`, `POST /invoke`.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::http::{header, Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::net::TcpListener;

use crate::control_desc::write_control_descriptor;
use crate::tools::{self, EngineCtx};

/// Shared HTTP app state.
#[derive(Clone)]
struct AppState {
    token: String,
    ctx: Arc<EngineCtx>,
}

#[derive(Debug, Deserialize)]
struct InvokeBody {
    tool: String,
    #[serde(default)]
    args: Value,
}

/// Bind `127.0.0.1:0`, write control descriptor, serve until cancelled.
pub async fn serve(ctx: EngineCtx) -> Result<(), String> {
    let token = random_token_hex(24);
    let host = ctx.config.bind.clone();
    let state = AppState {
        token: token.clone(),
        ctx: Arc::new(ctx),
    };

    let app = Router::new()
        .route("/tools", get(get_tools))
        .route("/invoke", post(post_invoke))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state.clone());

    let addr: SocketAddr = format!("{host}:0")
        .parse()
        .map_err(|e| format!("bad bind address: {e}"))?;
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| format!("bind failed: {e}"))?;
    let local = listener
        .local_addr()
        .map_err(|e| format!("local_addr: {e}"))?;
    let port = local.port();

    let path = write_control_descriptor(port, &token)
        .map_err(|e| format!("write control descriptor: {e}"))?;
    eprintln!(
        "slate-engine listening on http://{host}:{port} (descriptor {})",
        path.display()
    );

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("serve error: {e}"))
}

async fn auth_middleware(
    State(state): State<AppState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let expected = format!("Bearer {}", state.token);
    let ok = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|v| v == expected)
        .unwrap_or(false);
    if !ok {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "unauthorized" })),
        )
            .into_response();
    }
    next.run(req).await
}

async fn get_tools() -> impl IntoResponse {
    Json(json!({ "tools": tools::catalog() }))
}

async fn post_invoke(
    State(state): State<AppState>,
    Json(body): Json<InvokeBody>,
) -> Response {
    match tools::invoke(&body.tool, body.args, state.ctx.as_ref()).await {
        Ok(result) => (StatusCode::OK, Json(json!({ "result": result }))).into_response(),
        Err(e) => {
            // Unknown tool → 404; other handler errors → 400 (mirrors control.ts).
            let status = if e.starts_with("Unknown tool:") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_REQUEST
            };
            (status, Json(json!({ "error": e }))).into_response()
        }
    }
}

fn random_token_hex(n_bytes: usize) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..n_bytes)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect()
}
