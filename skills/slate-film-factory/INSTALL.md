# Install — Slate Film Factory skill

Share this folder (or `share/slate-film-factory.zip` from the repo). It is **instructions + MCP contract**, not model weights.

## Hermes

```bash
# 1) Copy the skill
# Windows:  %USERPROFILE%\.hermes\skills\slate-film-factory\SKILL.md
# macOS/Linux: ~/.hermes/skills/slate-film-factory/SKILL.md

# 2) Build / point at slate-engine
cargo build -p slate-engine

# 3) Register MCP (blocking factory, 1800s)
hermes mcp add slate -- /ABS/PATH/to/target/debug/slate-engine mcp
```

YAML:

```yaml
mcp_servers:
  slate:
    command: /ABS/PATH/to/target/debug/slate-engine
    args: [mcp]
    enabled: true
    timeout: 1800
    connect_timeout: 120
```

Restart the Hermes gateway.

## Claude Code / other MCP

```bash
# stdio
slate-engine mcp
```

## What the agent must do

- Call **`slate_film_factory` blocking** — do **not** pass `"background": true` (that is Electron-only).
- Timeout **1800s**.
- Prefer `pack_id: "default-still"` for a 4-shot factory; `default-i2v` / `default-flf2v` / `default-video` need local LTX 2.3 + Gemma + distilled LoRA on Comfy `:8188`.

Repo: https://github.com/thcabq352/slate (Agent-Slate). Operator docs: `docs/STATUS.md`.
