# Seedance 2.5 Draft → Final — field-shape record

**Local-only hard requirement.** Agents do not execute this folder. Generation for Pack C is local Comfy at `http://127.0.0.1:8188` via `workflows/packs/` (`default-still`, `default-video`, `default-i2v`, `default-flf2v`) and `slate-engine`.

The JSON here records Partner node fields (ComfyUI #16529, template ids `api_seedance2_5_draft_{t2v,i2v,r2v}`). `ByteDance2TextToVideoNode`, `ByteDance2FirstLastFrameNode`, `ByteDance2ReferenceNodeV2`, and `ByteDance2DraftToFinalVideoNode` call hosted inference. Node mode stays `4` (bypass). `contract.json` → `localOnly.agentExecution` is `forbidden`.

`slate-engine` only loads `workflows/packs/` directories that contain `manifest.json`. This folder has none.

## Recorded shape

| | Scout fields | Promote fields |
|--|----------------|----------------|
| Model option | `Seedance 2.5 Draft` | inherited |
| Model id | `dreamina-seedance-2-5-260628` | `dreamina-seedance-2-5-260628` |
| Flag | `draft: true` via the Draft model option | `draft_task_id` |
| Resolution | `480p` | `1080p` on `ByteDance2DraftToFinalVideoNode` |
| Class | `ByteDance2TextToVideoNode` / `ByteDance2FirstLastFrameNode` / `ByteDance2ReferenceNodeV2` | `ByteDance2DraftToFinalVideoNode` |

Prompt string: `prompt-template.txt`. Seed widget ships at `0` with `control_after_generate` `fixed`. `model.generate_audio` is `false`.

Skill: [`skills/slate-seedance-draft/SKILL.md`](../../../skills/slate-seedance-draft/SKILL.md). Local-only rule: [`references/local-only.md`](../../../skills/slate-seedance-draft/references/local-only.md).
