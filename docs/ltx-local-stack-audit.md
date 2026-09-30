# Local LTX stack audit

Date: 2026-09-30. Scope: Agent-Slate packs, skills, and docs. Video stays on the **local Comfy** stack. This audit does not add an LTX Cloud client, and it does not retarget checked-in graphs at `api.ltx.io`.

Searched `workflows/`, `skills/`, `docs/`, `share/`, `data/`, `mcp/`, `crates/`, and `src/` for legacy **local** LTX-Video (pre-LTX-2) stack markers and for cloud generation URLs.

## Result

No legacy local LTX V1 stack is checked in. No cloud generation caller is checked in. The three video packs are already the local LTX-2.3 monolith (Gemma 3 text encoder, joint audio latent).

## Routing lock

| Job | Local stack |
| --- | --- |
| New scene, multi-cut, or synced dialogue | **LTX-2.5**. Split weights. Gemma 4 text encoder (`gemma4-12b-with-proj-ltx-2.5`). Do not reuse a 2.3 monolith or Gemma 3. |
| Extend or retake an existing plate | **LTX-2.3 Pro only.** Do not run that job on LTX-2.5 or on the distilled draft checkpoint in the factory graphs. |

No retake or extend graph ships in this repo. Do not add one on the distilled T2V graph or on LTX-2.5.

Lightricks publishes local LTX-2.3 as `ltx-2.3-22b-dev.safetensors` and `ltx-2.3-22b-distilled-1.1.safetensors` (Gemma 3 downloaded separately). There is no `ltx-2.3-22b-pro.safetensors` in that model list. "LTX-2.3 Pro" is the routing name for extend/retake. It is not a filename to drop into `ckpt_name`.

## Local V1 → local V2 (guidance only)

Use this when an old recipe shows up on a machine. Do not load the V1 column into `default-video`, `default-i2v`, or `default-flf2v`.

| Local V1 (LTX-Video, not in this repo) | Local V2 (LTX-2 family) |
| --- | --- |
| `ltxv-2b-*`, `ltxv-13b-0.9.*`, `ltx-video-2b-v0.9.*` | LTX-2.3 monolith **or** LTX-2.5 split files. Not interchangeable with each other. |
| T5-XXL as the **video** text encoder | Gemma 3 for a 2.3 monolith (`gemma_3_12B_it_fp4_mixed.safetensors` on the factory graphs). Gemma 4 for 2.5 (`gemma4-12b-with-proj-ltx-2.5`). Stock Gemma is not a substitute for the 2.5 encoder. |
| Video latent only, no audio VAE | Joint audio latent (`LTXVEmptyLatentAudio` + concat) on the checked-in graphs. |
| Prompt as a tag list or a cloud JSON body | One chronological prose paragraph (action, motion, character, place, camera, light, sound). No endpoint names in the paragraph. |

`default-still` uses `t5xxl_fp8_e4m3fn.safetensors` for **Flux**. That file is not an LTX text encoder and is not part of this remap.

Frame length on the checked-in graphs stays **8n+1**. LTX-2.5 is a different graph (separate transformer, text encoder, video VAE, audio VAE). Swapping `ckpt_name` on the 2.3 API graph does not produce a 2.5 pack.

## Inventory

| Path | What it is | Remap |
| --- | --- | --- |
| `workflows/packs/default-video/workflow.api.json` | Local 2.3 distilled fp8 T2V. `CheckpointLoaderSimple` + `LTXAVTextEncoderLoader` (`gemma_3_12B_it_fp4_mixed.safetensors`) + `LTXVAudioVAELoader` + distilled LoRA. | Graph unchanged. New-scene routing target is LTX-2.5 when a split-weight graph is aligned. This file is the installed draft stack, not 2.5. |
| `workflows/packs/default-i2v/workflow.api.json` | Same stack + `LTXVImgToVideo`. | Same drift. Graph unchanged. |
| `workflows/packs/default-flf2v/workflow.api.json` | Same stack + `LTXVAddGuide` / `LTXVCropGuides`. | Same drift. First/last frame is a new plate, not an extend/retake. |
| `workflows/packs/default-video/manifest.json` | Label: local LTX 2.3 distilled T2V. | Unchanged. |
| `workflows/packs/default-i2v/manifest.json` | Label: local LTX 2.3 distilled I2V. | Unchanged. |
| `workflows/packs/default-flf2v/manifest.json` | Label: local LTX 2.3 FLF. | Unchanged. |
| `workflows/packs/default-*/README.md` | Pack docs. | Routing lock added. |
| `workflows/packs/README.md` | Pack index. | Routing lock added. |
| `workflows/packs/default-still/` | Flux fp8 + T5-XXL. | Out of scope. |
| `skills/slate-film-factory/SKILL.md` | Hermes skill. Local Comfy only. | Routing lock added. |
| `skills/slate-film-factory/references/tools.md` | Pack table for agents. | Routing lock added. |
| `skills/slate-film-factory/INSTALL.md`, `references/install.md` | Install steps. No checkpoint ids. | No stack remap. |
| `share/slate-film-factory.zip` | Skill bundle. | Rebuilt from the skill tree. |
| `share/README.md` | Zip install note. No LTX checkpoint ids. | No stack remap. |
| `data/model-profiles.json` (`id: ltx-2.3`) | Studio prompt dialect. | Notes and param notes now say local Comfy, not a cloud wrapper. Variant labels `ltx-2.3-22b-fast` / `ltx-2.3-22b-pro` kept and marked as **not** local filenames. |
| `docs/STATUS.md`, `docs/engine.md`, `docs/GUIDE.md`, `README.md` | Operator pack tables. | Pointer plus routing lock. Graphs still described as the live 2.3 distilled stack. |
| `docs/CHANGELOG-FORK.md` | Historical fork notes ("LTX 2.3 distilled"). | Left as history. |
| `docs/superpowers/specs/…`, `docs/superpowers/plans/…` | Design history. "V1" there means the Agent-Slate factory milestone, not LTX-Video. | Left as history. |
| `src/renderer/src/lib/firstAD.ts` | Spec vocabulary includes target id `ltx-2.3`. | Unchanged. That id matches the profile. |
| `crates/slate-engine/src/factory.rs` | `ltx_frame_count` snaps to 8n+1 for the video packs. | Unchanged. Local latent rule, not a cloud response. |
| `crates/slate-comfy/tests/live_ltx_video.rs` | Opt-in live Comfy smoke. | Unchanged. |
| `crates/slate-comfy/tests/inject_test.rs` | Asserts `LTXVImgToVideo` / `LTXVAddGuide`. | Unchanged. Those class names are the current Comfy LTX-2 nodes. |
| `mcp/slate-mcp.mjs` | Electron studio MCP. | No LTX stack strings. |

`LTXV*` class names (`LTXVConditioning`, `EmptyLTXVLatentVideo`, `LTXVImgToVideo`, `LTXVAddGuide`, and the audio nodes) are the Comfy node titles on the **LTX-2.3** graphs. They are not the retired LTX-Video 0.9 checkpoints.

## Drift

1. **New-scene graphs are still 2.3 distilled, not 2.5.** `default-video`, `default-i2v`, and `default-flf2v` load `ltx-2.3-22b-distilled-fp8.safetensors`, `gemma_3_12B_it_fp4_mixed.safetensors`, and `ltx-2.3-22b-distilled-1.1_lora-dynamic_fro09_avg_rank_111_bf16.safetensors`. That matches this machine’s Video Buddy Comfy install. LTX-2.5 needs a new API graph. This change does not rewrite `workflow.api.json`.
2. **No extend/retake pack.** IC-LoRA / lipsync / movie-builder graphs are still unshipped (`docs/STATUS.md`). When one is added for an existing plate, pin LTX-2.3 Pro only.
3. **Profile variant labels.** `data/model-profiles.json` lists `ltx-2.3-22b-fast` and `ltx-2.3-22b-pro`. Those strings are not files in the Lightricks LTX-2.3 model card. The factory filename is `ltx-2.3-22b-distilled-fp8.safetensors`.
4. **`generate_audio` on the profile** was described as a third-party API-wrapper flag. The checked-in graphs always build a joint audio latent. The note now says that. The param name remains so older compiled params still parse.

## Explicit non-hits

- No `api.ltx.io`.
- No `/v1/text-to-video`, `/v1/image-to-video`, `/v1/audio-to-video`, `/v1/retake`, `/v1/extend`, or `/v2/…` generation URLs.
- No `ltxv-13b`, `ltxv-2b`, or `ltx-video-2b-v0.9` checkpoint id.
- No T5-XXL wired as an LTX text encoder.
