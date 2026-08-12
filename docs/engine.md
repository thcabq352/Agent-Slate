# slate-engine (optional headless film factory)

`slate-engine` plans a one-scene shoot from a plain-language brief, writes sectioned prompts, compiles for ComfyUI packs, and can queue local ComfyUI generation. Agents use stdio MCP or localhost HTTP.

## Build & run

```bash
cargo build -p slate-engine --release
cargo test --workspace
cargo run -p slate-engine -- mcp
cargo run -p slate-engine -- serve
```

## Environment

| Variable | Meaning |
|----------|---------|
| `SLATE_DATA_DIR` | Project root (default: Documents/Slate) |
| `SLATE_COMFY_URL` | Comfy base URL (default `http://127.0.0.1:8188`) |
| `SLATE_PACKS_DIR` | Workflow packs directory |
| `SLATE_BRAIN` | `local` \| `claude` \| `codex` |
| `SLATE_DRY_RUN` | `1` = stub plan + dry-run takes |
| `SLATE_JUDGE_MODEL` | VL judge tag (default **`qwen3.5:9b`**). **Not bundled** — install with Ollama |
| `SLATE_JUDGE_ENDPOINT` | OpenAI-compat base for judge (default `http://127.0.0.1:11434/v1`) |
| `SLATE_JUDGE_PASS_THRESHOLD` | Auto-accept mean score 0–1 (default `0.7`) |
| `SLATE_JUDGE_MAX_RETRIES` | Max auto retries after reject (default `2`) |

## Vision / quality gate (Phase 0–1)

`slate_health` returns:

- `vision` — Ollama-first VL resolution for the quality judge  
  - Preferred: **`qwen3.5:9b`**  
  - Fallbacks if missing: `qwen3-vl:8b`, `qwen3-vl:30b`, `qwen3.6:35b`, `llava`, then heuristic VL-ish names  
  - `ready: true` only when endpoint is up **and** a model was selected  
  - `hint` explains how to fix when not ready  
- `qualityGate` — pass threshold, max retries, configured model/endpoint  

Install weights yourself (never shipped in the app package):

```bash
ollama pull qwen3.5:9b
```

Image inputs for local brains already use OpenAI multimodal `image_url` (base64) on `/v1/chat/completions` (works with Ollama VL models).

## Tools (MCP / HTTP)

| Tool | Notes |
|------|--------|
| `slate_health` | Engine + Comfy + brains + **vision/judge** |
| `slate_film_factory` | Synchronous one-scene pipeline (generate + quality gate) |
| `slate_generate_shot` | Re-roll one shot with quality-gate retries |
| `slate_judge_take` | Score a media file only (`mediaPath`, optional `prompt` / `continuity`) |
| `slate_first_ad` | First AD turn: plan/mutate project (`projectId`, `message`, `history?`) + continuity book |
| `slate_list_projects` / `slate_get_project` / `slate_list_takes` | Store |
| `slate_status` / `slate_cancel` | Job control (+ continuitySummary, scenePlan, lastShotId) |

### Scene continuity (Phase 3)

During `slate_film_factory` generate, a **scene continuity book** accumulates:

- bible locks (cast wardrobe, location weather/time)
- per-shot beats (intent, take path, quality pass/fail)
- handoff line into the next shot
- standing orders from quality issues

That book is passed into the quality judge and appears on `slate_status` as `continuitySummary` / `scenePlan`.

### Quality gate loop (Phase 2)

After each Comfy take:

1. VL judge (`qwen3.5:9b` preferred) scores visual quality, continuity, artifacts, prompt fidelity.
2. If **mean ≥ `SLATE_JUDGE_PASS_THRESHOLD`** (default 0.7) → accept (take rating good/circled).
3. Else apply `retry_hints` into a prompt pickup, new seed, regenerate (up to `SLATE_JUDGE_MAX_RETRIES`).
4. Dry-run / missing VL / `.txt` takes → gate **skipped** (take kept, `quality` may still be present as skip).

Verdict is stored on the take `notes` line and returned on shot outcomes as `quality` + `attempts`.

## Comfy packs

See `workflows/packs/`. Default still pack targets Flux.1-dev fp8 on a local Comfy install.
