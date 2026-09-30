# Comfy Desk attach contract

Partner path for Pack C. Graphs in the Agent-Slate repo are references for Comfy Desk. They are not `workflows/packs/` factory graphs, and `slate_film_factory` / `slate_run_pack` do not load them.

Official template ids (Comfy-Org/workflow_templates #1288):

| Route | Template id | Scout class | Display name |
|-------|-------------|-------------|--------------|
| Text | `api_seedance2_5_draft_t2v` | `ByteDance2TextToVideoNode` | ByteDance Seedance 2.5 Text to Video |
| First frame | `api_seedance2_5_draft_i2v` | `ByteDance2FirstLastFrameNode` | ByteDance Seedance 2.5 First-Last-Frame to Video |
| References | `api_seedance2_5_draft_r2v` | `ByteDance2ReferenceNodeV2` | ByteDance Seedance 2.5 Reference to Video |

Promote, all three: class `ByteDance2DraftToFinalVideoNode`, display name **ByteDance Seedance 2.5 Draft to Final Video**.

## Scout

| Field | Value |
|-------|--------|
| Model option | `Seedance 2.5 Draft` |
| Model id | `dreamina-seedance-2-5-260628` |
| `draft` | `true`, sent because the model option is Draft. No separate draft widget. |
| `model.resolution` | `480p` (the Draft option locks the list to 480p) |
| `model.ratio` | `16:9` on T2V and R2V |
| I2V ratio | No ratio widget. Seedance 2.5 on the first-last node submits adaptive ratio from the first frame. The plate is 16:9. `last_frame` stays empty for this one-take. |
| `model.duration` | `30` in the shipped graphs (legal range 4–30) |
| `model.generate_audio` | `false` when post owns audio |
| `model.output_format` | `mp4` |
| R2V `model.task_type` | `reference` (validated at submit). `edit` / `extend` are other tasks. |
| `seed` | integer for this scout; shipped graphs use `0` |
| `control_after_generate` | `fixed` before promote |
| `watermark` | `false` |
| `draft_task_id` output | Connect only while Draft is selected. Save Text writes it. |

`control_after_generate` is a Desk widget. It is omitted from the API-format JSON. The UI graphs carry `fixed`.

## Promote

Paste the saved id into `draft_task_id` on `ByteDance2DraftToFinalVideoNode`. Optional `watermark` (shipped `false`).

The node submits model `dreamina-seedance-2-5-260628` at resolution `1080p`. Prompt, refs, duration, aspect ratio, seed, and audio come from the draft. The id is usable for about 7 days.

Bypass Stage 1 (node mode `4`) before that queue. A live Stage 1 node creates a new draft instead of reusing the preview.

## Repo files

| File | Role |
|------|------|
| `api_seedance2_5_draft_t2v.json` | Desk workflow, T2V, both stages bypassed |
| `api_seedance2_5_draft_i2v.json` | Desk workflow, first frame |
| `api_seedance2_5_draft_r2v.json` | Desk workflow, three image refs |
| `*.api.json` | API-format field map for that scout (Stage 1 only) |
| `api_seedance2_5_draft_promote.api.json` | API-format field map for Stage 2 only |
| `contract.json` | Machine-readable copy of this table |
| `prompt-template.txt` | Prompt string embedded in the three Desk graphs |

Node mode `4` is bypass. Mode `0` is always. Shipped graphs use mode `4` on every Partner and save node so a Desk open does not submit a task. Enable a stage by setting that stage's nodes to mode `0`.

Load in Comfy Desk: **Load** the repo JSON, or open the official template id and apply the table above. Replace `first-frame.png`, `character.png`, `location.png`, and `clay-blocking.png` with the shot's media before enabling Stage 1.

## Sources

- https://docs.comfy.org/tutorials/partner-nodes/bytedance/seedance-2-5-draft
- https://github.com/Comfy-Org/ComfyUI/pull/16529
- https://github.com/Comfy-Org/workflow_templates/pull/1288
