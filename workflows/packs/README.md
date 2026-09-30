# Comfy packs

Checked-in API graphs for Agent-Slate `slate_film_factory` / `slate_run_pack` / `slate_generate_shot`. Weights stay in your Comfy install. Snapshot: [docs/STATUS.md](../../docs/STATUS.md) · flows: [docs/GUIDE.md](../../docs/GUIDE.md).

| Pack | Modality | Graph | Factory |
|------|----------|--------|---------|
| [`default-still`](default-still/) | image | Flux.1-dev fp8 | Direct |
| [`default-video`](default-video/) | video | LTX 2.3 distilled T2V | Direct (768 long-edge, 49 frames) |
| [`default-i2v`](default-i2v/) | video | LTX `LTXVImgToVideo` | Flux keyframe (or last still) → clip |
| [`default-flf2v`](default-flf2v/) | video | LTX `LTXVAddGuide` 0 / −1 | Start + next-shot (or same) still |

Video packs are **local Comfy** only. They do not call LTX Cloud.

| Job | Local stack |
| --- | --- |
| New scene, multi-cut, or synced dialogue | **LTX-2.5** (split weights, Gemma 4). Not a filename swap onto the 2.3 graphs below. |
| Extend or retake an existing plate | **LTX-2.3 Pro only.** No extend/retake graph is checked in. Do not use LTX-2.5 or the distilled draft checkpoint for that job. |

The checked-in video graphs stay on the installed LTX-2.3 distilled fp8 + Gemma 3 stack. Inventory and the local V1→V2 recipe: [docs/ltx-local-stack-audit.md](../../docs/ltx-local-stack-audit.md).

`slate_list_packs` reports `ready: false` if `workflow.api.json` still contains `PLACEHOLDER` / `ALIGN_ME`.

`slate_run_pack` extra fields: `image`, `image_end` (I2V / FLF2V), `frames`.

See [docs/STATUS.md](../../docs/STATUS.md).
