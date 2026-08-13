# Slate Rust Agent + One-Prompt Film Factory — Design Spec

**Date:** 2026-08-11  
**Status:** **Implemented (V1 shipped 2026-08-12).** This file is the original design. Current operator snapshot: [`docs/STATUS.md`](../../STATUS.md).  
**Repo:** [thcabq352/Agent-Slate](https://github.com/thcabq352/Agent-Slate) (Apache-2.0 — see NOTICE)  
**Approach:** A — Rust workflow engine beside Electron; Hermes Gateway as primary non-pro front  

**Shipped vs this draft (2026-08-13, v0.3.2):** Do **not** implement Claude Code or `control.json` from this file. Brains are **cursor / grok-4.5 / grok-4.6 / codex / local**. Descriptors are `electron-control.json` / `engine-control.json`. `default-still`, `default-video`, **`default-i2v`**, **`default-flf2v`** are live. Electron Agent dock exists (spec PR8) with Run brief / Compile cues / Assemble cut. Hermes uses **blocking** `slate_film_factory` (timeout 1800s). Dock uses `background: true`. Video VL judge extracts a first frame. `slate_assemble` writes a cut. Not shipped: multi-scene, music audio render, IC-LoRA/lipsync.

---

## 1. Problem

Slate already contains a full AI-filmmaking pre-production pipeline (project bible, coverage, First AD actions, sectioned prompts, model compile, control HTTP, MCP bridge). Those steps are **pro-facing** and mostly **UI-bound**. Non-professionals cannot operate the workflow without learning shot language, model dialects, and the Electron UI.

Generation is also incomplete as a product loop: ComfyUI exists only as a **prompt dialect** in `model-profiles.json`, not as a live generation backplane. The user wants **local ComfyUI API workflows** so every provider can be reached by swapping workflow packs, without baking vendor SDKs into Slate.

## 2. Goals

### Product goals

1. **One-prompt film factory for non-pros** — user describes a scene in plain English; system produces a one-scene project with shots, prompts, and generated media.
2. **Rust owns the workflow engine** — plan, mutate project, call brains, queue ComfyUI, return structured results.
3. **Hermes Gateway is the primary non-pro front** — conversation + tool calls (same pattern as Video Buddy `master-agent`).
4. **Local ComfyUI required** — default base URL `http://127.0.0.1:8188` (aligned with Video Buddy portable Comfy); documented shared-GPU guidance.
5. **V1 success:** one scene end-to-end (brief → bible → coverage → prompts → compile → Comfy generate → takes attached).

### Non-goals (V1)

- Full multi-scene feature films
- Bundling/installing ComfyUI or models
- Replacing every existing pro Electron panel
- Direct cloud provider APIs outside Comfy workflow packs
- Embedding Hermes gateway process inside slate-engine

## 3. Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Architecture | Rust engine beside Electron | Ships film factory without greenfield rewrite of all UI |
| Non-pro front | Hermes Gateway primary | User already runs Hermes; MCP tools match Agent OS |
| Generation | Local ComfyUI API workflows only | Universal provider routing via packs; no keys in Slate |
| Comfy URL default | `http://127.0.0.1:8188` | Match Video Buddy; document exclusive GPU use |
| V1 scope | One scene, 4–8 shots, end-to-end | Smallest complete loop |
| `slate_film_factory` | **Synchronous** MCP tool | Hermes-friendly single tool call; long timeout (~900s+) |
| Brain adapters V1 | Local OpenAI-compat **+ Claude Code + Codex** | Match current Slate brain model; no API keys stored |
| Project model | Keep Slate JSON shape (versioned if needed) | Reuse domain; Electron Advanced can read same files later |
| Electron in V1 | Optional thin client / status UI | Not required for non-pro path |

## 4. Architecture

```
Non-pro user
    │
    ▼
Hermes Gateway (chat / channels / profiles / skills)
    │ MCP (stdio or configured transport)
    ▼
slate-engine (Rust process)
    ├── Workflow runner (film_factory graph)
    ├── Project store (~/Documents/Slate or SLATE_DATA_DIR)
    ├── Brain adapters (claude CLI, codex CLI, local /v1)
    ├── Domain steps (intake, bible, coverage, prompts, compile)
    └── Comfy client → http://127.0.0.1:8188
            │
            ▼
     Local ComfyUI + workflow packs under workflows/packs/

Optional secondary:
  Electron Director UI / CLI  →  same HTTP API as MCP tools
```

### Process boundaries

| Component | Role |
|-----------|------|
| **Hermes Gateway** | Conversational front; discovers and calls Slate MCP tools; long tool timeouts |
| **slate-engine** | Source of truth for projects, jobs, workflow execution, brain, Comfy |
| **ComfyUI** | Pixel generation only |
| **Electron app** | Secondary UI; may spawn or attach to engine later; V1 can be deferred for non-pro path |
| **Hermes skill** | `skills/slate-film-factory` (or under `~/.hermes/skills/media/`) — when to use, health preflight, tool list, GPU warning |

### Target repo layout

```
slate/
  crates/
    slate-engine/      # binary: HTTP + MCP server, config, runner
    slate-domain/      # Project/Scene/Shot types, actions, compile rules
    slate-brain/       # claude / codex / local adapters
    slate-comfy/       # Comfy HTTP client + template inject
  workflows/
    packs/
      default-still/
        workflow.api.json
        manifest.json
      default-video/
        workflow.api.json
        manifest.json
  skills/
    slate-film-factory/
      SKILL.md         # Hermes skill: preflight, tools, GPU note
  app/                 # existing Electron/React (becomes client over time)
  mcp/                 # legacy TS bridge; replaced by engine-native MCP
  docs/superpowers/specs/
```

## 5. V1 film-factory pipeline

### Input

Plain-language brief, e.g.:

> A courier runs across rainy neon rooftops at night; cinematic chase, about 8 seconds.

### Output

- One `Project` with one `Scene`
- 4–8 `Shot`s with sectioned prompts + specs
- Compiled positive/negative (and pack params) per shot
- Generated media files under project takes
- Structured JSON result for Hermes to summarize

### Step graph

| Step | Node | Behavior |
|------|------|----------|
| 0 | `health` | Engine OK; Comfy reachable at configured URL (default 8188); at least one brain available |
| 1 | `intake` | LLM → structured `SceneBrief` (logline, world hints, shot_count, duration_sec, pack_id, aspect) |
| 2 | `bible` | Create project; set logline/world; add 1–2 characters + 1 location via First-AD-style actions |
| 3 | `coverage` | Plan 4–8 shots with name, intent, spec (size, movement, duration, angle) |
| 4 | `prompts` | Write sectioned prompts per shot; continuity from bible |
| 5 | `compile` | Map each shot to pack inputs (positive/negative/size) using Comfy compile dialect + pack limits |
| 6 | `generate` | For each shot: inject pack template → Comfy `/prompt` → poll history → download → attach take |
| 7 | `review` | Build result: projectId, sceneId, shots, take paths, receipts, failures |

### Defaults (non-pro)

| Setting | Default |
|---------|---------|
| Shots | 6 (or from brief, clamped 4–8) |
| Duration | 8s for video packs; N/A for stills |
| Aspect | 16:9 |
| Pack | `default-video` if healthy, else `default-still` |
| Brain | From engine config: prefer configured backend; detect claude/codex/local like current Slate |
| Continuity | Soft check after prompts; non-blocking warnings in receipts |

### Synchronous Hermes contract

- **`slate_film_factory`** runs steps 0–7 **in one MCP tool call** and returns only when the run finishes or fails fatally.
- Recommended Hermes tool timeout: **≥ 900s** (generation-dominated); document 1800s for slow GPUs.
- Internal job id still exists for logging, cancel, and optional `slate_status` mid-flight if the gateway supports concurrent tools; **primary path remains blocking**.

### Failure policy

| Failure | Behavior |
|---------|----------|
| Comfy down at health | Fail fast with start instructions (`run_api_8188.bat` / open Comfy on 8188) |
| No brain | Fail fast with install/login or local server instructions |
| Brain error mid-run | Retry once on the step; then fail with **partial project saved** |
| Single shot generation fails | Mark shot failed; continue remaining shots |
| Cancel | `slate_cancel` aborts Comfy queue item if possible; keep finished takes |

## 6. ComfyUI packs (universal providers)

### Principle

Every generation target is a **checked-in ComfyUI API-format workflow** + **manifest** mapping logical fields → node ids. Adding a provider = adding a pack, not new Rust vendor code.

### Default endpoint

```yaml
# engine config
comfy:
  base_url: "http://127.0.0.1:8188"
```

Document in Hermes skill:

- Aligns with Video Buddy portable Comfy (`run_api_8188.bat`).
- **Do not** stack heavy Slate generation with Video Buddy or other Comfy jobs on the same GPU.
- Studio UI ports (e.g. 8189) are **not** the API; generation requires the Comfy API port.

### Pack layout

```
workflows/packs/<pack_id>/
  workflow.api.json    # Comfy "Save (API Format)"
  manifest.json        # injection map + limits
```

### Manifest schema (normative for V1)

```json
{
  "id": "default-still",
  "label": "Default local still",
  "modality": "image",
  "inputs": {
    "positive": { "node_id": "6", "field": "text" },
    "negative": { "node_id": "7", "field": "text" },
    "width": { "node_id": "5", "field": "width" },
    "height": { "node_id": "5", "field": "height" },
    "seed": { "node_id": "3", "field": "seed", "mode": "randomize" }
  },
  "outputs": {
    "media": { "node_id": "9", "type": "image" }
  },
  "limits": {
    "aspect_ratios": ["16:9", "9:16", "1:1"],
    "max_duration_sec": null
  },
  "compile_profile": "comfyui"
}
```

### Engine generate algorithm

1. Clone `workflow.api.json`
2. Apply manifest injections from compiled shot payload
3. `POST {base_url}/prompt` with `{ "prompt": graph, "client_id": "..." }`
4. Poll `{base_url}/history/{prompt_id}` until done or timeout
5. Resolve output filenames; `GET /view?...` download
6. Store under `{projectDir}/takes/{shotId}/{timestamp}.{ext}`
7. Append take metadata on shot (path, pack id, prompt used, createdAt)

### V1 packs

| Pack | Required |
|------|----------|
| `default-still` | **Yes** (smoke path) |
| `default-video` | **Yes** if a known-good graph exists on the machine; otherwise document as 1.1 with still-only factory flag |

Vendor/API packs (Flux cloud nodes, etc.) are post-V1: same manifest pattern.

## 7. Data model

### Persist compatibility

Keep the existing Slate `Project` JSON shape as the baseline (`src/shared/types.ts` as reference):

- `Project` → `scenes[]` → `shots[]` with `prompt`, `spec`, `history`, `takes`
- Bible: `characters`, `locations`, `lookbook`, `artDept`, defaults
- New fields allowed behind `schemaVersion` if needed:
  - `defaults.comfyPackId`
  - take: `packId`, `comfyPromptId`, `mediaPath`

Projects remain under:

- Default: `~/Documents/Slate/<projectId>/project.json`
- Override: `SLATE_DATA_DIR`

### First AD action contract

Port `AdAction` / `applyAdActions` semantics from `src/renderer/src/lib/firstAD.ts` into `slate-domain` so bible/coverage/prompts mutate projects identically. Receipts stay human-readable strings for Hermes replies.

### SceneBrief (intake output)

```json
{
  "title": "string",
  "logline": "string",
  "world": "string",
  "shot_count": 6,
  "duration_sec": 8,
  "aspect_ratio": "16:9",
  "pack_id": "default-still",
  "characters": [{ "name": "…", "one_liner": "…" }],
  "location": { "name": "…", "description": "…" },
  "style_notes": "string"
}
```

## 8. Brain adapters (V1 includes all three)

Port behavior from `src/main/brain.ts`:

| Backend | Mechanism | Notes |
|---------|-----------|--------|
| `local` | OpenAI-compatible `POST /v1/chat/completions` | Probe common ports; config override |
| `claude` | `claude -p --output-format json` | PATH resolution; auth errors → clear message |
| `codex` | `codex exec --output-last-message <file>` | Prefer ChatGPT.app bundled codex on macOS when present |

Config:

```yaml
brain:
  default: local   # or claude | codex
  local:
    endpoint: null  # auto-detect
    model: null
  tier_map:
    fast: haiku      # claude only
    standard: sonnet
    top: null
```

Film-factory steps use:

- `intake`, `bible`, `coverage`, `prompts`: tier `top` or `standard` as appropriate
- Mechanical compile reshaping: `standard`
- `expect_json` with extract-JSON + one retry nudge (same as current brain)

**No API keys stored in Slate** for Claude/Codex; local models never leave the machine.

## 9. Engine API and MCP tools

### Transport

- **HTTP:** `127.0.0.1` only by default; bearer token in `~/.config/slate/control.json` (Windows: `%APPDATA%/slate/control.json`) — same idea as current `control.ts`
- **MCP:** engine binary mode `slate-engine mcp` (stdio) for Hermes registration

### Tools (V1)

| Tool | Sync | Description |
|------|------|-------------|
| `slate_health` | yes | Engine, Comfy (`:8188`), brains |
| `slate_film_factory` | **yes (blocking)** | Full pipeline from brief; optional `pack_id`, `brain` |
| `slate_status` | yes | Optional mid-run / last job status |
| `slate_cancel` | yes | Cancel active factory/generation |
| `slate_list_projects` | yes | List project metas |
| `slate_get_project` | yes | Full project JSON |
| `slate_list_takes` | yes | Media for a project/shot |
| `slate_generate_shot` | **yes (blocking)** | Re-roll one shot through Comfy pack |

### `slate_film_factory` args

```json
{
  "brief": "string (required)",
  "pack_id": "optional pack override",
  "brain": "optional claude|codex|local",
  "shot_count": "optional 4-8",
  "project_name": "optional"
}
```

### `slate_film_factory` result

```json
{
  "ok": true,
  "projectId": "…",
  "sceneId": "…",
  "shots": [
    {
      "id": "…",
      "name": "…",
      "prompt": "…",
      "takePath": "… or null",
      "error": "… or null"
    }
  ],
  "receipts": ["✓ …"],
  "warnings": ["…"],
  "elapsedMs": 0
}
```

## 10. Hermes skill surface

Ship `skills/slate-film-factory/SKILL.md` (installable to `~/.hermes/skills/…` and/or agents skills hub):

- **When to use:** non-pro one-prompt scene; multi-shot continuity; Comfy generation into a Slate project
- **When not to use:** pure Video Buddy LTX packs / music video CLI; HyperFrames marketing; Grok cloud ≤15s
- **Preflight:** `slate_health`; start Comfy on 8188 if down; pick one GPU owner
- **Primary call:** `slate_film_factory` with user brief; long timeout
- **Iteration:** `slate_generate_shot`, `slate_list_takes`, `slate_get_project`
- **Registration:** `hermes mcp add slate -- slate-engine mcp` (exact command finalized at implement)

## 11. Electron / existing app

V1 non-pro path **does not require** Electron. Migration path:

1. Engine implements project store + MCP  
2. Hermes skill works  
3. Later: Electron preload calls engine HTTP instead of local IPC for projects/brain  
4. “Pro mode” retains advanced panels once they are clients of the engine  

Legacy `mcp/slate-mcp.mjs` is superseded by engine-native MCP once engine is default.

## 12. Error handling and observability

- Structured logs: step name, project id, elapsed ms, Comfy prompt_id  
- User-facing errors: actionable (auth login, start Comfy, load model)  
- Partial success: `ok: false` or `ok: true` with per-shot errors — prefer `ok: true` if ≥1 take produced, with `warnings`  
- Never leave corrupt `project.json` (atomic write: temp + rename, as today)

## 13. Testing strategy

| Layer | Tests |
|-------|--------|
| `slate-domain` | applyAdActions, project serde, compile preflight pure functions |
| `slate-comfy` | manifest inject unit tests with fixture workflow JSON; mock HTTP |
| `slate-brain` | extract_json; mock local server; CLI build args (no live network in CI) |
| `slate-engine` | health + film_factory dry-run mode (`SLATE_DRY_RUN=1` skips Comfy GPU) |
| Integration (manual) | Hermes `mcp test slate`; one still factory on real Comfy |

## 14. Security

- Bind HTTP to loopback only by default  
- Per-session bearer token for HTTP  
- MCP stdio inherits user permissions (local machine)  
- Do not log full bearer tokens or raw auth material  
- Comfy URL configurable but default loopback

## 15. Implementation phases (PR-oriented)

See **PR Plan** below. Ordering principle: domain + health first, then brain, then Comfy, then film_factory, then Hermes skill, then optional UI.

## 16. Open questions (resolved during brainstorming)

| Question | Resolution |
|----------|------------|
| Product shape | One-prompt film factory |
| Rust scope | Full workflow engine |
| Comfy | Local required, default 8188 |
| V1 scope | One scene E2E |
| Architecture | A — engine beside Electron |
| Hermes | Primary non-pro front |
| film_factory | Synchronous for Hermes |
| Brain V1 | local + Claude + Codex |

### Remaining open (non-blocking for spec)

1. Exact default-still / default-video graphs available on the user’s Comfy install (implement packs against real node ids at pack-author time).  
2. Whether V1 ships a minimal Windows binary path only first (user is on Windows) or cross-platform from day one — **recommendation: Windows-first binaries, keep code portable**.

## 17. Alternatives considered

| Approach | Why not chosen |
|----------|----------------|
| B — Greenfield full Rust rewrite | Too slow for V1 film factory; risks losing working domain |
| C — Comfy-centric agent only | Weak continuity / project bible; bad non-pro film UX |
| Async-only film_factory | Rejected; Hermes primary path needs blocking tool |
| Stills-only forever | Stills pack is required smoke; video pack is V1 if graph exists |
| Direct provider SDKs | Violates Comfy-as-universal-router goal |

---

## Key Decisions (summary)

1. **Rust `slate-engine` is the OS of the pipeline** — UI and Hermes are clients.  
2. **Hermes is the non-pro front** via MCP; skill documents preflight and GPU exclusivity.  
3. **ComfyUI API packs** express all providers; default `http://127.0.0.1:8188`.  
4. **`slate_film_factory` is synchronous** end-to-end for Hermes.  
5. **V1 = one scene E2E** with still pack required and video pack preferred.  
6. **Brain: local + Claude Code + Codex in V1** — no API keys in app.  
7. **Preserve Slate project JSON** and First AD action semantics.  
8. **Electron is secondary** for V1 non-pro success criteria.

---

## PR Plan

### PR1 — Workspace + domain crate
- **Title:** `feat(engine): cargo workspace and slate-domain project model`
- **Affects:** `crates/slate-domain`, project serde, `apply_ad_actions`, fixtures from existing types
- **Deps:** none
- **Description:** Port Project/Shot/AdAction; unit tests for mutations and atomic save helpers

### PR2 — Brain crate
- **Title:** `feat(engine): slate-brain local + claude + codex adapters`
- **Affects:** `crates/slate-brain`
- **Deps:** PR1
- **Description:** BrainRequest/Result; JSON extract; status detection; unit tests with mocks

### PR3 — Comfy crate + default-still pack
- **Title:** `feat(engine): slate-comfy client and default-still pack`
- **Affects:** `crates/slate-comfy`, `workflows/packs/default-still`
- **Deps:** none (can parallel PR1)
- **Description:** Health, inject, prompt, poll, download; default base URL 8188; fixture tests

### PR4 — Engine HTTP + MCP skeleton
- **Title:** `feat(engine): slate-engine binary with health tools and control descriptor`
- **Affects:** `crates/slate-engine`
- **Deps:** PR1–PR3
- **Description:** Loopback HTTP + bearer token; stdio MCP; `slate_health`, list/get project

### PR5 — Film factory pipeline (sync)
- **Title:** `feat(engine): slate_film_factory synchronous one-scene pipeline`
- **Affects:** `crates/slate-engine` workflow runner
- **Deps:** PR2–PR4
- **Description:** Steps 0–7; dry-run mode; `slate_generate_shot`; cancel/status

### PR6 — Hermes skill + docs
- **Title:** `docs(skills): slate-film-factory Hermes skill and README engine section`
- **Affects:** `skills/slate-film-factory/SKILL.md`, README
- **Deps:** PR5
- **Description:** Registration, timeouts, Comfy 8188, GPU exclusivity vs Video Buddy

### PR7 — default-video pack (if graph ready)
- **Title:** `feat(comfy): default-video pack and factory pack selection`
- **Affects:** `workflows/packs/default-video`, compile limits
- **Deps:** PR5
- **Description:** Video modality generate path; fall back to still if pack missing

### PR8 — Optional Electron attach (post non-pro V1)
- **Title:** `feat(app): point preload project/brain calls at slate-engine HTTP`
- **Affects:** `app/` or existing `src/main`, preload
- **Deps:** PR5
- **Description:** Dual-mode: embedded engine child process or attach to running engine

---

## Success criteria (V1 exit)

1. With Comfy on `127.0.0.1:8188` and any one brain available, Hermes can run `slate_film_factory` with a one-line brief and block until result.  
2. Result includes a saved project, ≥4 shots with prompts, and ≥1 take file for a successful still path.  
3. `slate_health` reports Comfy + brain clearly when either is down.  
4. No vendor API keys required in Slate config.  
5. Skill docs state 8188 default and “one GPU owner” rule.
