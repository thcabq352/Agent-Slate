# Fork changelog (thcabq352/slate)

Notable changes in this maintained fork relative to upstream Slate. Apache-2.0 attribution for the original work remains with Sam Wasserman; see [NOTICE](../NOTICE) and [ATTRIBUTION.md](ATTRIBUTION.md).

## Unreleased / 2026-08

### Added

- **Take.mediaPath** — first-class path on takes (notes line no longer the only locator)
- **Comfy interrupt** — `slate_cancel` POSTs `/interrupt` + clears the queue; generate poll honors cancel
- **Background factory** — `slate_film_factory` `{ background: true }` for the Agent dock; Hermes still uses the blocking call
- **Coverage parse** — accepts string-encoded JSON, `coverage`/`title`/`purpose`, and shot maps
- **default-video pack** — real LTX 2.3 distilled T2V API graph (Gemma + distilled LoRA + joint AV), factory canvas clamp (768×432 / 432×768)
- **Agent dock** — `slate_compile_music` (generic / Suno) and **Run brief** (`slate_film_factory`)
- **Rust film factory engine** (`slate-engine` and crates):
  - `slate-domain` — project model, First AD actions, filesystem store, rule-based Comfy compile
  - `slate-brain` — local OpenAI-compatible servers, Claude Code CLI, Codex CLI
  - `slate-comfy` — ComfyUI API packs, workflow inject, default-still pack, dry-run generation
  - `slate-engine` — loopback HTTP control server, stdio MCP, synchronous `slate_film_factory`
- **Hermes skill** — `skills/slate-film-factory/SKILL.md`
- **Docs** — design spec and implementation plan under `docs/superpowers/`; maintainer/attribution docs
- **Cargo workspace** — root `Cargo.toml` / `Cargo.lock`

### Changed

- README — engine/Hermes section, development notes for this fork
- NOTICE / package metadata — fork maintainer **thcabq352** alongside required upstream credit

### Notes

- Live Comfy stills: `default-still` (Flux.1-dev fp8). Live video: `default-video` (LTX 2.3 distilled). Re-align checkpoint names if Comfy lists different files.
- Electron app remains the interactive studio; `slate-engine` is the headless/agent path.
