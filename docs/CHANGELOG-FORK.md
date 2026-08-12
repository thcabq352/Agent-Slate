# Fork changelog (thcabq352/slate)

Notable changes in this maintained fork relative to upstream Slate. Apache-2.0 attribution for the original work remains with Sam Wasserman; see [NOTICE](../NOTICE) and [ATTRIBUTION.md](ATTRIBUTION.md).

## Unreleased / 2026-08

### Added

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

- Live Comfy still generation requires aligning `workflows/packs/default-still` with a local checkpoint graph (fixture pack ships for structure/dry-run).
- Electron app remains the interactive studio; `slate-engine` is the headless/agent path.
