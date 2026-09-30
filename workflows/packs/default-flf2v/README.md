# default-flf2v pack

LTX 2.3 distilled **first + last frame → video** (`LTXVAddGuide` at 0 and −1, then `LTXVCropGuides`). Agent-Slate pack. Index: [../README.md](../README.md) · [docs/STATUS.md](../../../docs/STATUS.md).

| Logical | Node | Field |
|---------|------|--------|
| `image` | `8` | start frame path |
| `image_end` | `18` | end frame path |
| `positive` / `negative` | `10` / `11` | text |
| size / frames / seed | `20` / `21` / `42` | same as T2V |

Factory: start = this shot’s still (or generated keyframe); end = next shot still if any, else the same keyframe.

`slate_run_pack` `{ pack_id: "default-flf2v", image, image_end, positive }`.

## Local LTX routing

Local Comfy only. This pack does not call LTX Cloud.

| Job | Local stack |
| --- | --- |
| New scene, multi-cut, or synced dialogue | **LTX-2.5** (split weights, Gemma 4). This graph is still the installed 2.3 distilled + Gemma 3 stack. |
| Extend or retake an existing plate | **LTX-2.3 Pro only.** First/last-frame guidance is a new plate, not an extend/retake. |

Inventory: [docs/ltx-local-stack-audit.md](../../../docs/ltx-local-stack-audit.md).
