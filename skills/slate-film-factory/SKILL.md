---
name: slate-film-factory
description: "One-prompt film factory via slate-engine MCP (Hermes primary front)."
version: 0.1.0
metadata:
  hermes:
    tags: [slate, film, comfyui, prompts, mcp]
    category: media
    related_skills: [video-buddy]
---

# Slate Film Factory

## Identity
- Engine: `cargo run -p slate-engine -- mcp` (or installed `slate-engine mcp`)
- Comfy API default: http://127.0.0.1:8188
- Not Video Buddy — Slate owns multi-shot continuity + project bible; Comfy owns pixels

## When to use
- Non-pro: plain-language scene → shots + local generations into a Slate project
- Multi-shot continuity / prompt bible

## When NOT to use
| Signal | Route |
|--------|--------|
| LTX/Wan packs, music video CLI, Video Buddy outputs | video-buddy / master-agent |
| HyperFrames / brand package | forge profile |
| No Comfy and no desire to install | plan-only later; health will fail generate |

## Preflight
1. Start ComfyUI API on 8188 (e.g. Video Buddy `run_api_8188.bat`)
2. One GPU owner only — do not stack Video Buddy + Slate heavy jobs
3. `slate_health` via MCP must show comfy ok + at least one brain
4. Tool timeout for `slate_film_factory`: ≥ 900s (1800s slow GPU)

## Primary tool
`slate_film_factory` { "brief": "…", "pack_id": "default-still", "shot_count": 4 }

Synchronous: blocks until project + takes ready.

## Other tools
slate_health, slate_status, slate_cancel, slate_list_projects, slate_get_project,
slate_list_takes, slate_generate_shot

## Register (Hermes)
hermes mcp add slate -- slate-engine mcp
# or full path to binary
