# Prompting research — Seedance 2.0, MiniMax H3, FLUX 3

Internal note for **studio Deliver compile** (`data/model-profiles.json`). Not Comfy factory packs. Still current as of **2026-08-13** (v0.3.2) for those three profiles. Factory graphs live in `workflows/packs/`.

Convention used throughout the profiles: a `null` limit or a `false` feature
flag means *undocumented — do not rely on it*, not *confirmed absent*.

---

## Seedance 2.0 (ByteDance)

### Sources

Official (primary):

- Doubao Seedance 2.0 prompt guide —
  <https://docs.volcengine.com/docs/82379/2222480> (page last updated 2026-07-20)
- Doubao Seedance 2.0 tutorial / quickstart —
  <https://docs.volcengine.com/docs/82379/2291680>
- Volcano Engine article, text-to-video prompt writing —
  <https://www.volcengine.com/article/40840>

Both docs pages are client-rendered; they return an empty shell to a plain
fetch and need a rendering fetch to read.

Secondary (corroboration only): assorted platform write-ups agreed on the
9 images / 3 videos / 3 audio reference limit and the 15s ceiling.

### What the official guide says

**Advanced formula (verbatim ordering).** `precise subject + action detail +
scene/environment + light and colour + camera movement + visual style + image
quality + constraint words`. The guide frames the model as reading a *spatial
layer* (what is in frame) and a *temporal layer* (what changes over time)
separately, and says a good prompt is an engineering instruction rather than
descriptive copy.

**Reference binding.** Define subjects before using them:
`define [core features] in <Image/Video N> as <Subject N>`, using 2–3 stable
static traits (clothing, hairstyle, appearance, category). Reuse the same label
every time. For undefined subjects, tag inline as `Name@Image1`. Asset-library
IDs cannot substitute for `<Image N>` — the model cannot associate them.
Put the reference that most needs precise matching earliest in the prompt.

**Multi-shot.** Split into `Shot 1 / Shot 2 / Shot 3` in event order. Each shot
is organised as: camera move or cut → subject action and expression → position
or spatial change → audio for that shot. The guide explicitly warns that
support for precise times (e.g. "0–3 s") is unstable and that forcing segment
durations can produce abnormal output. Note the tension: ByteDance's own
quickstart example prompt *does* use second ranges ("2-4s", "4-6s", "6-8s") for
a multimodal-reference ad. Shot labels are the safer default.

**Camera.** Standard cinematography terms are understood directly (medium shot,
close-up, wide, slow push-in, steady lateral move, locked-off). One camera move
per shot — combining push/pull/pan/tilt increases instability.

**Action.** Specify the body part plus amplitude, speed and force. Prefer slow,
gentle, continuous small movements; avoid sprinting, large jumps and violent
tumbling. Write the inertia linking one action to the next. Externalise emotion
as physical detail rather than abstract words — the guide ships a table mapping
"sad" to "head lowered, shoulders trembling, fingers gripping the hem".

**Punctuation channels for audio and text.** Documented markup:

| Content   | Marker | Example |
|-----------|--------|---------|
| Music     | `( )`  | `(fast-paced rock plays in the background)` |
| Sound FX  | `< >`  | `<a dog barks in the distance>` |
| Dialogue  | `{ }`  | `{hello, world}` — name the language for anything other than Chinese/English |
| Subtitle  | `【 】` | `【Chapter One: Departure】` |

**Constraints instead of a negative field.** No negative prompt exists. The
guide supplies constraint templates: "keep it subtitle-free" / "avoid
generating any text or subtitles", "do not generate a logo", "do not generate a
watermark". It notes 100% suppression is not achievable, and that landscape
framings leak subtitles noticeably less often than vertical.

**Asset strategy.** Four functional roles — character anchor, scene tone,
camera reference, rhythm/mood (audio). Recommended total is 4–5 assets: 1–2
character images (headshot + full body) + 1 scene image + 1 camera-move clip +
1 audio clip. Filling the reference cap is explicitly discouraged: too many
assets make feature priority ambiguous.

**Documented failure modes and fixes.** Identity drift → supply a bare headshot
with no wardrobe or background, never a multi-view turnaround sheet (the model
reads multiple angles as multiple people). Duplicate "twin" characters → label
each character with its source image and append a global no-duplicate
constraint; never paste a full script as the prompt. Style drift → explicit
style lock ("2D anime style"). Extension seams → trim 6 frames off the tail of
the earlier clip and 1 frame off the head of the next. Quality decay when
re-extending a generated clip → convert to a white-model pass first, or limit
extension chains. Effects that miss → supply a reference video of the effect
rather than describing it.

**Long take vs cut.** Video extension for single-scene dialogue and emotional
progression; separate generations spliced in edit for action, chases and
montage.

### Specs verified from the tutorial page

- Model IDs: `doubao-seedance-2-0-260128`, `doubao-seedance-2-0-fast-260128`,
  `doubao-seedance-2-0-mini-260615`
- Duration 4–15 s (all three variants); output mp4
- `ratio`: 21:9, 16:9, 4:3, 1:1, 3:4, 9:16 — no auto/adaptive value listed
- `resolution`: 480p / 720p / 1080p / 4k (10-bit) on Seedance 2.0;
  480p / 720p only on Fast and Mini
- `generate_audio`, `return_last_frame`, `service_tier: "flex"` params
- `content` array of typed items with roles `reference_image`,
  `reference_video`, `first_frame`, `last_frame`
- References: 0–9 images, 0–3 videos, 0–3 audio. "Text + audio" and audio-only
  inputs are rejected. Video extension chains at most 3 clips, 15 s total.

### Profile changes

- `dialect.guidance` rewritten around the official formula, subject binding,
  shot labelling, the punctuation channels, one-move-per-shot, action
  granularity, constraint words and the 4–5 asset ceiling.
- `limits.aspectRatios` — dropped `"auto"` (not in the official table).
- `limits.resolutions` — `["720p","1080p","2K"]` → `["480p","720p","1080p","4k"]`.
- `params` replaced with the documented parameter set (model IDs, duration,
  ratio, resolution, generate_audio, return_last_frame, service_tier, content,
  reference limits).
- `notes` rewritten. Two prior claims corrected: the "up to 12 mixed
  references" figure (secondary-sourced; actually 9/3/3), and the claim that
  native audio generation was a 2.5-only capability — `generate_audio` is
  documented for 2.0.
- `features.timecodeBeats` left `true`, with the nuance documented in `notes`.
- Still `null`/`false`: `maxChars`, `fps`, `seeds`.

---

## MiniMax Hailuo H3 (MiniMax)

### Sources

Official (primary):

- Video Generation guide —
  <https://platform.minimax.io/docs/guides/video-generation>
- Create Video Generation Task (V2) API reference —
  <https://platform.minimax.io/docs/api-reference/video-generation-v2-create>
- MiniMax H3 launch/research post — <https://www.minimax.io/blog/minimax-h3>
- Open weights — <https://huggingface.co/MiniMaxAI/MiniMax-H3>

MiniMax publishes no standalone prompt guide. The prompting content is a few
lines inside the Video Generation guide plus the worked example in the launch
post.

### What the official docs say

**Prompt as a relationship statement.** The launch post's example prompt is the
canonical pattern: *"Reference the Hitchcock camera movement from Video 1, have
the character in Image 2 sing, with the vocals matching Audio 3."* References
are numbered in upload order and the prompt states what each contributes;
MiniMax's framing is that language is the bridge, so the relationship between
context and target is described in words rather than selected via task flags.

**Camera control.** The guide documents bracketed camera-motion instructions —
`[pan]`, `[zoom]`, `[static]` — placed **directly after the description they
modify**. This is the only prompt-side control MiniMax publishes.

**No timecode syntax.** Nothing in MiniMax's docs describes timecodes, shot
markers or beat ranges. Multi-shot generation is real (the research post
discusses audio-visual relationships "across multiple shots") but has no
documented notation.

**Audio.** Native stereo generated jointly with picture; dialogue, SFX and
music are not separate domains and there is no audio toggle in the API.

**Prompt expansion.** The `H3-Context-IR` endpoint reads the same multimodal
content and returns an expanded structured prompt in `content.prompt` without
generating video — a usable prompt-compilation step.

**Mode exclusivity.** `first_frame`/`last_frame` roles and
`reference_*` roles cannot appear in the same request. Reference audio cannot
be sent alone; at least one reference image or video is required.

### Specs verified

- `model`: `MiniMax-H3`; endpoint `/v2/video_generation`, async create → poll
- `duration`: required integer, **4–15 s**
- `resolution`: `768P` or `2K` — both generally available; 2K is the default
  tier, 768P the cheaper one
- `ratio`: `adaptive` (default), 21:9, 16:9, 4:3, 1:1, 3:4, 9:16.
  Text-to-video requires a concrete ratio and rejects `adaptive`;
  first/last-frame mode always behaves as `adaptive`
- Prompt length ≤ 7000 characters; exactly one non-empty `text` item required
- References: ≤ 9 images, ≤ 3 videos, ≤ 3 audio, **≤ 12 files total**; video and
  audio 2–15 s per clip and ≤ 15 s combined; image/video ratio in [0.4, 2.5];
  input dimensions in [256, 5760]
- File caps: video ≤ 50 MB, image ≤ 30 MB, audio ≤ 15 MB, request body ≤ 64 MB
- Input video frame rate accepted in [23.976, 60]. **No output fps is published.**
- `Regeneration` task upscales a 768P result to 2K by resubmitting the original
  `content` plus one `role=base_video` item
- Optional `callback_url` with a 3-second challenge echo

### Profile changes

- `dialect.guidance` rewritten: reference numbering and relationship phrasing,
  bracketed camera cues as the published control, audio direction, the 7000-char
  budget, mode exclusivity, and the H3-Context-IR pre-pass. The previous
  guidance asserted the opposite of the documentation on two points — it told
  the model to write timed beats, and it said bracketed director commands were
  *not* the documented interface. Both corrected.
- `limits.durations` — floor moved 5 → **4** (stated in the API reference).
- `limits.aspectRatios` — `"auto"` → `"adaptive"` (the real enum value).
- `limits.fps` — `24` → `null`. MiniMax publishes no output frame rate; the 24
  figure was secondary-sourced.
- `limits.resolutions` — `["1440p (2K)"]` → `["768P", "2K"]`. 768P is GA, not
  closed beta.
- `features.timecodeBeats` — `true` → `false` (undocumented; do not rely on it).
- `params` replaced with the documented set; `prompt_optimizer` removed (a
  Hailuo 2.3 parameter, absent from the H3 V2 API); added `content`,
  `h3_context_ir`, `regeneration`, `callback_url`.
- `notes` rewritten, including the open-weights release. The exact open-weights
  date was not confirmed from a MiniMax-owned page — the launch post only says
  "in the coming days" — so the note points at the Hugging Face repo without
  asserting a date.
- Still `null`/`false`: `fps`, `seeds`, `negativePrompt`.

---

## FLUX 3 (Black Forest Labs)

### Sources

Official (primary):

- FLUX 3 launch post — <https://bfl.ai/blog/flux-3>
- Documentation index — <https://docs.bfl.ml/llms.txt>
- Building a Good Prompt — <https://docs.bfl.ai/guides/prompting_unified_building>
- Technical Parameters — <https://docs.bfl.ai/guides/prompting_unified_technical>
- BFL agent skills repo — <https://github.com/black-forest-labs/skills>

### Finding: there is no FLUX 3 prompting documentation

Re-checked 2026-08-04. The docs index lists **no** FLUX 3 page, no video
endpoint, and no video prompting guide — every prompting and API-reference entry
covers FLUX.1, FLUX.1 Kontext or FLUX.2. BFL's own agent-skills repo
(`flux-best-practices`, `bfl-api`) likewise covers FLUX.1/FLUX.2 only. FLUX 3
Video remains gated early access.

The launch post is the only first-party FLUX 3 material. It states: clips up to
20 seconds; native audio with all video output; modes text-to-video,
image-to-video, video-to-video, video/audio continuation and keyframe-to-video;
multilingual dialogue; multi-shot sequences via agentic chaining; "a broad range
of visual styles and aspect ratios"; and 720p as the setting used for
preliminary evaluations. It gives **no** prompting instructions, no parameter
table, no fps and no aspect-ratio enum.

### Family-level guidance carried over (explicitly not FLUX 3 docs)

From *Building a Good Prompt*:

- Component order: `[SUBJECT], [LOCATION], [STYLE], [CAMERA SETTINGS],
  [LIGHTING], [COLORS], [EFFECT], [ADDITIONAL ELEMENTS]` — described as a
  building aid, not a rule.
- Length tiers: short 10–30 words (quick concepts), medium 30–80 (most scenes),
  long 80–300+ (complex multi-subject). "Start short. Add only what changes the
  image." / "Specific detail helps. Filler hurts."
- Reliable pattern: clear subject → main action or state → mood/context/visual
  direction only when it improves the result.

From *Technical Parameters*:

- "Most FLUX models do not support negative prompts", and negation misfires
  anyway. Documented replacement: identify the unwanted element, ask what would
  fill that space, describe that positively ("no crowds" → "empty pathways").

Also documented for the family: quotation marks around literal on-screen text,
and hex codes for exact colour matching.

### Profile changes

- `dialect.guidance` rewritten to use BFL's own component ordering, its length
  tiers and short-first rule, and its positive-replacement strategy for
  negatives, while keeping the video-specific parts (one audio cue always, name
  the dialogue language, one action per clip, chain for longer sequences, prose
  over tag lists). Added the quoted-text and hex-colour conventions.
- `params[resolution].note` — clarified that 720p is the only figure BFL names
  and that it appears only as an evaluation setting; 480p is provider-sourced.
- `notes` extended with the negative finding above, and with an explicit
  statement that the guidance draws on FLUX.2-era family guides applied as
  conventions rather than as documented FLUX 3 Video behaviour.
- `limits` and `features` otherwise **unchanged** — no authoritative evidence
  was found to change them. `maxChars`, `aspectRatios` and `fps` stay `null`;
  `seeds`, `cameraControls`, `timecodeBeats` and `negativePrompt` stay `false`.

---

## Cross-model observations

- Seedance 2.0 and MiniMax H3 have converged on the same reference budget
  (9 images / 3 videos / 3 audio) and the same "Image N / Video N / Audio N"
  addressing convention in the prompt. A compiler can share that layer.
- Neither vendor exposes a negative prompt. Both expect exclusions phrased as
  positive constraints; only Seedance publishes concrete constraint templates.
- Timecodes are the weakest common ground: Seedance documents shot labels and
  warns against forced timings, MiniMax documents nothing at all, and FLUX 3
  has no documentation. Prefer ordered prose or shot labels over second markers
  everywhere.
- Only Seedance publishes a punctuation grammar for audio and subtitles. Do not
  carry `( ) < > { } 【 】` into the other two dialects.

## Unverified after this pass

- Seedance 2.0: max prompt length, output fps, seed support.
- MiniMax H3: output fps, seed support, exact open-weights release date.
- FLUX 3: essentially everything — prompt length, fps, aspect-ratio enum, seed
  support, reference count, real parameter names, and any prompting guidance at
  all. Recheck when BFL opens FLUX 3 documentation.

## Addendum (2026-08-04): Official Dreamina Seedance 2.5 Prompt Guide

Source: ByteDance Feishu doc (updated Jul 31) — https://bytedance.larkoffice.com/docx/A88jd0B47oAd8zxWp5ycZFMfnxh (via t.co link from Sam).

Incorporated into the `seedance-2` profile (relabeled "Seedance 2.5 (Dreamina)"):
- Core formula ordering + omit-what-you-don't-need; parameters never in the prompt.
- @-role reference mapping with use-only/do-not-use exclusions; name-and-bind every subject; [Subject Profile] blocks; select references per scene.
- Long-video [Stage] blocks: ONE primary change per stage + explicit visible End state; time ranges are budgets, not edit points. Up to 30s, up to 50 reference materials (30 img / 10 vid ≤30s / 10 audio ≤30s).
- Audio syntax: (music), <SFX>, {dialogue}, 【subtitles】; dialogue-language reinforcement formula.
- Emotional direction via 2–4 observable cues; uncommon camera terms translated to subject + visible change; supported technique list (dolly zoom, FPV, bullet time, whip-pan, bounce speed ramp…).
- Editing (sole-editing-master pattern), extension (boundary-frame alignment), first/last-frame + keyframes, storyboard grids, coarse/fine blockout re-rendering, one-click video, seamless transitions, and the pre-submission checklist.


## Addendum — MiniMax H3 official prompt guides (August 4, 2026)

MiniMax published two official prompt-writing guides alongside the open weights
at huggingface.co/MiniMaxAI/MiniMax-H3:

- `docs/VIDEO_PROMPT_WRITING_GUIDE_base_en.md` — the three-field prompt format
  (integrated_multimodal_description / overall_soundscape / non_diegetic_music),
  `[Shot N]` markers with `MM:SS.SSS` timestamps, the camera-move vocabulary with
  amplitude/speed modifiers, `(Sx)` speaker ids with `<d>[Language] ...</d>`
  dialogue, `<scenetrans>` / `<cutoff>` tags, quoted on-screen text, and the
  task-instruction first lines for image-, first/last-frame- and last-frame-to-video.
- `docs/VIDEO_PROMPT_WRITING_GUIDE_ref_en.md` — the reference-label grammar:
  `<Subject N>` / `<Picture N>` / `<Video N>` / `<Audio N>`, the bracketed
  task-type summary, and retention_analysis markers.

Both are folded into the `minimax-h3` profile. They supersede the earlier finding
that MiniMax documents no shot-marker syntax (that was true of the platform API
docs; the open-weights guides define one), so `timecodeBeats` is back to true and
Slate beat sheets compile to H3 shot markers.
