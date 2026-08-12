# default-still pack

Minimal **still-image** ComfyUI API pack used by `slate_film_factory` and `slate_generate_shot`.

## Files

| File | Role |
|------|------|
| `manifest.json` | Logical inputs → Comfy **node id** + field mappings |
| `workflow.api.json` | API-format graph (fixture-shaped skeleton) |

## Manifest node ids (current skeleton)

| Logical field | Node id | Field | Notes |
|---------------|---------|-------|-------|
| `positive` | `6` | `text` | CLIPTextEncode |
| `negative` | `7` | `text` | CLIPTextEncode |
| `width` / `height` | `5` | `width` / `height` | EmptyLatentImage |
| `seed` | `3` | `seed` | KSampler; `mode: randomize` |
| media output | `9` | (SaveImage) | `outputs.media` |

Graph shape: CheckpointLoaderSimple (`4`) → encode/sample → VAEDecode (`8`) → SaveImage (`9`).

## Local alignment required

This pack ships as a **fixture-shaped** graph for structure and CI. It is **not** guaranteed to run as-is on a given Comfy install.

Before live GPU generation:

1. Export your working still graph from ComfyUI (**Save (API Format)**).
2. Replace `workflow.api.json` with that export (or a trimmed still subgraph).
3. Update **every** `node_id` / `field` in `manifest.json` so inject targets match your graph.
4. Set a real `ckpt_name` (or equivalent loader input) — the skeleton uses `PLACEHOLDER.safetensors`, which will not exist locally.
5. Confirm output node id for `outputs.media` matches the node that produces images in history.
6. Smoke: `SLATE_DRY_RUN=0`, Comfy on `http://127.0.0.1:8188`, then one `slate_generate_shot` or short `slate_film_factory` run.

If node ids drift after you edit the graph in the UI, re-export API JSON and re-align the manifest — the engine does not auto-discover node ids.

## Dry-run

With `SLATE_DRY_RUN=1`, Comfy is skipped; takes land as `dry-run.txt` under the project takes dir.
