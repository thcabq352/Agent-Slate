---
name: slate-seedance-draft
description: "Seedance 2.5 one-take: Draft 480p then native 1080p."
version: 0.1.0
author: thcabq352, Hermes Agent
license: Apache-2.0
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [film, seedance, comfyui, draft, slate]
    category: media
    related_skills: [slate-film-factory]
    homepage: https://github.com/thcabq352/Agent-Slate
---

# Seedance 2.5 one-take — Draft to Final

Pack C for a 16:9 cinematic single continuous take, up to 30 seconds, on Comfy Partner. Scout with model option **Seedance 2.5 Draft** (`draft: true`, 480p). Promote with `draft_task_id` on **ByteDance Seedance 2.5 Draft to Final Video** (native 1080p). Model id for both stages: `dreamina-seedance-2-5-260628`.

This skill writes the prompt and the Desk attach contract. It does not queue Comfy, does not call Partner, and does not spend.

## When to Use

- A 20–30s one-take that should scout at 480p and promote to native 1080p
- Comfy Desk / Partner nodes, templates `api_seedance2_5_draft_t2v`, `api_seedance2_5_draft_i2v`, `api_seedance2_5_draft_r2v`
- The user names Seedance 2.5 Draft, `draft_task_id`, or Draft to Final

Local Flux/LTX factory shots stay on `slate-film-factory` (`default-still`, `default-i2v`, `default-flf2v`, `default-video`).

## Prerequisites

1. Comfy Desk with Partner nodes from ComfyUI that includes Seedance 2.5 Draft (ComfyUI PR #16529, workflow templates #1288).
2. Prompt and refs from this skill. Full shape: `skill_view("slate-seedance-draft", "references/prompt-pack.md")`.
3. Desk contract: `skill_view("slate-seedance-draft", "references/comfy-desk.md")`.

Repo graphs, when this checkout is available: `workflows/partner/seedance-2-5-draft/`.

## How to Run

1. Pick one template: text-only → `t2v`; start plate → `i2v`; character + location + blocking refs → `r2v`.
2. Fill the Pack C prompt. Keep one continuous take, the three ref labels, three timed beats, `No subtitles, No BGM.`, and the identity lock.
3. Hand the operator the Desk graph (repo JSON or the official template id) with this contract applied:
   - Model option `Seedance 2.5 Draft` (that selection sends `draft: true`)
   - Resolution `480p` (locked on Draft)
   - Same prompt and the same refs on the scout that you intend to promote
   - `seed` integer chosen for the scout, `control_after_generate` = `fixed` before any promote
   - `model.generate_audio` = `false` when post owns audio
4. Stop before queue. The operator enables Stage 1 on their Desk when they choose to spend.
5. After they return a `draft_task_id`, remind them it lasts about 7 days. Promote by pasting that id into `ByteDance2DraftToFinalVideoNode`. Stage 2 has no prompt field; 1080p reuses the draft.

## Quick Reference

| Step | Node | Class | Setting |
|------|------|--------|---------|
| Scout T2V | ByteDance Seedance 2.5 Text to Video | `ByteDance2TextToVideoNode` | `Seedance 2.5 Draft`, `480p`, ratio `16:9` |
| Scout I2V | ByteDance Seedance 2.5 First-Last-Frame to Video | `ByteDance2FirstLastFrameNode` | `Seedance 2.5 Draft`, `480p`, 16:9 first frame, `last_frame` empty |
| Scout R2V | ByteDance Seedance 2.5 Reference to Video | `ByteDance2ReferenceNodeV2` | `Seedance 2.5 Draft`, `480p`, ratio `16:9`, `task_type` `reference` |
| Save id | Save Text | `SaveText` | `draft_task_id` string output |
| Promote | ByteDance Seedance 2.5 Draft to Final Video | `ByteDance2DraftToFinalVideoNode` | paste `draft_task_id` → native `1080p` |

## Procedure

1. Confirm the shot is one continuous take. Skip a 30s pass that is a multi-body fight.
2. Write the prompt in the Pack C shape. Bind `@Image1` character, `@Image2` location, `@Clay Render 1` blocking.
3. Set duration to the beat window (4–30 seconds; this pack's example is 30).
4. Record the seed integer and set the Desk control to **fixed** before promote so the next Stage 1 queue does not mint a new `draft_task_id`.
5. Point at the matching graph under `workflows/partner/seedance-2-5-draft/` or at official template `api_seedance2_5_draft_{t2v,i2v,r2v}`.
6. Deliver the prompt, the template id, and the promote reminder. Do not POST `/prompt`. Do not call `slate_film_factory` or `slate_run_pack` for this route.

## Pitfalls

- `draft_task_id` connected while the model option is anything other than `Seedance 2.5 Draft` makes the scout node error. Disconnect that output, or select Draft.
- Re-queueing Stage 1 creates a new draft. After the preview is approved, bypass Stage 1 and queue only Stage 2.
- The final node inherits prompt, refs, duration, aspect ratio, seed, and `generate_audio`. Pasting a different prompt into Stage 2 is impossible; that node only accepts `draft_task_id` and `watermark`.
- I2V on Seedance 2.5 has no ratio widget. The first frame's aspect is the clip aspect, so the plate is 16:9.
- Draft ids last about 7 days.
- Partner nodes need a comfy.org sign-in. This skill does not log in and does not submit a task.
- Seed on these nodes controls whether Comfy re-runs the node. The node tooltip says results are non-deterministic regardless of seed. Fixed seed is how you keep the approved draft id.

## Verification

- [ ] Prompt contains `16:9, cinematic, single continuous take.`, `@Image1` / `@Image2` / `@Clay Render 1`, `0–10s` / `10–20s` / `20–30s`, `No subtitles, No BGM.`, and `Keep identity/wardrobe locked.`
- [ ] Scout model option is `Seedance 2.5 Draft`, resolution `480p`, model id `dreamina-seedance-2-5-260628`
- [ ] `control_after_generate` is `fixed` before promote
- [ ] `generate_audio` is `false` when post owns audio
- [ ] Promote target is `ByteDance2DraftToFinalVideoNode` / display name `ByteDance Seedance 2.5 Draft to Final Video`
- [ ] No factory pack id was queued and no Partner task was submitted from this skill
