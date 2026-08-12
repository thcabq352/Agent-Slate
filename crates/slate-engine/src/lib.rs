//! slate-engine library — config, control descriptor, tools, HTTP + MCP serve.
//!
//! Binary entry is `main.rs` (CLI: `serve` default, `mcp` stdio).

pub mod config;
pub mod continuity;
pub mod control_desc;
pub mod factory;
pub mod first_ad;
pub mod http;
pub mod mcp;
pub mod music;
pub mod notes;
pub mod prompts;
pub mod quality_gate;
pub mod tools;

pub use config::{apply_env, load_config, EngineConfig};
pub use tools::{catalog, invoke, EngineCtx, JobStatus, ToolInfo};
