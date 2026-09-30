# default-video pack

Video-modality ComfyUI API pack for Agent-Slate `slate_film_factory` / `slate_run_pack` / `slate_generate_shot`.

**Aligned to this machine’s Video Buddy Comfy** (`http://127.0.0.1:8188`) using **LTX 2.3 22B distilled fp8** (text-to-video + joint audio latent). Graph shape matches Video Buddy’s simplified single-pass T2V (`base_t2v_i2v.json`).

Factory injects 768×432 (16:9) or 432×768 (9:16) so 1280×720 stills sizes do not OOM a 16 GB card. Clip length is **shot `durationSec × fps`**, snapped to LTX **8n+1** (graph default 49 is only the fallback when frames are omitted). Weights are **not** bundled.

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
| `frames` | `20` | `length` | factory injects `durationSec × fps`, LTX 8n+1, clamped to pack max |
| media output | `90` | SaveVideo | history reports under `images` (animated) |

## Smoke

```bash
# Comfy running on 8188
cargo run -p slate-engine -- serve
# slate_run_pack / slate_generate_shot / slate_film_factory with pack_id default-video
# Live one-clip (slow): SLATE_LIVE_VIDEO=1 cargo test -p slate-comfy --test live_ltx_video -- --ignored --nocapture
```

**This host (2026-08-12):** `slate_video_00001_.mp4` · 356 KB · 92 s · 768×432 · 49 frames @ 24 fps.

## Local LTX routing

Local Comfy only. This pack does not call LTX Cloud.

| Job | Local stack |
| --- | --- |
| New scene, multi-cut, or synced dialogue | **LTX-2.5** (split weights, Gemma 4 text encoder). Do not point this 2.3 monolith graph at those files. |
| Extend or retake an existing plate | **LTX-2.3 Pro only.** This T2V graph is not that job. |

This file is the installed **new-scene draft**: `ltx-2.3-22b-distilled-fp8.safetensors` + `gemma_3_12B_it_fp4_mixed.safetensors`. There is no legacy LTX-Video 0.9 / T5 video encoder in the graph. Full inventory: [docs/ltx-local-stack-audit.md](../../../docs/ltx-local-stack-audit.md).

## Re-align on another machine

1. Confirm LTX 2.3 distilled + Gemma 3 + distilled LoRA names in Comfy. A new-scene 2.5 graph is a separate export (split transformer, Gemma 4, video VAE, audio VAE), not a `ckpt_name` edit here.
2. Export a working T2V graph (**Save (API Format)**) if node ids differ.
3. Replace `workflow.api.json` and update `manifest.json`.
4. `slate_list_packs` shows `ready: true` when the graph no longer contains `PLACEHOLDER` / `ALIGN_ME`.
