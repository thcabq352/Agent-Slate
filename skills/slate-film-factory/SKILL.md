---
name: slate-film-factory
description: "One-prompt film factory via optional slate-engine MCP (plan shots, prompts, ComfyUI packs)."
version: 0.1.0
---

# Slate Film Factory (slate-engine)

## Identity

- Binary: `slate-engine` (`cargo run -p slate-engine -- mcp` or release build)
- Comfy API default: `http://127.0.0.1:8188`
- Project store: same layout as the Slate app (`~/Documents/Slate` or `SLATE_DATA_DIR`)
- Optional headless path — does not replace the Electron studio

## When to use

- Plain-language brief → one scene, multiple shots, sectioned prompts
- Agent/automation needs multi-shot continuity without driving the UI
- Optional local ComfyUI generation via API workflow packs

## When NOT to use

- Interactive prompt editing, First AD chat in the app, sound dept UI → use Electron Slate
- Cloud generator web UIs only (no local Comfy) → plan/prompts still work; generation needs a pack + Comfy
- Unrelated video tooling / other product CLIs

## Preflight

1. Start ComfyUI API on port **8188** if generation is required
2. Prefer one heavy GPU job at a time (do not stack unrelated Comfy workloads)
3. Call `slate_health` — need engine ok; for live generation, Comfy + at least one brain
4. Tool timeout for `slate_film_factory`: **≥ 900s** (1800s on slow GPUs)

## Primary tool

```json
{
  "brief": "Rainy neon rooftop chase, cinematic, ~8s",
  "pack_id": "default-still",
  "shot_count": 4
}
```

Tool name: **`slate_film_factory`**. Synchronous: blocks until the run finishes (or fails).

Dry-run without GPU: set `SLATE_DRY_RUN=1` on the engine process.

## Other tools

`slate_health`, `slate_status`, `slate_cancel`, `slate_list_projects`, `slate_get_project`, `slate_list_takes`, `slate_generate_shot`

## Register (stdio MCP)

```bash
# Example: Claude Code
claude mcp add slate-engine -- /absolute/path/to/target/release/slate-engine mcp
```

Any MCP host that can spawn a stdio server works the same way. See `docs/engine.md`.
