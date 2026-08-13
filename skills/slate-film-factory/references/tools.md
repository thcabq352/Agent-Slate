# slate-engine MCP tools

Call these on MCP server **`slate`**. Agent factory is **blocking**. Never pass `background: true` from Hermes / Cursor / Grok — that flag is Electron ◆ Agent dock only.

| Tool | Args (common) | Notes |
|------|----------------|-------|
| `slate_health` | (none) | Comfy, brains, ffmpeg, packsDir, VL |
| `slate_film_factory` | `brief` (required), `pack_id`, `brain`, `shot_count` (4–8), `project_name` | Timeout **1800s**. Omit `background`. |
| `slate_generate_shot` | `projectId`, `shotId`, `pack_id`? | Quality-gate retries |
| `slate_judge_take` | `mediaPath`, `prompt`?, `continuity`? | mp4 → first ffmpeg frame |
| `slate_first_ad` | `projectId`, `message`, `history`?, `brain`? | **Factory AD**. Not studio titlebar First AD. |
| `slate_circle_take` | `projectId`, `takeId`?, `shotId`? | Same ⭕ as Deliver / dock Approve |
| `slate_assemble` | `projectId`, `circledOnly`? | `{project}/cut/slate_cut.mp4` |
| `slate_cancel` | (none) | Flag + Comfy `/interrupt` + queue clear |
| `slate_status` | (none) | Background dock jobs |
| `slate_list_packs` | (none) | `ready: false` if graph has PLACEHOLDER |
| `slate_run_pack` | `pack_id`, `positive`, `negative`?, size, `frames`?, `image`?, `image_end`?, seed | One graph |
| `slate_compile_music` | `projectId`, `target`, `cueId`? | Text only |
| `slate_list_projects` / `slate_get_project` / `slate_list_takes` | ids as named | Store |
| `slate_note_write` / `slate_note_search` | project notes | `.notes/notes.jsonl` |

## Packs

| pack_id | Result |
|---------|--------|
| `default-still` | Flux stills — best first 4-shot factory |
| `default-i2v` | Flux keyframe or still → LTX I2V (~2s) |
| `default-flf2v` | First + last still → LTX |
| `default-video` | LTX T2V from text |

Comfy: `http://127.0.0.1:8188`. Weights stay in Comfy. Env: `SLATE_DRY_RUN`, `SLATE_PACKS_DIR`, `SLATE_BRAIN`, `SLATE_FFMPEG`, `SLATE_JUDGE_MODEL`.
