# Fork changelog (thcabq352/slate)

Notable changes in this maintained fork relative to upstream Slate. Apache-2.0 attribution for the original work remains with Sam Wasserman; see [NOTICE](../NOTICE) and [ATTRIBUTION.md](ATTRIBUTION.md).

Current snapshot: [STATUS.md](STATUS.md).

## 2026-08-13

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
  - `slate-brain` — local OpenAI-compatible servers, Claude Code CLI, Codex CLI
  - `slate-comfy` — ComfyUI API packs, workflow inject, dry-run generation
  - `slate-engine` — loopback HTTP, stdio MCP, `slate_film_factory`
- **Packs** — `default-still` (Flux.1-dev fp8, live); `default-video` (see above)
- Quality gate (preferred `qwen3.5:9b`), First AD continuity book, Atomic Notes
- Hermes skill — `skills/slate-film-factory/SKILL.md`
- Cargo workspace — root `Cargo.toml` / `Cargo.lock`

### Changed

- README — engine/Hermes section, development notes for this fork
- NOTICE / package metadata — fork maintainer **thcabq352** alongside required upstream credit

### Notes

- Weights are never bundled. Re-align checkpoint names if Comfy lists different files.
- Electron remains the interactive studio; `slate-engine` is the headless/agent path.
