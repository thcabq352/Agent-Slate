---
name: slate-seedance-draft
description: "Local-only Seedance one-take on Comfy."
version: 0.2.0
author: thcabq352, Hermes Agent
license: Apache-2.0
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [film, seedance, comfyui, draft, slate, local]
    category: media
    related_skills: [slate-film-factory]
    homepage: https://github.com/thcabq352/Agent-Slate
---

# Seedance 2.5 one-take — local only

Pack C prompt for a 16:9 cinematic single continuous take. **Local-only is a hard requirement.** The agent uses no cloud APIs, no cloud services, and no hosted inference.

Generate on local Comfy at `http://127.0.0.1:8188` through `slate-engine` and `workflows/packs/` (`default-still`, `default-video`, `default-i2v`, `default-flf2v`).

`workflows/partner/seedance-2-5-draft/` is a field-shape record of the Partner nodes (`Seedance 2.5 Draft`, model id `dreamina-seedance-2-5-260628`, 480p, `draft_task_id`, `ByteDance2DraftToFinalVideoNode`). Those nodes are hosted inference. They are not an execution route.

Rule file: `skill_view("slate-seedance-draft", "references/local-only.md")`.

## When to Use

- A 20–30s one-take prompt in the Pack C shape, rendered on local Comfy
- The user names Seedance 2.5 Draft, `draft_task_id`, or Draft to Final

## Prerequisites

1. Local ComfyUI at `http://127.0.0.1:8188` and a built `slate-engine`.
2. Prompt: `skill_view("slate-seedance-draft", "references/prompt-pack.md")`.
3. Local-only rule: `skill_view("slate-seedance-draft", "references/local-only.md")`.

## How to Run

1. Write the Pack C prompt. One continuous take, ref labels, three timed beats, `No subtitles, No BGM.`, identity lock. `generate_audio` stays off when post owns audio.
2. Scout a plate with local pack `default-still` when the shot needs a still. Same prompt and refs.
3. Move with local pack `default-i2v` (plate → clip) or `default-video` (text → clip). Fix the seed in the local pack manifest sense: pass an explicit `seed` and reuse it if you regenerate the same shot.
4. Call `slate_film_factory` / `slate_run_pack` / `slate_generate_shot` only against those pack ids. Comfy base URL stays `http://127.0.0.1:8188`.

## Procedure

1. Confirm the shot is one continuous take. A multi-body fight stays out of one 30s pass.
2. Bind `@Image1` character, `@Image2` location, `@Clay Render 1` blocking in the prompt.
3. Render with the local packs above.
4. Leave `workflows/partner/seedance-2-5-draft/` on disk. Do not submit those graphs.

## Pitfalls

- `ByteDance2TextToVideoNode`, `ByteDance2FirstLastFrameNode`, `ByteDance2ReferenceNodeV2`, and `ByteDance2DraftToFinalVideoNode` are hosted Partner nodes. Queueing them leaves the machine.
- Templates `api_seedance2_5_draft_t2v`, `api_seedance2_5_draft_i2v`, and `api_seedance2_5_draft_r2v` name that hosted shape. They are not local checkpoints.
- Local LTX packs are short clips. They do not call Seedance and they do not promote a `draft_task_id`.

## Verification

- [ ] Prompt contains `16:9, cinematic, single continuous take.`, `@Image1` / `@Image2` / `@Clay Render 1`, `0–10s` / `10–20s` / `20–30s`, `No subtitles, No BGM.`, and `Keep identity/wardrobe locked.`
- [ ] Generate target is `http://127.0.0.1:8188` and a `workflows/packs/` id
- [ ] No Partner node was enabled or queued
- [ ] No call to comfy.org, BytePlus, or ModelArk
