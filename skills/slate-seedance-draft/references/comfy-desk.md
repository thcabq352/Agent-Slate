# Partner field-shape record

Local-only hard requirement: this file is not an execution route. Generate with [`local-only.md`](local-only.md) on `http://127.0.0.1:8188`.

`ByteDance2*` classes call hosted inference. The agent does not load, enable, or queue the graphs in `workflows/partner/seedance-2-5-draft/`.

## Recorded node fields

| Route | Template id | Class | Display name |
|-------|-------------|-------|--------------|
| Text | `api_seedance2_5_draft_t2v` | `ByteDance2TextToVideoNode` | ByteDance Seedance 2.5 Text to Video |
| First frame | `api_seedance2_5_draft_i2v` | `ByteDance2FirstLastFrameNode` | ByteDance Seedance 2.5 First-Last-Frame to Video |
| References | `api_seedance2_5_draft_r2v` | `ByteDance2ReferenceNodeV2` | ByteDance Seedance 2.5 Reference to Video |
| Promote | (same three files, Stage 2) | `ByteDance2DraftToFinalVideoNode` | ByteDance Seedance 2.5 Draft to Final Video |

| Field | Recorded value |
|-------|----------------|
| Model option | `Seedance 2.5 Draft` |
| Model id | `dreamina-seedance-2-5-260628` |
| `draft` | `true` when that model option is selected |
| `model.resolution` | `480p` on the scout |
| Promote resolution | `1080p` inside `ByteDance2DraftToFinalVideoNode` |
| `model.ratio` | `16:9` on the T2V and R2V records |
| I2V ratio | No ratio widget on the first-last node; the record expects a 16:9 plate |
| `model.duration` | `30` |
| `model.generate_audio` | `false` |
| R2V `model.task_type` | `reference` |
| `seed` / `control_after_generate` | `0` / `fixed` |
| `draft_task_id` | String output of the scout, input of the promote node |

Graphs ship with node mode `4` (bypass). `contract.json` sets `localOnly.agentExecution` to `forbidden`.
