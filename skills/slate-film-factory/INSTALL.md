# Install — Agent-Slate Film Factory skill

Hermes Skills Hub layout: `SKILL.md` + `references/`. No weights, no binary.

**Preferred**

```bash
hermes skills tap add thcabq352/Agent-Slate
hermes skills install slate-film-factory -y --category media
```

Zip: `npm run share:skill` → `share/slate-film-factory.zip`, then `hermes skills install <zip> -y --name slate-film-factory --category media`.

**MCP** — `hermes mcp add slate -- <path-to-slate-engine> mcp` then `hermes mcp test slate`. Tool timeout **1800s**. Do not paste MCP server YAML.

Full steps: `references/install.md` (or `skill_view("slate-film-factory", "references/install.md")`).

Repo: https://github.com/thcabq352/Agent-Slate
