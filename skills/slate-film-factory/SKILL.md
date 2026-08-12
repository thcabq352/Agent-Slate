---
name: slate-film-factory
description: "One-prompt film factory via slate-engine MCP (plan shots, prompts, ComfyUI packs, vision health)."
version: 0.1.1
metadata:
  hermes:
    tags: [slate, film, comfyui, prompts, mcp, ollama, vision]
    category: media
---

# Slate Film Factory

## Identity
- Engine: `cargo run -p slate-engine -- mcp` (or installed `slate-engine mcp`)
- Comfy API default: http://127.0.0.1:8188
- Vision judge (preferred): **qwen3.5:9b** via Ollama — weights not bundled (`ollama pull qwen3.5:9b`)

## When to use
- Non-pro: plain-language scene → shots + local generations into a Slate project
- Multi-shot continuity / prompt bible
- Agent/automation without the Electron UI

## When NOT to use
| Signal | Route |
|--------|--------|
| LTX/Wan packs, music video CLI, Video Buddy outputs | video-buddy / master-agent |
| HyperFrames / brand package | forge profile |
| No Comfy and no desire to install | plan/prompts only; generation needs packs |

## Preflight
1. Start ComfyUI API on 8188 if generation is required
2. One GPU owner only — do not stack heavy Comfy jobs
3. `slate_health` — engine ok; for generation, Comfy ok + brain
4. For vision gate readiness: `vision.ready` true. Default model **qwen3.5:9b** (override `SLATE_JUDGE_MODEL`). If missing: `ollama pull qwen3.5:9b`
5. Tool timeout for `slate_film_factory`: ≥ 900s (1800s slow GPU)

## Primary tool
`slate_film_factory` { "brief": "…", "pack_id": "default-still", "shot_count": 4 }

Synchronous: blocks until the run finishes.

Dry-run: `SLATE_DRY_RUN=1` on the engine process.

## Other tools
slate_health, slate_status, slate_cancel, slate_list_projects, slate_get_project,
slate_list_takes, slate_generate_shot, slate_judge_take, **slate_first_ad**

After generate, the engine runs a **quality gate** (Ollama VL). Failures auto-retry with seed + prompt pickups until max retries. Check `quality` / `attempts` on shot results.

**First AD:** `slate_first_ad` `{ "projectId", "message", "history"? }` plans/mutates the project and returns a continuity book + receipts. Factory generate accumulates continuity across shots for the judge.

**Atomic Notes:** `slate_note_write` / `slate_note_search` — project memory under `.notes/notes.jsonl` (continuity, quality, scene plans). Auto-written on generate/First AD.

**Packs / music:** `slate_list_packs`, `slate_run_pack` (any pack). `default-still` is Flux stills; `default-video` is LTX 2.3 distilled T2V (clamped 768 long-edge, 49 frames). `slate_compile_music` compiles project cues to Suno/generic text (no audio render). Electron Agent dock: **Run brief** + **Compile cues**.

## Register (Hermes / MCP)
```bash
hermes mcp add slate -- slate-engine mcp
# or absolute path to target/release/slate-engine
```
