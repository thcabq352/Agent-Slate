---
name: slate-film-factory
description: "One-prompt film factory via slate-engine MCP (plan shots, prompts, ComfyUI packs, vision health)."
version: 0.2.0
metadata:
  hermes:
    tags: [slate, film, comfyui, prompts, mcp, ollama, vision, ltx]
    category: media
---

# Slate Film Factory

Shipped V1 snapshot: `docs/STATUS.md` in the Slate repo.

## Identity
- Engine: `cargo run -p slate-engine -- mcp` (or installed `slate-engine mcp`)
- Comfy API default: http://127.0.0.1:8188
- Vision judge (preferred): **qwen3.5:9b** via Ollama — weights not bundled (`ollama pull qwen3.5:9b`)

## When to use
- Plain-language scene → 4–8 shots + local Comfy takes in a Slate project
- Multi-shot continuity / prompt bible
- Agent/automation without clicking the Electron editor

## When NOT to use
| Signal | Route |
|--------|--------|
| Complex LTX I2V / lipsync / VFX graphs, Video Buddy movie builder | video-buddy / master-agent |
| HyperFrames / brand package | forge profile |
| No Comfy and no desire to install | plan/prompts only (`SLATE_DRY_RUN=1`) |
| A finished **cut** of several video takes | not shipped — stills factory or one-clip `slate_run_pack` |

Simple **LTX T2V** is in-repo: `pack_id: "default-video"` (49 frames, 768 long-edge). Prefer `default-still` for a 4-shot factory.

## Preflight
1. Start ComfyUI API on 8188 if generation is required
2. One GPU owner only — do not stack heavy Comfy jobs
3. `slate_health` — engine ok; for generation, Comfy ok + at least one brain
4. Vision: `vision.ready` true. Default **qwen3.5:9b** (`SLATE_JUDGE_MODEL`). If missing: `ollama pull qwen3.5:9b`
5. Tool timeout for **blocking** `slate_film_factory`: **1800s** (900s minimum)

## Primary tool (Hermes)

```
slate_film_factory { "brief": "…", "pack_id": "default-still", "shot_count": 4 }
```

**Blocking.** Do **not** pass `"background": true` from Hermes. That flag is for the Electron dock only (it polls `slate_status`).

Dry-run: `SLATE_DRY_RUN=1` on the engine process.

## Other tools
`slate_health`, `slate_status`, `slate_cancel` (also Comfy `/interrupt`), `slate_list_projects`, `slate_get_project`, `slate_list_takes` (`mediaPath` on takes), `slate_generate_shot`, `slate_judge_take`, `slate_first_ad`, `slate_note_write`, `slate_note_search`, `slate_list_packs`, `slate_run_pack`, `slate_compile_music`.

After generate, the engine runs a **quality gate** (Ollama VL). Failures auto-retry with seed + prompt pickups. Bad judge JSON / dry-run / `.txt` → skip, keep take. Check `quality` / `attempts`.

**First AD:** `slate_first_ad` `{ "projectId", "message", "history"? }`.

**Atomic Notes:** `.notes/notes.jsonl`.

**Packs:** `default-still` (Flux). `default-video` (LTX 2.3 distilled T2V, factory-clamped 768×432 / 49 frames). `slate_compile_music` is text only.

## Register (Hermes / MCP)
```bash
hermes mcp add slate -- /absolute/path/to/target/debug/slate-engine mcp
```

Set server `timeout: 1800`. Rebuild `slate-engine` after pulling engine changes. Restart the Hermes gateway so it reloads MCP.
