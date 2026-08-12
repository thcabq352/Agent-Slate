//! slate-engine CLI — loopback HTTP control server (default) and stdio MCP.

use clap::{Parser, Subcommand};
use slate_engine::{http, mcp, EngineCtx};

#[derive(Parser, Debug)]
#[command(name = "slate-engine", version, about = "Slate film factory engine")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start loopback HTTP control server (writes control.json descriptor).
    Serve,
    /// Stdio JSON-RPC MCP server (Hermes / Claude Code).
    Mcp,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Commands::Serve) {
        Commands::Serve => {
            let ctx = EngineCtx::from_env();
            if let Err(e) = http::serve(ctx).await {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
        Commands::Mcp => {
            let ctx = EngineCtx::from_env();
            if let Err(e) = mcp::serve(ctx).await {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    }
}
