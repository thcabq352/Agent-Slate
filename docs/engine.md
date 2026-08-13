# slate-engine

**v0.3.2.** `slate-engine` plans a one-scene shoot from a plain-language brief, writes sectioned prompts, compiles for ComfyUI packs, and queues local ComfyUI generation. Agents use **stdio MCP**. The Electron **◆ Agent** dock uses **loopback HTTP**.

Current snapshot: [STATUS.md](STATUS.md). Functions and flows: [GUIDE.md](GUIDE.md). Agent handbook (Windows / macOS / Linux): [AGENTS.json](../AGENTS.json).

## Build & run

```bash
cargo build -p slate-engine                 # debug — what the dock auto-spawns first after release
cargo build -p slate-engine --release
cargo test --workspace
cargo run -p slate-engine -- mcp            # Hermes / Cursor / Grok (blocking tools)
cargo run -p slate-engine -- serve          # Windows: %APPDATA%/slate/engine-control.json
                                            # macOS/Linux: ~/.config/slate/engine-control.json
```

Rebuild after engine changes before **Connect / start engine**. An old `serve` process will keep the previous binary.

## Environment

| Variable | Meaning |
|----------|---------|
| `SLATE_DATA_DIR` | Project root (default: Documents/Slate) |
| `SLATE_COMFY_URL` | Comfy base URL (default `http://127.0.0.1:8188`) |
| `SLATE_PACKS_DIR` | Workflow packs directory. Default: walk up from the **engine binary** for `workflows/packs` (not process cwd). `slate_health` reports `packsDir` / `packsOk`. |
| `SLATE_BRAIN` | `local` \| `cursor` \| `grok-4.5` \| `grok-4.6` \| `codex` |
| `SLATE_DRY_RUN` | `1` = stub plan + dry-run takes |
| `SLATE_JUDGE_MODEL` | VL judge tag (default **`qwen3.5:9b`**). **Not bundled** |
| `SLATE_JUDGE_ENDPOINT` | OpenAI-compat base (default `http://127.0.0.1:11434/v1`) |
| `SLATE_JUDGE_PASS_THRESHOLD` | Auto-accept mean score 0–1 (default `0.7`) |
| `SLATE_JUDGE_MAX_RETRIES` | Max auto retries after reject (default `2`) |
| `SLATE_FFMPEG` | Absolute path to `ffmpeg` / `ffmpeg.exe` (judge, stills, ingest, assemble). Also probes winget/scoop/chocolatey/`C:\ffmpeg\bin`. |
| `CIRCLE_TAKE_RECENTS` | Override path to Circle Take `recents.json` (else Electron userData: `%APPDATA%\circle-take` or `Circle Take`). |
| `SLATE_LIVE_COMFY` / `SLATE_LIVE_VIDEO` | Opt-in live pack smokes (`1`) |

## Hermes vs Electron

| Front | Command | `slate_film_factory` | Timeout |
|-------|---------|----------------------|---------|
| **Hermes / MCP agents** | `slate-engine mcp` | **Blocking** — omit `background` | **1800s** (900s minimum) |
| **◆ Agent dock** | `slate-engine serve` | `{ "background": true }` then poll `slate_status` | N/A (job is async) |

Do **not** teach Hermes `background: true`. That flag exists so the UI does not block IPC for 15+ minutes.

### Control descriptors

Electron and `slate-engine serve` must not share a file. Last writer used to win on `control.json`.

| Writer | File | `app` field | Readers |
|--------|------|-------------|---------|
| Electron studio | `%APPDATA%/slate/electron-control.json` (Unix: `~/.config/slate/`) | `slate-electron` | `mcp/slate-mcp.mjs` |
| `slate-engine serve` | `%APPDATA%/slate/engine-control.json` | `slate-engine` | Agent dock (`engineBridge.ts`) |

Clients ignore a descriptor whose `app` does not match. A leftover `control.json` from older builds is unused. Stdio MCP (`slate-engine mcp`) does not write a descriptor.

Register:

```bash
hermes mcp add slate -- /absolute/path/to/target/debug/slate-engine mcp
# YAML equivalent:
# slate:
#   command: …/slate-engine
#   args: [mcp]
#   timeout: 1800
#   connect_timeout: 120
```

Preflight: `slate_health` (Comfy ok + at least one brain). One GPU owner only — do not stack Video Buddy heavy jobs with Agent-Slate generations.

## Vision / quality gate

`slate_health` returns:

- `ffmpeg` — resolved binary (`ok`, `path`, `hint` if missing). Judge/assemble/stills need it.
- `vision` — Ollama-first VL for the quality judge  
  - Preferred: **`qwen3.5:9b`**  
  - Fallbacks: `qwen3-vl:8b`, `qwen3-vl:30b`, `qwen3.6:35b`, `llava`, then heuristic VL-ish names  
  - `ready: true` only when the endpoint is up **and** a model was selected  
- `qualityGate` — pass threshold, max retries, configured model/endpoint  

```bash
ollama pull qwen3.5:9b
```

After each Comfy take:

1. VL scores visual quality, continuity, artifacts, prompt fidelity.
2. Mean ≥ `SLATE_JUDGE_PASS_THRESHOLD` (0.7) → accept.
3. Else apply `retry_hints`, new seed, regenerate (up to `SLATE_JUDGE_MAX_RETRIES`).
4. Dry-run / missing VL / `.txt` / unreadable judge JSON → gate **skipped** (take kept).

Takes store **`mediaPath`** (absolute file) plus a compact `notes` line with the verdict. `slate_list_takes` prefers `mediaPath`.

Image brains use OpenAI multimodal `image_url` (base64). **Video takes:** ffmpeg grabs the first frame (`*_judge.png` beside the mp4) and that still is what the VL model scores. If ffmpeg is missing, the gate is skipped and the take is kept.

## Tools (MCP / HTTP)

| Tool | Notes |
|------|--------|
| `slate_health` | Engine + Comfy + brains + vision/judge |
| `slate_film_factory` | One-scene pipeline. Required: `brief`. Optional: `pack_id`, `brain`, `shot_count` (4–8), `project_name`, `background` |
| `slate_generate_shot` | Re-roll one shot with quality-gate retries |
| `slate_judge_take` | Score a file (`mediaPath`, optional `prompt` / `continuity`) |
| `slate_first_ad` | **Factory AD** conversational turn (`projectId`, `message`, `history?`). Tool id unchanged. Not the titlebar ✦ First AD. |
| `slate_note_write` / `slate_note_search` | Atomic notes |
| `slate_list_packs` | Pack id, modality, `ready` |
| `slate_run_pack` | Generic generate (`pack_id`, positive/negative/width/height/`frames`/`image`/`image_end`/seed, destDir?) |
| `slate_assemble` | Concat takes → `{project}/cut/slate_cut.mp4` (`circledOnly?`) |
| `slate_compile_music` | Cue → Suno/generic text (`projectId`, `target`, optional `cueId`) |
| `slate_list_projects` / `slate_get_project` / `slate_list_takes` | Store |
| `slate_status` / `slate_cancel` | Job control. Cancel sets the flag **and** POSTs Comfy `/interrupt` + queue clear |

Coverage JSON from the brain may be a bare array, `{ "shots" \| "coverage": [...] }`, a string of JSON, `{ "title", "purpose" }`, or a numbered map. Parse failures fall back to stub shots after one schema retry.

### Atomic notes

`{projectDir}/.notes/notes.jsonl` — kinds: `continuity` · `shot_decision` · `quality_feedback` · `scene_plan` · `general`.

### Electron ◆ Agent dock

With the engine built, click **◆ Agent**:

- Connect / start `slate-engine serve` (`target/release` then `target/debug`)
- Pills: Comfy / VL / ffmpeg / busy step
- **Run brief** — background factory; live `step` / `scenePlan`; opens the new project when idle
- **Assemble cut** — `slate_assemble` → `{project}/cut/slate_cut.mp4` (stills become 2s holds)
- **Compile cues** — music text only
- ✦ **First AD** (titlebar) — studio planner (scenes, shots, prompts). **Factory AD** (Agent dock) — `slate_first_ad` on the engine (continuity + scene plan for generates).
- Judge latest take (`mediaPath` or notes path; mp4 → first frame), **Approve** (writes `rating: circled` on the latest take), Retry shot
- Cancel job — stop between shots **and** interrupt Comfy

### Continuity book

During generate the engine accumulates bible locks, per-shot beats, handoff, and standing orders. Exposed on `slate_status` as `continuitySummary` / `scenePlan` and passed into the VL judge.

## Comfy packs

See `workflows/packs/`.

| Pack | Modality | Live? |
|------|----------|--------|
| `default-still` | image | Yes — Flux.1-dev fp8 |
| `default-video` | video | Yes — LTX 2.3 distilled T2V (factory clamps 768×432 / 432×768; 49 frames @ 24 fps) |
| `default-i2v` | video | Yes — LTX I2V (`LTXVImgToVideo`). Factory makes/reuses a Flux keyframe |
| `default-flf2v` | video | Yes — first+last frame (`LTXVAddGuide` 0 / −1) |

`ready: false` if `workflow.api.json` still contains `PLACEHOLDER` / `ALIGN_ME`. Music is compile-only.

Live LTX **T2V** one-clip on this host: `slate_video_00001_.mp4`, 356 KB, ~92 s (2026-08-12). I2V / FLF2V graphs match live Comfy `object_info` (not yet one-clip smoked).

Shareable skill zip: [`share/slate-film-factory.zip`](../share/slate-film-factory.zip) (`npm run share:skill`). Hub: `hermes skills tap add thcabq352/Agent-Slate` then `hermes skills install slate-film-factory -y --category media`.
