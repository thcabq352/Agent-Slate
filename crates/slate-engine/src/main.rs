//! slate-engine CLI — loopback HTTP control server (default) and future MCP mode.

use clap::{Parser, Subcommand};
use slate_engine::{http, EngineCtx};

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
    /// Stdio MCP server (Task 11 — not yet implemented).
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
            eprintln!("slate-engine mcp: not implemented yet (Task 11)");
            std::process::exit(1);
        }
    }
}
