# Share

Rebuild the zip after editing the skill:

```bash
npm run share:skill
```

| File | What to send |
|------|----------------|
| **`slate-film-factory.zip`** | Hermes Skills Hub bundle (`SKILL.md` + `references/`). No weights, no binary. |

Preferred: `hermes skills tap add thcabq352/Agent-Slate` then `hermes skills install slate-film-factory -y --category media`.

Or install the zip as-is (do not unzip first): `hermes skills install share/slate-film-factory.zip -y --name slate-film-factory --category media`. Then `hermes mcp add slate -- <slate-engine> mcp` (timeout **1800s**; `.exe` on Windows). See `references/install.md` inside the skill.

Repo: https://github.com/thcabq352/Agent-Slate · [docs/GUIDE.md](../docs/GUIDE.md)
