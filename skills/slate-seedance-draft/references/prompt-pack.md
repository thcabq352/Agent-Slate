# Pack C prompt

One paragraph. Parameters stay on the nodes (`resolution`, `ratio`, `duration`, `seed`, `generate_audio`). They do not go in the prompt text.

## Shape

```
16:9, cinematic, single continuous take. Refs: @Image1 character, @Image2 location, @Clay Render 1 blocking. 0–10s: … 10–20s: … 20–30s: …. No subtitles, No BGM. Keep identity/wardrobe locked.
```

## Filled template

The Desk graphs ship this text in `model.prompt` (same string as `workflows/partner/seedance-2-5-draft/prompt-template.txt`):

```
16:9, cinematic, single continuous take. Refs: @Image1 character, @Image2 location, @Clay Render 1 blocking. 0–10s: the character crosses into the location and lands the clay-blockout mark. 10–20s: one continuous move, camera stays with them, wardrobe unchanged. 20–30s: they settle on the end mark in the same light. No subtitles, No BGM. Keep identity/wardrobe locked.
```

Replace the three beats with the shot. Keep the opener, the ref line, the closers, and a single continuous take.

## Rules that travel with the shape

- Refs on the scout and the promote are the same set. R2V wires `@Image1` → `model.reference_images.image_1` (`character.png`), `@Image2` → `image_2` (`location.png`), `@Clay Render 1` → `image_3` (`clay-blocking.png`).
- Beats are time ranges inside one take, not cut points. Avoid a multi-body fight in one 30s pass.
- `No subtitles, No BGM.` Post owns the soundtrack. Set node `model.generate_audio` to `false` in that case. Set it `true` only when the take must carry its own dialogue or effects; the 1080p final inherits that flag.
- Close with the identity lock: `Keep identity/wardrobe locked.`
- Duration widget: whole seconds, 4–30. The filled template uses 30 so the three beats cover the clip.

## Studio compile profile

`data/model-profiles.json` id `seedance-2` stays the Dreamina section formula for Deliver. Pack C is this one-take string for the Partner Draft route. Do not rewrite that profile to force this shape onto every Seedance compile.
