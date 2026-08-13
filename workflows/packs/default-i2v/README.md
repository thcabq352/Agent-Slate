# default-i2v pack

LTX 2.3 distilled **image-to-video**. Factory generates a Flux keyframe (or reuses the last still / previous-shot frame) then animates it.

| Logical | Node | Field |
|---------|------|--------|
| `image` | `8` VHS_LoadImagePath | path |
| `positive` / `negative` | `10` / `11` | text |
| `width` / `height` / `frames` | `20` LTXVImgToVideo | + audio frames mirror |
| `seed` | `42` | `noise_seed` |

Same checkpoints as `default-video`. `slate_run_pack` `{ pack_id: "default-i2v", image: "C:/path/to.png", positive: "…" }`.
