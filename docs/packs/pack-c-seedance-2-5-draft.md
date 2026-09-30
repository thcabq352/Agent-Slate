# Pack C — Seedance 2.5 one-take (local only)

Staged 2026-09-30. Updated the same day: **local-only is a hard requirement.** Agents running this route use no cloud APIs, no cloud services, and no hosted inference.

## Route the agent runs

1. Write the Pack C prompt locally.
2. Scout and move on local Comfy `http://127.0.0.1:8188` with `slate-engine` packs `default-still`, `default-i2v`, `default-video`, `default-flf2v`.
3. Reuse an explicit seed when repeating the same shot. `generate_audio` stays off when post owns audio.

Skill: [`skills/slate-seedance-draft/SKILL.md`](../../skills/slate-seedance-draft/SKILL.md). Rule: [`references/local-only.md`](../../skills/slate-seedance-draft/references/local-only.md).

## Prompt

```
16:9, cinematic, single continuous take. Refs: @Image1 character, @Image2 location, @Clay Render 1 blocking. 0–10s: … 10–20s: … 20–30s: …. No subtitles, No BGM. Keep identity/wardrobe locked.
```

Filled text: [`workflows/partner/seedance-2-5-draft/prompt-template.txt`](../../workflows/partner/seedance-2-5-draft/prompt-template.txt).

One continuous take. A multi-body fight stays out of one 30s pass.

## Field-shape record (not an execution route)

`workflows/partner/seedance-2-5-draft/` still records the Partner node shape from ComfyUI #16529: model option `Seedance 2.5 Draft`, model id `dreamina-seedance-2-5-260628`, `draft: true`, 480p scout, `draft_task_id`, promote node `ByteDance2DraftToFinalVideoNode` (display name ByteDance Seedance 2.5 Draft to Final Video), templates `api_seedance2_5_draft_{t2v,i2v,r2v}`.

Those class types call hosted inference. Graphs stay bypassed (mode `4`). `contract.json` sets `localOnly.agentExecution` to `forbidden`. The agent does not enable or queue them.

`data/model-profiles.json` → `seedance-2` remains the studio Deliver profile.
