# slate-engine (optional headless film factory)

`slate-engine` is an optional Rust binary that plans a **one-scene** shoot from a plain-language brief, writes sectioned prompts, compiles them for ComfyUI-style packs, and can queue generation on a **local** ComfyUI API. It is meant for agents and automation; the Electron app remains the interactive studio.

## Why it exists

- The app already has project JSON, First AD-style actions, brains (Claude Code / Codex / local), and MCP for CRUD.
- Agents still need a **headless** path that can run multi-step planning and optional generation without driving the UI.
- ComfyUI becomes the generation backplane via **checked-in API workflow packs** (inject positive/negative/size), default `http://127.0.0.1:8188`.

## Crates

| Crate | Role |
|-------|------|
| `slate-domain` | Project model (camelCase JSON, same spirit as `src/shared/types.ts`), First AD actions, store under `~/Documents/Slate` or `SLATE_DATA_DIR` |
| `slate-brain` | Local OpenAI-compatible chat, Claude Code CLI, Codex CLI |
| `slate-comfy` | Pack manifest inject, Comfy HTTP client, dry-run |
| `slate-engine` | Config, control descriptor, HTTP `/tools` + `/invoke`, stdio MCP, `slate_film_factory` |

## Quick start

```bash
cargo build -p slate-engine --release
cargo test --workspace

# Agent MCP (stdio)
cargo run -p slate-engine --release -- mcp

# Loopback HTTP (writes control.json with port + bearer token)
cargo run -p slate-engine --release -- serve
```

Dry-run (stub planner + marker take files, no Comfy GPU):

```bash
# Windows PowerShell
$env:SLATE_DRY_RUN = "1"
cargo run -p slate-engine -- mcp

# Unix
SLATE_DRY_RUN=1 cargo run -p slate-engine -- mcp
```

## Tools (MCP / HTTP invoke)

| Tool | Notes |
|------|--------|
| `slate_health` | Engine + Comfy + brain availability |
| `slate_film_factory` | **Synchronous** one-scene pipeline from a `brief` |
| `slate_generate_shot` | Re-roll one shot through the pack |
| `slate_list_projects` / `slate_get_project` | Project store |
| `slate_list_takes` | Media paths for a project |
| `slate_status` / `slate_cancel` | Job status / cancel between shots |

`slate_film_factory` should use a long tool timeout (e.g. 900–1800s) when generation is enabled.

## Environment

| Variable | Meaning |
|----------|---------|
| `SLATE_DATA_DIR` | Project root (default: Documents/Slate) |
| `SLATE_COMFY_URL` | Comfy base URL (default `http://127.0.0.1:8188`) |
| `SLATE_PACKS_DIR` | Workflow packs directory |
| `SLATE_BRAIN` | `local` \| `claude` \| `codex` |
| `SLATE_DRY_RUN` | `1` = stub plan + dry-run takes |

HTTP control descriptor (same idea as the Electron app’s control server):  
`~/.config/slate/control.json` or `%APPDATA%/slate/control.json` when `serve` is running.

## Comfy packs

See `workflows/packs/<id>/workflow.api.json` + `manifest.json`. The shipped `default-still` pack is a **template**; node IDs and checkpoints must match your local Comfy graph for real generation. Dry-run does not need a working checkpoint.

## Relation to Electron MCP

| Path | Use when |
|------|----------|
| `mcp/slate-mcp.mjs` + app running | Read/write projects while the UI is open |
| `slate-engine mcp` | Headless film factory + generation without the UI |

Both can coexist; they share the project JSON layout on disk when `SLATE_DATA_DIR` matches.
