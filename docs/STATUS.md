# Status — 2026-08-12

Canonical snapshot of the **thcabq352/slate** fork (Agent-Slate). Historical design lives in [the spec](superpowers/specs/2026-08-11-slate-rust-agent-film-factory-design.md) and [the plan](superpowers/plans/2026-08-12-slate-rust-film-factory.md); this file is what is **shipped**.

## What this fork is

Upstream Slate is a **prompt studio** (craft → copy → paste into a generator). This fork keeps that studio and adds an optional **local film factory**:

- `slate-engine` (Rust) owns brief → project → prompts → ComfyUI packs → takes
- **Hermes** is the agent front (stdio MCP, **blocking** `slate_film_factory`)
- **Electron ◆ Agent dock** is the human front (HTTP `serve`, **background** factory + `slate_status`)

No API keys. No model weights in the package. Local Comfy on `http://127.0.0.1:8188`.

## V1 — shipped

| Piece | State |
|-------|--------|
| One scene, 4–8 shots | Yes |
| Brains `local` / `claude` / `codex` | Yes |
| `default-still` Flux.1-dev fp8 | Live on this host |
| `default-video` LTX 2.3 distilled T2V | Live — one-clip smoke **356 KB / 92s** (`slate_video_00001_.mp4`, 768×432, 49 frames @ 24 fps) |
| Quality gate (Ollama VL, preferred `qwen3.5:9b`) | Yes; soft-skip on bad JSON / dry-run / `.txt` |
| First AD + scene continuity book | Yes |
| Atomic notes (`.notes/notes.jsonl`) | Yes |
| Agent dock (connect, AD, judge, retry, **Run brief**, **Compile cues**) | Yes |
| `take.mediaPath` | Yes (notes path is fallback) |
| `slate_cancel` | Flag **and** Comfy `POST /interrupt` + queue clear |
| Coverage LLM parse | Tolerant (`coverage`/`title`/`purpose`, string JSON, shot maps) |
| Music | Compile-only (Suno/generic text; no audio render) |

HEAD around this write-up: `9b708ba` on `main` (see [CHANGELOG-FORK.md](CHANGELOG-FORK.md)).

## How to run it

```bash
# Engine (dock + agents)
cargo build -p slate-engine
cargo run -p slate-engine -- serve   # Electron ◆ Agent → Connect
cargo run -p slate-engine -- mcp     # Hermes / Claude Code

# UI
npm ci && npm run dev
```

**Hermes:** stdio MCP, **do not** pass `background: true`. Tool timeout **1800s** (900s minimum). This machine’s Hermes default + cinegen profiles register:

```yaml
slate:
  command: …/target/debug/slate-engine.exe
  args: [mcp]
  timeout: 1800
```

**Electron Run brief:** `{ "brief", "pack_id", "shot_count", "background": true }` then poll `slate_status`. Opens the new project when the job goes idle.

**Dry-run:** `SLATE_DRY_RUN=1`.

## Packs

| Pack | Graph | Inject | Notes |
|------|--------|--------|--------|
| `default-still` | Flux UNET + DualCLIP + FluxGuidance + EmptySD3 + KSampler 8 | `6/7` text, `27`+`30` size, `3` seed | 1280×720 compile sizes |
| `default-video` | LTX 2.3 distilled T2V + Gemma + distilled LoRA + joint AV | `10/11` text, `20` size, `42` `noise_seed`, optional `frames` | Factory clamps to **768×432** / **432×768**; default **49** frames |

`slate_list_packs` → `ready: false` only if the graph still contains `PLACEHOLDER` / `ALIGN_ME`.

Live video test (opt-in, slow):

```bash
SLATE_LIVE_VIDEO=1 cargo test -p slate-comfy --test live_ltx_video -- --ignored --nocapture
```

## Not next / not shipped

- Flux still → LTX **I2V** pack
- VL judge on **video** (extract a frame; mp4 is not a first-class image)
- Assemble / concat circled takes into a cut
- Multi-scene factory
- Music **audio** render
- 4-shot live T2V factory (VRAM / time; use stills or one-clip `slate_run_pack`)

Operator docs: [engine.md](engine.md) · skill: [`skills/slate-film-factory/SKILL.md`](../skills/slate-film-factory/SKILL.md).
