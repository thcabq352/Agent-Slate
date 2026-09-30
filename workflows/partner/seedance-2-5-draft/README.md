# Seedance 2.5 Draft → Final (Comfy Desk)

Pack C reference graphs for **Comfy Desk**. They match the Partner nodes added in ComfyUI #16529 and the template ids from workflow_templates #1288.

`slate-engine` discovers factory packs only under `workflows/packs/` (a directory with `manifest.json`). This folder has neither, so `slate_list_packs`, `slate_run_pack`, and `slate_film_factory` leave these graphs alone.

Both stages ship **bypassed** (node mode `4`). Opening a graph does not submit a Partner task. Enable Stage 1 on the Desk when a scout is intended. Promote within about **7 days**.

## Contract

| | Scout | Promote |
|--|--------|---------|
| Model option | `Seedance 2.5 Draft` | inherited |
| Model id | `dreamina-seedance-2-5-260628` | `dreamina-seedance-2-5-260628` |
| API flag | `draft: true` via the Draft model option | `draft_task_id` content |
| Resolution | `480p` (locked) | native `1080p` |
| Node | `ByteDance2TextToVideoNode` / `ByteDance2FirstLastFrameNode` / `ByteDance2ReferenceNodeV2` | `ByteDance2DraftToFinalVideoNode` |
| Display name | ByteDance Seedance 2.5 Text / First-Last-Frame / Reference to Video | **ByteDance Seedance 2.5 Draft to Final Video** |

Same prompt and the same refs on both stages. The final node has no prompt input; it reuses the draft's prompt, refs, duration, aspect ratio, seed, and `generate_audio`.

`control_after_generate` is `fixed` in these graphs. Set the seed integer that belongs to the approved scout and leave the control on fixed before promote, so the next Stage 1 queue does not mint a new `draft_task_id`. The Partner node tooltip says picture results are non-deterministic regardless of seed; the fixed control is what keeps Comfy from changing the seed.

`model.generate_audio` is `false` because post owns audio. Set it `true` on the scout only when the take must carry dialogue or effects.

## Files

| File | What it is |
|------|------------|
| `api_seedance2_5_draft_t2v.json` | Desk workflow. Template id `api_seedance2_5_draft_t2v`. Ratio `16:9`. |
| `api_seedance2_5_draft_i2v.json` | Desk workflow. Template id `api_seedance2_5_draft_i2v`. First frame only; ratio follows that plate, so the still is 16:9. |
| `api_seedance2_5_draft_r2v.json` | Desk workflow. Template id `api_seedance2_5_draft_r2v`. `task_type` `reference`. Images: character, location, clay blocking. |
| `*.api.json` | API-format field map (Stage 1). Dotted widget names, no `control_after_generate`. |
| `api_seedance2_5_draft_promote.api.json` | API-format field map (Stage 2 only). |
| `contract.json` | Same contract in JSON. |
| `prompt-template.txt` | Pack C prompt stored in each scout node. |

Replace the Load Image filenames before enabling Stage 1. `draft_task_id` is saved by Save Text (`AgentSlate_seedance25_draft_task_id.txt`). Paste that string into the Stage 2 node, bypass Stage 1, set Stage 2 to mode `0`, and queue the promote by itself.

Skill: [`skills/slate-seedance-draft/SKILL.md`](../../../skills/slate-seedance-draft/SKILL.md). Narrative: [`docs/packs/pack-c-seedance-2-5-draft.md`](../../../docs/packs/pack-c-seedance-2-5-draft.md).

Official docs: https://docs.comfy.org/tutorials/partner-nodes/bytedance/seedance-2-5-draft
