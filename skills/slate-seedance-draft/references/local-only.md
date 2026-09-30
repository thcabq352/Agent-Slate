# Local-only hard requirement

Agents operating Pack C run entirely on this machine.

- Cloud APIs: forbidden
- Cloud services: forbidden
- Cloud-hosted inference: forbidden

Allowed generate path:

| Piece | Value |
|-------|--------|
| Comfy | `http://127.0.0.1:8188` |
| Engine | `slate-engine` (`slate_film_factory`, `slate_run_pack`, `slate_generate_shot`) |
| Packs | `default-still`, `default-video`, `default-i2v`, `default-flf2v` under `workflows/packs/` |

`workflows/partner/seedance-2-5-draft/` records Partner field names (`Seedance 2.5 Draft`, `draft: true`, 480p, `draft_task_id`, `ByteDance2DraftToFinalVideoNode`). Those class types call hosted ByteDance inference. The agent does not load them, does not set their mode to `0`, does not queue them, and does not sign in to comfy.org, BytePlus, or ModelArk.

Scout and promote for a take the agent actually renders:

1. Write the Pack C prompt locally.
2. Scout a still with `default-still` (local Flux) when a plate is needed.
3. Move with `default-i2v` or `default-video` (local LTX) on the same prompt and refs.
4. Keep `generate_audio` off when post owns audio. Local packs do not call a hosted audio model for this route.
