//! slate-engine library — config, control descriptor, tools, HTTP + MCP serve.
//!
//! Binary entry is `main.rs` (CLI: `serve` default, `mcp` stdio).

pub mod config;
pub mod control_desc;
pub mod http;
pub mod mcp;
pub mod tools;

pub use config::{apply_env, load_config, EngineConfig};
pub use tools::{catalog, invoke, EngineCtx, ToolInfo};
