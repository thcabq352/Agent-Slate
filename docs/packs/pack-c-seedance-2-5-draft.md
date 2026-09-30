# Pack C — Seedance 2.5 one-take + Draft (Comfy Partner)

Staged 2026-09-30 from the Cut List digest. This is a Hermes prompt pack plus a Comfy Desk attach contract. It is not a live `slate-engine` factory pack and it does not deploy a tower graph.

## Route

20–30s one-take on Seedance 2.5:

1. **Scout.** Model option `Seedance 2.5 Draft` on the T2V, first-last, or reference node. That option is model id `dreamina-seedance-2-5-260628` with `draft: true`. Resolution locks to **480p**. Use the same prompt and the same refs you will promote. Set `control_after_generate` to **fixed** and keep that seed integer before promote.
2. **Promote.** Paste `draft_task_id` into Comfy node **ByteDance Seedance 2.5 Draft to Final Video** (`ByteDance2DraftToFinalVideoNode`). Output is native **1080p**. The draft id lasts about **7 days**.
3. **Templates.** `api_seedance2_5_draft_t2v`, `api_seedance2_5_draft_i2v`, `api_seedance2_5_draft_r2v`.

`generate_audio: false` when post owns audio. The final inherits the scout's audio flag.

## Prompt

```
16:9, cinematic, single continuous take. Refs: @Image1 character, @Image2 location, @Clay Render 1 blocking. 0–10s: … 10–20s: … 20–30s: …. No subtitles, No BGM. Keep identity/wardrobe locked.
```

Filled text used in the Desk graphs: [`workflows/partner/seedance-2-5-draft/prompt-template.txt`](../../workflows/partner/seedance-2-5-draft/prompt-template.txt).

Keep one continuous take. A multi-body fight does not belong in one 30s pass.

## Where the files are

| Path | Role |
|------|------|
| [`skills/slate-seedance-draft/`](../../skills/slate-seedance-draft/) | Hermes skill |
| [`workflows/partner/seedance-2-5-draft/`](../../workflows/partner/seedance-2-5-draft/) | Comfy Desk graphs, API field maps, `contract.json` |

Factory packs that stay on local Comfy are unchanged: `default-still`, `default-video`, `default-i2v`, `default-flf2v`.

## Desk, not the engine

Comfy Partner nodes bill on queue and need a comfy.org login. These JSON files ship with Partner nodes bypassed (mode `4`) so loading them submits nothing. The operator enables Stage 1 for a scout and Stage 2 for the 1080p promote.

`data/model-profiles.json` → `seedance-2` remains the studio Deliver profile (Dreamina section formula). Pack C does not replace it.
