# Install — Hermes Skills Hub

Instructions + MCP contract only. No weights, no engine binary.

## Hub / tap (preferred)

From a clone of [thcabq352/Agent-Slate](https://github.com/thcabq352/Agent-Slate):

```bash
hermes skills tap add thcabq352/Agent-Slate
hermes skills inspect slate-film-factory
hermes skills install slate-film-factory -y --category media
```

Zip from this repo (`npm run share:skill` → `share/slate-film-factory.zip`). Install the zip **as-is** (do not unzip first). The archive nests `slate-film-factory/SKILL.md` + `references/`:

```bash
hermes skills install /ABS/PATH/to/share/slate-film-factory.zip -y --name slate-film-factory --category media
```

If the repo is private and tap install cannot fetch, use the zip from a clone. Do **not** install a raw `SKILL.md` URL alone — that drops `references/`.

Lands under `~/.hermes/skills/media/slate-film-factory/` (Windows: `%USERPROFILE%\.hermes\skills\media\…`). Restart the session so the catalog picks it up.

## Engine MCP

Build, then register with the Hermes CLI (do **not** paste server YAML; hub scan treats that as config mutation):

```bash
cargo build -p slate-engine
# Windows:
hermes mcp add slate -- /ABS/PATH/to/target/debug/slate-engine.exe mcp
# macOS / Linux:
hermes mcp add slate -- /ABS/PATH/to/target/debug/slate-engine mcp
hermes mcp test slate
```

Tool calls need **1800 seconds**. If `hermes mcp test` or a factory run dies around five minutes, raise the **slate** MCP timeout through `hermes mcp` / Hermes MCP settings — not a hand-edited config file.

Dry-run (plan only, no Comfy): set `SLATE_DRY_RUN=1` on the engine process.

## Cursor / other MCP

Same stdio command: `slate-engine mcp` (`.exe` on Windows). Studio MCP while Electron is open is `mcp/slate-mcp.mjs` — that is a different server from `slate-engine`.

Repo: https://github.com/thcabq352/Agent-Slate · `docs/GUIDE.md` · `docs/STATUS.md` · `AGENTS.json`
