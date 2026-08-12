# default-video pack

Video-modality ComfyUI API pack for `slate_film_factory` / `slate_run_pack` / `slate_generate_shot`.

**Aligned to this machine’s Video Buddy Comfy** (`http://127.0.0.1:8188`) using **LTX 2.3 22B distilled fp8** (text-to-video + joint audio latent). Graph shape matches Video Buddy’s simplified single-pass T2V (`base_t2v_i2v.json`).

Factory injects 768×432 (16:9) or 432×768 (9:16) so 1280×720 stills sizes do not OOM a 16 GB card. Default length is **49 frames @ 24 fps** (~2 s). Weights are **not** bundled.

## Models required (as Comfy lists them)

| Slot | File |
|------|------|
| Checkpoint / VAE / audio VAE | `ltx-2.3-22b-distilled-fp8.safetensors` |
| Text encoder | `gemma_3_12B_it_fp4_mixed.safetensors` |
| Distilled LoRA | `ltx-2.3-22b-distilled-1.1_lora-dynamic_fro09_avg_rank_111_bf16.safetensors` |

## Graph

Checkpoint + Gemma/projection + audio VAE + distilled LoRA → CLIPTextEncode ×2 → LTXVConditioning → EmptyLTXVLatentVideo + LTXVEmptyLatentAudio → concat AV → MultimodalGuider → LTXVScheduler (8 steps) → SamplerCustomAdvanced → separate AV → tiled VAE decode + audio decode → CreateVideo → SaveVideo.

| Logical field | Node | Field | Notes |
|---------------|------|-------|--------|
| `positive` | `10` | `text` | |
| `negative` | `11` | `text` | |
| `width` / `height` | `20` | width/height | factory clamps to 768 long edge |
| `seed` | `42` | `noise_seed` | randomize if omitted |
| `frames` (optional) | `20` | `length` | mirrors `21.frames_number` |
| media output | `90` | SaveVideo | history reports under `images` (animated) |

## Smoke

```bash
# Comfy running on 8188
cargo run -p slate-engine -- serve
# slate_run_pack / slate_generate_shot / slate_film_factory with pack_id default-video
# Live one-clip (slow): SLATE_LIVE_VIDEO=1 cargo test -p slate-comfy --test live_ltx_video -- --ignored --nocapture
```

## Re-align on another machine

1. Confirm LTX 2.3 distilled + Gemma + distilled LoRA names in Comfy.
2. Export a working T2V graph (**Save (API Format)**) if node ids differ.
3. Replace `workflow.api.json` and update `manifest.json`.
4. `slate_list_packs` shows `ready: true` when the graph no longer contains `PLACEHOLDER` / `ALIGN_ME`.
