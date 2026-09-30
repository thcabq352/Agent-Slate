# Install — slate-seedance-draft

Hermes skill for Pack C (Seedance 2.5 Draft → native 1080p). Prompt and Comfy Desk contract only. No weights, no binary, no MCP server.

From a checkout of Agent-Slate:

```bash
hermes skills install /ABS/PATH/to/Agent-Slate/skills/slate-seedance-draft -y --name slate-seedance-draft --category media
```

The film-factory zip (`npm run share:skill` → `share/slate-film-factory.zip`) is a different skill. This one stays source-only until a hub publish.

Desk graphs stay in the repo at `workflows/partner/seedance-2-5-draft/`. They are not registered as `slate-engine` packs.
