# default-i2v pack

LTX 2.3 distilled **image-to-video** for Agent-Slate. Factory generates a Flux keyframe (or reuses the last still / previous-shot frame) then animates it. Index: [../README.md](../README.md) · [docs/STATUS.md](../../../docs/STATUS.md).

| Logical | Node | Field |
|---------|------|--------|
| `image` | `8` VHS_LoadImagePath | path |
| `positive` / `negative` | `10` / `11` | text |
| `width` / `height` / `frames` | `20` LTXVImgToVideo | + audio frames mirror |
| `seed` | `42` | `noise_seed` |

Same checkpoints as `default-video`. `slate_run_pack` `{ pack_id: "default-i2v", image: "C:/path/to.png", positive: "…" }`.

## Local LTX routing

Local Comfy only. This pack does not call LTX Cloud.

| Job | Local stack |
| --- | --- |
| New scene, multi-cut, or synced dialogue | **LTX-2.5** (split weights, Gemma 4). This graph is still the installed 2.3 distilled + Gemma 3 stack. |
| Extend or retake an existing plate | **LTX-2.3 Pro only.** I2V from a still is a new plate, not an extend/retake. |

Inventory: [docs/ltx-local-stack-audit.md](../../../docs/ltx-local-stack-audit.md).
