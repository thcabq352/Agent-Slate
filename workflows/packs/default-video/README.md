# default-video pack

Video-modality slot for `slate_film_factory` / `slate_run_pack`.

This machine has **LTX 2.3** and **Wan** diffusion models on Comfy `:8188`, but those graphs are **not** the same as Flux stills. This folder ships a **template** (`PLACEHOLDER` / `ALIGN_ME`).

## Before live video

1. Build a working text-to-video graph in ComfyUI (LTX / Wan).
2. **Save (API Format)** → replace `workflow.api.json`.
3. Update `manifest.json` node ids for positive/negative/size/seed/output.
4. `slate_list_packs` will show `ready: true` once PLACEHOLDER is gone.

Dry-run (`SLATE_DRY_RUN=1`) still writes marker files so factory/video tools can be exercised without GPU.

Do **not** ship model weights in the Slate package.
