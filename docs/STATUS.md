# Status — 2026-08-13 · v0.3.2

Canonical snapshot of **Agent-Slate** ([thcabq352/Agent-Slate](https://github.com/thcabq352/Agent-Slate)). How to install, click, and follow flows: [GUIDE.md](GUIDE.md). Agents in-repo: [AGENTS.json](../AGENTS.json). Historical design lives in [the spec](superpowers/specs/2026-08-11-slate-rust-agent-film-factory-design.md) and [the plan](superpowers/plans/2026-08-12-slate-rust-film-factory.md); this file is what is **shipped**.

## What Agent-Slate is

Upstream Slate is a **prompt studio** (craft → copy → paste into a generator). Agent-Slate keeps that studio and adds an optional **local film factory**:

- `slate-engine` (Rust) owns brief → project → prompts → ComfyUI packs → takes
- **Hermes / Cursor / Grok** are agent fronts (stdio MCP, **blocking** `slate_film_factory`)
- **Electron ◆ Agent dock** is the human front (HTTP `serve`, **background** factory + `slate_status`)

No API keys for the brain, Comfy, or Grok VO (`grok login` OAuth). Local Comfy on `http://127.0.0.1:8188`. First run on Windows: README clip [`docs/images/first-run.mp4`](../docs/images/first-run.mp4).

## V1 — shipped

| Piece | State |
|-------|--------|
| One scene, 4–8 shots | Yes |
| Brains `local` / `cursor` / `grok-4.5` / `grok-4.6` / `codex` | Yes (Grok Build OAuth first for Grok; Cursor OAuth fallback + Composer; Claude Code removed) |
| Windows / macOS / Linux | From-source `install.ps1` / `install.sh`; `npm run package:desktop` |
| `default-still` Flux.1-dev fp8 | Live on this host |
| `default-video` LTX 2.3 distilled T2V | Live — frames from shot duration × fps (8n+1); output node `90` |
| Quality gate (Ollama VL, preferred `qwen3.5:9b`) | Yes; soft-skip on bad JSON / dry-run / `.txt` |
| ✦ First AD (titlebar, studio planner) | Yes — no Comfy |
| Factory AD (`slate_first_ad`) + scene continuity book | Yes — dock / MCP, for generates |
| Atomic notes (`.notes/notes.jsonl`) | Yes |
| Agent dock (connect, Factory AD, judge, retry, **Run brief**, **Compile cues**, **Assemble cut**, **Approve** → circled take) | Yes |
| `take.mediaPath` | Yes (notes path is fallback) |
| `slate_cancel` | Flag **and** Comfy `POST /interrupt` + queue clear |
| Coverage LLM parse | Tolerant (`coverage`/`title`/`purpose`, string JSON, shot maps) |
| Music | Compile-only (Suno/generic text; no audio render) |
| Grok VO | Electron Studios → Sound → Voices **Speak with Grok** (`POST api.x.ai/v1/tts`, `grok login` OAuth) |
| `default-i2v` | LTX I2V — factory makes/reuses a Flux keyframe then animates |
| `default-flf2v` | LTX first+last frame (`LTXVAddGuide` 0 / −1) |
| Video VL judge | First frame extracted with ffmpeg |
| `slate_assemble` | Concat takes → `{project}/cut/slate_cut.mp4` |
| README first-run clip | [`docs/images/first-run.mp4`](../docs/images/first-run.mp4) — Windows, 15s |
| Shareable skill | Hermes Skills Hub: `skills/slate-film-factory/` (SKILL.md + `references/`) · `npm run share:skill` → `share/slate-film-factory.zip` |
| App / crate version | **0.3.2** (`package.json`, Cargo workspace, MCP `serverInfo`) |

See [CHANGELOG-FORK.md](CHANGELOG-FORK.md) for the fork delta vs upstream.

## How to run it

```bash
# Engine (dock + agents) — Windows binary is slate-engine.exe
cargo build -p slate-engine
cargo run -p slate-engine -- serve   # Electron ◆ Agent → Connect
cargo run -p slate-engine -- mcp     # Hermes / Cursor / Grok

# UI (Windows, macOS, Linux)
npm ci && npm run dev
# or:  powershell -ExecutionPolicy Bypass -File .\install.ps1 -Grok
#      ./install.sh --grok
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
| `default-i2v` | Same stack + `LTXVImgToVideo` | + `image` → VHS_LoadImagePath `8` | Factory keyframe via `default-still` if no still exists |
| `default-flf2v` | `LTXVAddGuide` start/end + crop | `image` + `image_end` | End frame = next shot still or same keyframe |

`slate_list_packs` → `ready: false` only if the graph still contains `PLACEHOLDER` / `ALIGN_ME`.

Live video test (opt-in, slow):

```bash
SLATE_LIVE_VIDEO=1 cargo test -p slate-comfy --test live_ltx_video -- --ignored --nocapture
```

## Not next / not shipped

- Multi-scene factory
- Music **audio** render
- 4-shot live T2V/I2V factory in one sitting (VRAM / time — stills first, then one I2V hero)
- IC-LoRA / lipsync / Video Buddy movie-builder graphs

Operator docs: [engine.md](engine.md) · skill: [`skills/slate-film-factory/SKILL.md`](../skills/slate-film-factory/SKILL.md) · zip: [`share/slate-film-factory.zip`](../share/slate-film-factory.zip) · packs: [`workflows/packs/README.md`](../workflows/packs/README.md).
