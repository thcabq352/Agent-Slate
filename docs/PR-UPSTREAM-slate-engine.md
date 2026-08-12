# Draft PR: optional slate-engine (for wassermanproductions/slate)

**Base:** `wassermanproductions/slate` `main`  
**Head:** `thcabq352/slate` branch `pr/slate-engine`

## Title

`feat: optional slate-engine — headless film factory (MCP + ComfyUI packs)`

## Summary

Adds an optional Rust binary, `slate-engine`, that can plan a one-scene shoot from a plain-language brief, write sectioned prompts, compile for ComfyUI-style packs, and queue local ComfyUI generation. It exposes stdio MCP and a localhost HTTP control API so agents can run the pipeline without the Electron UI.

The Electron app is unchanged as the primary interactive studio. This is an additive, optional path (Rust toolchain required only if you build the engine).

## Why

1. MCP today can CRUD projects while the app runs; agents still lack a headless multi-step plan → generate loop.
2. ComfyUI is already a compile dialect in model profiles; packs make generation real via the local Comfy API (`127.0.0.1:8188` by default).
3. Domain code mirrors existing project JSON / First AD action semantics so disk projects stay compatible with the app.
4. Same no-API-key brain model: Claude Code, Codex, or local OpenAI-compatible servers.

## What’s included

- Cargo workspace: `slate-domain`, `slate-brain`, `slate-comfy`, `slate-engine`
- Workflow pack skeleton: `workflows/packs/default-still/` (template; align node IDs/checkpoints for real GPU runs)
- Docs: `docs/engine.md`, README section, `skills/slate-film-factory/SKILL.md`
- Tests: `cargo test --workspace` (includes dry-run factory integration test)

## What’s not included

- Replacing Electron or changing default install flow
- Bundling ComfyUI or models
- Fork branding / third-party maintainer attribution
- Wiring Electron IPC to the engine (possible follow-up)

## How to try

```bash
cargo build -p slate-engine --release
cargo test --workspace
SLATE_DRY_RUN=1 cargo run -p slate-engine -- mcp
```

Live generation needs Comfy on `:8188` and a pack matching your graph (see `workflows/packs/default-still/README.md`).

## Follow-ups (optional)

- First-class take `mediaPath` field
- Comfy interrupt on cancel; CLI process kill-on-timeout
- Real still/video packs for common installs
- Electron attach to engine HTTP

## License

Apache-2.0. Original copyright and NOTICE for Sam Wasserman retained.
