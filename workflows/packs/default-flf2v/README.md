# default-flf2v pack

LTX 2.3 distilled **first + last frame → video** (`LTXVAddGuide` at 0 and −1, then `LTXVCropGuides`).

| Logical | Node | Field |
|---------|------|--------|
| `image` | `8` | start frame path |
| `image_end` | `18` | end frame path |
| `positive` / `negative` | `10` / `11` | text |
| size / frames / seed | `20` / `21` / `42` | same as T2V |

Factory: start = this shot’s still (or generated keyframe); end = next shot still if any, else the same keyframe.

`slate_run_pack` `{ pack_id: "default-flf2v", image, image_end, positive }`.
