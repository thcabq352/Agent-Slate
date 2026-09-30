# Install — slate-seedance-draft

Hermes skill for Pack C. Local-only hard requirement: no cloud APIs, no cloud services, no hosted inference. Prompt plus local Comfy (`http://127.0.0.1:8188`, `workflows/packs/`). No weights, no binary, no MCP server. Partner JSON under `workflows/partner/seedance-2-5-draft/` is a field-shape record and is not an execution route.

From a checkout of Agent-Slate:

```bash
hermes skills install /ABS/PATH/to/Agent-Slate/skills/slate-seedance-draft -y --name slate-seedance-draft --category media
```

The film-factory zip (`npm run share:skill` → `share/slate-film-factory.zip`) is a different skill. This one stays source-only until a hub publish.

Local packs stay `workflows/packs/`. The Partner folder is not a `slate-engine` pack.
