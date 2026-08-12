# default-still pack

Still-image ComfyUI API pack for `slate_film_factory` / `slate_generate_shot`.

**Aligned to this machine’s Video Buddy Comfy** (`http://127.0.0.1:8188`) using **Flux.1-dev fp8**.

## Models required (relative to Comfy `models/`)

| Slot | File |
|------|------|
| UNet | `diffusion_models/flux1-dev-fp8.safetensors` |
| CLIP-L | `clip/clip_l.safetensors` (or text_encoders path as Comfy lists) |
| T5 | `t5xxl_fp8_e4m3fn.safetensors` |
| VAE | `vae/ae.safetensors` |

## Graph

UNET + DualCLIP (flux) + VAE → CLIPTextEncode ×2 → FluxGuidance → EmptySD3LatentImage → ModelSamplingFlux → KSampler (8 steps, cfg 1, euler/simple) → VAEDecode → SaveImage.

| Logical field | Node | Field | Mirrors |
|---------------|------|-------|---------|
| `positive` | `6` | `text` | |
| `negative` | `7` | `text` | (Flux often ignores; left empty ok) |
| `width` / `height` | `27` | width/height | node `30` ModelSamplingFlux |
| `seed` | `3` | `seed` | randomize if omitted |
| media output | `9` | SaveImage | |

## Smoke

```bash
# Comfy running on 8188
cargo run -p slate-engine -- serve
# then invoke slate_generate_shot or slate_film_factory with pack_id default-still
# unset SLATE_DRY_RUN
```

Live test on this host produced `slate_test_00001_.png` via the same graph (8 steps ~20s on RTX 5060 Ti).

## Re-align on another machine

1. Export a working still graph (**Save (API Format)**).
2. Replace `workflow.api.json`.
3. Update `manifest.json` node ids / mirrors.
4. Point loaders at local checkpoint/CLIP/VAE names.
