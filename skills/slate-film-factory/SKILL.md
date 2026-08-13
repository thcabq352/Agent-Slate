---
name: slate-film-factory
description: "Generate a one-scene Comfy film factory via slate-engine."
version: 0.3.2
author: thcabq352, Hermes Agent
license: Apache-2.0
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [film, comfyui, mcp, ltx, flux, slate]
    category: media
    related_skills: [video-buddy]
    homepage: https://github.com/thcabq352/Agent-Slate
---

# Agent-Slate Film Factory

Turns a plain-language brief into one scene (4–8 shots), local Comfy takes, a VL quality gate, and an optional cut. Does **not** do lipsync, IC-LoRA, or multi-scene movies — route those to Video Buddy / master-agent. No API keys; brains are Grok Build, Cursor, Codex, or local.

## When to Use

- User wants a **local film factory**: brief → shots → Flux/LTX takes on disk
- Mentions **Agent-Slate**, `slate-engine`, `slate_film_factory`, Factory AD, or Comfy packs `default-still` / `default-i2v` / `default-flf2v` / `default-video`
- Needs an assembled cut of circled or all takes

**Don't use for:** studio-only prompt compile (no Comfy) — that's Electron Agent-Slate, not this skill. Don't use for HyperFrames / brand packages.

## Prerequisites

1. Built `slate-engine` binary (`cargo build -p slate-engine` from the Agent-Slate repo). Windows: `slate-engine.exe`.
2. MCP server registered as **`slate`** via `hermes mcp add` (see `references/install.md`). Tool timeout **1800s**.
3. ComfyUI at `http://127.0.0.1:8188` unless `SLATE_DRY_RUN=1`.
4. One GPU owner — do not stack Video Buddy heavy jobs with this factory.
5. Optional judge: Ollama `qwen3.5:9b` (`ollama pull qwen3.5:9b`).

Load install detail with `skill_view("slate-film-factory", "references/install.md")`.

## How to Run

Call MCP tools on server **`slate`**. Factory from an agent is **blocking** — never pass `background: true` (that flag is Electron dock only).

Canonical generate:

```
slate_film_factory { "brief": "…", "pack_id": "default-still", "shot_count": 4 }
```

Completion: tool returns `ok` with a `projectId` and take paths, or a clear error (Comfy down, brain missing, timeout).

## Quick Reference

| Tool | Purpose |
|------|---------|
| `slate_health` | Comfy, brain, ffmpeg, packs, VL |
| `slate_film_factory` | Brief → project + generates (blocking, 1800s) |
| `slate_generate_shot` | Retry one shot |
| `slate_judge_take` | VL gate (mp4 → first frame) |
| `slate_first_ad` | **Factory AD** chat (not titlebar First AD) |
| `slate_circle_take` | Circle a take |
| `slate_assemble` | Concat → `{project}/cut/slate_cut.mp4` |
| `slate_cancel` | Stop job + Comfy `/interrupt` |
| `slate_status` | Poll (dock background jobs only) |
| `slate_list_packs` / `slate_run_pack` | Pack inventory / one graph |
| `slate_compile_music` | Cue **text** only |

Packs: `default-still` (Flux, best first factory) · `default-i2v` · `default-flf2v` · `default-video` (LTX shorts). Full tool args: `skill_view("slate-film-factory", "references/tools.md")`.

## Procedure

1. `slate_health` — Comfy ok (or dry-run), at least one brain, `packsOk` true. Stop if not.
2. Prefer `pack_id: "default-still"` and `shot_count` 4–8. Call **blocking** `slate_film_factory`. Do not set `background`.
3. On success, report `projectId` and take `mediaPath`s. Stills first; then `slate_generate_shot` + `default-i2v` on the hero if motion is needed.
4. Optional: `slate_circle_take` keepers, then `slate_assemble` `{ "projectId", "circledOnly": true }`.
5. If the user talks continuity / scene plan for generates, use `slate_first_ad` — separate from the studio titlebar First AD.

## Pitfalls

- Default MCP timeouts (~300s) kill a live factory. Confirm 1800s with `hermes mcp test slate`.
- `background: true` from Hermes leaves the job running with no blocking result — never do that here.
- I2V/FLF2V/T2V are **one short clip per shot**. Don't promise a 4-shot live LTX movie in one sitting.
- Never spawn a binary named `agent` (Grok and Cursor collide). Engine binary is `slate-engine` / `slate-engine.exe`.
- Weights are not in this skill. Missing checkpoints → re-align pack `workflow.api.json` on that Comfy.

## Verification

- [ ] `slate_health` shows engine + packs; Comfy ok or dry-run
- [ ] `slate_film_factory` returns a project with 4–8 shots without `background`
- [ ] Takes exist on disk (`mediaPath`) or dry-run stubs
- [ ] `slate_assemble` writes `cut/slate_cut.mp4` when asked
- [ ] MCP server `slate` was registered with `hermes mcp add` (not a hand-edited config file)
