# Fork changelog (thcabq352/Agent-Slate)

Notable changes in this maintained fork relative to upstream Slate. Apache-2.0 attribution for the original work remains with Sam Wasserman; see [NOTICE](../NOTICE) and [ATTRIBUTION.md](ATTRIBUTION.md).

Current snapshot: [STATUS.md](STATUS.md).

## 2026-08-13

- Hermes Skills Hub packaging: `skills/slate-film-factory/` is SKILL.md + `references/` (≤60-char description, no MCP YAML). Install: `hermes skills tap add thcabq352/Agent-Slate` then `hermes skills install slate-film-factory -y --category media`.

- Fork polish vs upstream Slate: **[AGENTS.json](../AGENTS.json)** handbook, `npm run share:skill` zip, Factory AD copy on `slate_first_ad`, lockfile name `agent-slate@0.3.2`, Help/Home vs Wasserman original (macOS + Claude Code).

- Agent handbook is **[AGENTS.json](../AGENTS.json)** (was AGENTS.md).

- Home shows Studio / Brain / Factory paths; in-app Help has **Map & flows**. Canonical handbook: [GUIDE.md](GUIDE.md).

- From-source setup on **Windows, macOS, and Linux**: `install.ps1` / `install.sh` → `scripts/setup.mjs`. `npm run package:desktop` unpacks the Electron app on all three OS. Shared handbook: [GUIDE.md](GUIDE.md) (functions & flows) · [AGENTS.json](../AGENTS.json).

- Grok 4.5 / 4.6 brains prefer **Grok Build OAuth** (`grok login` + `~/.grok/bin/grok`) over Cursor. Composer stays on `cursor-agent login`. Missing Grok session falls back to Cursor OAuth.

- Sound → Voices **Speak with Grok** renders MP3 VO via xAI TTS using **`grok login` OAuth** (`~/.grok/auth.json`) — no API key. Cursor-agent login remains the Composer fallback.

- Windows: ffmpeg is resolved via `SLATE_FFMPEG` plus winget/scoop/chocolatey/`C:\ffmpeg\bin` (Electron + engine). Circle Take recents are read from `%APPDATA%\circle-take` (and `Circle Take`), not only macOS Library. Codex looks under `%LOCALAPPDATA%\Programs\ChatGPT`. Agent dock **Approve** writes `TakeRating::Circled` to `project.json` (`slate_circle_take`).
- Home treats a live local model (Ollama/LM Studio) as ready instead of nagging Cursor/Codex. Help shortcut is **Ctrl+/** on Windows (native caption + Help menu). Agent dock operator is **Factory AD** (`slate_first_ad`); titlebar ✦ First AD stays the studio planner.
- Agent dock: Comfy/Ollama are probed directly (and via a rebuilt `slate-engine` that writes `engine-control.json`). An old binary wrote leftover `control.json`, so the dock showed Comfy/VL down even when both servers were up.
- Cursor brain can use **Grok 4.5**, **Grok 4.6**, or Composer via `cursor-agent login` (Claude Code stays removed — watermarked generations). Old project `brain: "claude"` migrates to `cursor`.
- Split control descriptors: Electron → `electron-control.json` (`app: slate-electron`); engine serve → `engine-control.json` (`app: slate-engine`). Clients reject a mismatched `app`. Leftover `control.json` is unused.
- Rebrand fork identity to **Agent-Slate** (app name, menus, About/Help, GitHub URLs). Technical IDs unchanged: `slate-engine`, MCP `slate_*` tools, `~/Documents/Slate/`
- Docs pass: STATUS HEAD, engine video-judge + assemble dock, spec/plan banners, pack index, Help, README skill zip
- **`default-i2v` / `default-flf2v`** — LTX 2.3 distilled image-to-video and first-last-frame packs
- Factory auto-keyframes via Flux when those packs need an image
- VL judge extracts the first frame from mp4
- **`slate_assemble`** + Agent dock **Assemble cut**
- Shareable skill zip: `share/slate-film-factory.zip`

## 2026-08-12

### Docs

- Canonical [STATUS.md](STATUS.md); engine/README/skill/help brought in line with shipped V1

### Engine / packs (on `main`)

- **`default-video`** — live LTX 2.3 distilled T2V API graph (Gemma + distilled LoRA + joint AV). Factory canvas clamp 768×432 / 432×768. One-clip smoke: `slate_video_00001_.mp4`, 356 KB, 92s.
- **Agent dock** — **Run brief** (`background: true` + `slate_status`) and **Compile cues** (`slate_compile_music`)
- **`take.mediaPath`** — first-class path; notes line is fallback
- **Comfy interrupt** — `slate_cancel` POSTs `/interrupt` and clears the queue; generate poll honors cancel
- **Coverage parse** — string JSON, `coverage`/`title`/`purpose`, shot maps
- Hermes MCP: blocking `slate_film_factory`, timeout **1800s** (default + cinegen profiles)

## 2026-08 (film factory V1)

### Added

- **Rust film factory engine** (`slate-engine` and crates):
  - `slate-domain` — project model, First AD actions, filesystem store, rule-based Comfy compile
  - `slate-brain` — local OpenAI-compatible servers, Cursor CLI, Codex CLI
  - `slate-comfy` — ComfyUI API packs, workflow inject, dry-run generation
  - `slate-engine` — loopback HTTP, stdio MCP, `slate_film_factory`
- **Packs** — `default-still` (Flux); `default-video` (LTX T2V); later `default-i2v` / `default-flf2v` (see 2026-08-13)
- Quality gate (preferred `qwen3.5:9b`), First AD continuity book, Atomic Notes
- Hermes skill — `skills/slate-film-factory/SKILL.md`
- Cargo workspace — root `Cargo.toml` / `Cargo.lock`

### Changed

- README — engine/Hermes section, development notes for this fork
- NOTICE / package metadata — fork maintainer **thcabq352** alongside required upstream credit

### Notes

- Weights are never bundled. Re-align checkpoint names if Comfy lists different files.
- Electron remains the interactive studio; `slate-engine` is the headless/agent path.
