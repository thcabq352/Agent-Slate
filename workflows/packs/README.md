# Comfy packs

Checked-in API graphs for `slate_film_factory` / `slate_run_pack` / `slate_generate_shot`. Weights stay in your Comfy install.

| Pack | Modality | Graph | Factory |
|------|----------|--------|---------|
| [`default-still`](default-still/) | image | Flux.1-dev fp8 | Direct |
| [`default-video`](default-video/) | video | LTX 2.3 distilled T2V | Direct (768 long-edge, 49 frames) |
| [`default-i2v`](default-i2v/) | video | LTX `LTXVImgToVideo` | Flux keyframe (or last still) → clip |
| [`default-flf2v`](default-flf2v/) | video | LTX `LTXVAddGuide` 0 / −1 | Start + next-shot (or same) still |

`slate_list_packs` reports `ready: false` if `workflow.api.json` still contains `PLACEHOLDER` / `ALIGN_ME`.

`slate_run_pack` extra fields: `image`, `image_end` (I2V / FLF2V), `frames`.

See [docs/STATUS.md](../../docs/STATUS.md).
