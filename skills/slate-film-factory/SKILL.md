---
name: slate-film-factory
description: "One-prompt film factory via slate-engine MCP — brief to shots, Flux stills, LTX I2V/FLF2V/T2V, quality gate, assemble cut."
version: 0.3.0
metadata:
  hermes:
    tags: [slate, film, comfyui, prompts, mcp, ollama, vision, ltx, i2v, flf2v]
    category: media
---

# Slate Film Factory

Shipped snapshot: `docs/STATUS.md` in the Slate repo. Install: `INSTALL.md` in this folder.

## Identity
- Engine: `slate-engine mcp` (stdio). Electron dock uses `slate-engine serve`.
- Comfy: `http://127.0.0.1:8188`
- VL judge: **qwen3.5:9b** via Ollama (not bundled)

## When to use
- Plain-language scene → 4–8 shots + local takes
- Still → motion (`default-i2v`) or first/last frame (`default-flf2v`)
- Assemble circled/all takes into one cut

## When NOT to use
| Signal | Route |
|--------|--------|
| Complex lipsync / IC-LoRA / movie-builder graphs | video-buddy / master-agent |
| HyperFrames / brand package | forge |
| No Comfy | `SLATE_DRY_RUN=1` (plan only) |

## Preflight
1. Comfy on 8188 if generating
2. One GPU owner
3. `slate_health`
4. `ollama pull qwen3.5:9b` if judging
5. **Blocking** `slate_film_factory` timeout **1800s** (never `background: true` from Hermes)

## Primary call

```
slate_film_factory { "brief": "…", "pack_id": "default-still", "shot_count": 4 }
```

| pack_id | What happens |
|---------|----------------|
| `default-still` | Flux keyframes (best for a full 4-shot factory) |
| `default-i2v` | Flux keyframe (or last still) → LTX I2V ~2s |
| `default-flf2v` | Start still + next-shot still (or same) → LTX FLF2V |
| `default-video` | LTX T2V from text only |

I2V/FLF2V/T2V are **one short clip per shot**. Prefer stills for coverage, then `slate_generate_shot` + `default-i2v` on the hero.

## Other tools
`slate_health`, `slate_status`, `slate_cancel` (Comfy `/interrupt`), `slate_list_packs`, `slate_run_pack` (`image`, `image_end` for I2V/FLF), `slate_generate_shot`, `slate_judge_take` (mp4 → first frame), `slate_assemble` `{ projectId, circledOnly? }`, `slate_first_ad`, `slate_note_*`, `slate_compile_music` (text only), `slate_list_takes`.

Dry-run: `SLATE_DRY_RUN=1`.
