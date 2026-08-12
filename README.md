<div align="center">

<img src="docs/images/logo.png" alt="Slate logo" width="360" />

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License: Apache 2.0"></a>
  <a href="https://github.com/thcabq352/slate/releases"><img src="https://img.shields.io/github/v/release/thcabq352/slate?include_prereleases&label=download" alt="Latest release"></a>
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows-2f7bf6" alt="Platforms">
  <a href="https://ko-fi.com/samwasserman"><img src="https://img.shields.io/badge/Ko--fi-support%20Sam%20Wasserman-ff5e5b?logo=kofi&logoColor=white" alt="Support Sam Wasserman on Ko-fi"></a>
</p>

<p align="center"><strong>Maintained fork</strong> by <a href="https://github.com/thcabq352">thcabq352</a> · Original project by <a href="https://wassermanproductions.com">Sam Wasserman</a> · Upstream: <a href="https://github.com/wassermanproductions/slate">wassermanproductions/slate</a></p>

**The prompt studio for AI filmmaking.** Plan shots, direct coverage, spot your score, cast your voices, keep continuity across an entire film — and compile production-ready prompts for the exact generator you're using.

![Slate — the prompt editor with cinematic syntax highlighting](docs/images/1-editor.png)

</div>

---

Slate doesn't generate images or video. It makes the **prompts** you paste into your generators dramatically better, faster, and consistent across a whole film — the missing pre-production layer between *"I can see the shot"* and the generate button.

You write (or direct) structured, sectioned shot prompts — **Subject · Composition · Lighting · Camera · Style · Mood**, the categories a crew thinks in — with live cinematic syntax highlighting. A local AI brain helps you structure, tighten, enrich, riff, and iterate, always in the context of your film's characters, locations, props, and look. Then Slate compiles each shot, music cue, or voice for a specific target: **Seedance 2.0, Kling, Veo, Sora, Hailuo, LTX, Flux, Midjourney, GPT Image, Krea, ComfyUI, Suno, Eleven Music, Lyria, Stable Audio, ElevenLabs Voice Design, Hume, MiniMax** — each in its own dialect, against its real limits.

- 🎬 **Projects → scenes → shots.** Your film's bible (logline, world, cast, locations, props, style) travels with every prompt. The brain already knows your protagonist's scar and your city's neon.
- 🎛️ **Shot specs as real controls** — length (any seconds), fps, aspect ratio, shot size, angle, lens, movement, optional character budget. Structured fields, compiled correctly per model.
- 🖍️ **A prompt editor that reads like a lit set** — camera terms in cyan, lighting in gold, color in magenta, motion in green, mood in violet. **Picture-Lock** any line and no transform will ever touch it; **mute** a line to keep it without exporting it; highlight a phrase and **reshoot just that span**.
- 🎥 **Coverage Plans** — one scene description becomes a full set of shots: Full, Dialogue, Motion, Extreme Action, Establishing, Surveillance, Entrance, Parallel Action, Dance, Angle, Orbit, Story Beats — or call your own coverage in plain English.
- ⛓️ **Sequence Chunks** — a 3-minute fight becomes ~20-second generation prompts with explicit continuity handoffs (each chunk opens exactly where the last ended), optionally beat-directed with timecodes inside each chunk.
- 🗣️ **Director's Notes** — talk to the shot: *"make it rain, keep the neon."* The prompt updates; the old version goes to history.
- ✦ **First AD (optional)** — a conversational operator. Describe what you're after, hone it together, and it runs the set: scenes, shots, specs, prompts, cast, locations, music cues, voices — with a receipt for every action. Or never open it and drive everything by hand.
- 🧑‍🤝‍🧑 **Casting, Art Department, Locations, Lookbook** — structured sheets with natural-language auto-fill; one-click reference-sheet prompts pull consistent identity sheets from your image generator. Study any cinematographer, director, film, or series into a reusable style profile.
- 🎼 **A Sound Department** — design music cues like a composer spotting a scene (the brain writes tagged lyrics on request) and cast voices with audition text. Drop in **any audio file** and Slate measures it locally — tempo, pitch register, dynamics, brightness, energy arc — then reverse-engineers a matching cue or voice sheet.
- 🖼️ **References** — drop in stills or clips; clips are key-framed locally (ffmpeg) and broken down into element sheets — lensing, lighting, palette, movement — you reuse as one-click ingredients.
- 📦 **Deliverables** — per-model compile with preflight warnings (duration caps, aspect ratios, fps), smart character-budget compression that keeps locked lines verbatim, negative prompts where supported, timecode beats where honored. Copy one prompt, or export a scene as a Markdown shot list / CSV.
- 🗒️ **Takes Log & version history** — circle the take that worked; roll any prompt back.
- 🔌 **MCP built in** — agents and other tools can read and write your projects while Slate runs.
- 🎞 **A Stills Library** — scan your dailies for circled takes (Circle Take) or any clip, extract stills with ffmpeg, and pin them to a character, location, or look. ✦ Fill then describes the person or place you actually shot, so sheets stay true to the footage.
- 🔑 **No API keys, ever** — the brain is your own [Claude Code](https://claude.com/claude-code) or Codex sign-in, or any **local model** via Ollama, LM Studio, vLLM, llama.cpp… fully offline.

## Screenshots

| | |
|---|---|
| ![Home](docs/images/2-home.png) | ![Coverage](docs/images/3-coverage.png) |
| ![Sound Department](docs/images/4-sound.png) | ![First AD](docs/images/5-first-ad.png) |

## Install

**macOS — one line:**

```bash
curl -fsSL https://raw.githubusercontent.com/wassermanproductions/slate/main/install.sh | bash
```

Or grab `Slate-macOS.zip` from [Releases](../../releases), unzip, and drop `Slate.app` into Applications. (If macOS says the app "is damaged", that's Gatekeeper on unsigned browser downloads — the install script avoids it, or run `xattr -cr /Applications/Slate.app`.)

**From source (this fork)** — requires **Node 20+**, and **ffmpeg** on your PATH for clip/audio reference analysis (`brew install ffmpeg` / Windows: install ffmpeg and add it to PATH):

```bash
git clone https://github.com/thcabq352/slate.git
cd slate
npm ci
npm run dev          # Electron UI with hot reload
# optional production build:
# npm run build
# node scripts/package-macos.mjs --install   # macOS app package
```

Upstream install script (original releases):  
`curl -fsSL https://raw.githubusercontent.com/wassermanproductions/slate/main/install.sh | bash`

### The brain — your subscription or a local model, no API keys

Slate contains no API keys and makes no cloud calls of its own. Pick any of three brains:

- **Claude Code** (recommended) — install, then `claude auth login`
- **Codex** — if the ChatGPT desktop app is installed and signed in, Slate uses its bundled codex automatically; otherwise install the codex CLI and `codex login`
- **Local model (offline)** — any OpenAI-compatible local server: **Ollama, LM Studio, vLLM, llama.cpp, KoboldCpp, Jan**, and friends. Slate auto-detects the common ports (`11434`, `1234`, `8000`, `8080`), lists whatever models you have loaded, and never sends a byte off your machine. A custom endpoint field covers anything else. For the reference image/video breakdown features, load a vision-capable model (e.g. `llama3.2-vision`, `qwen2.5-vl`).

Pick the brain per project in the Project Bible. Click the **brain pill** in the titlebar any time to run a live connectivity test — if something's wrong it tells you the exact fix. If no brain is present, everything except the agent features still works.

## The workflow

1. **Create a project** and fill the **Project Bible** — logline, world & tone, defaults (aspect ratio, clip length, target model).
2. **Cast and scout** in the Studios: characters, props/wardrobe/vehicles, locations, a Lookbook style — each with ✦ auto-fill from one line of description.
3. **Build shots** by hand in the editor, or describe a scene and let **Coverage** lay it out, or tell the **First AD** what you want and watch the receipts roll in.
4. **Iterate** — one-click transforms (Structure / Tighten / Enrich / Distill / Shot / Angle), Variants, Punch-Ups, Alt Takes, the Tone dial, timecoded Beats, Pickups on any highlighted span.
5. **Keep it honest** — run a **Continuity Check** across the scene; a script-supervisor pass flags wardrobe, lighting, weather, and geography mismatches with concrete fixes.
6. **Deliver** — pick the target model, heed the preflight warnings, compile, copy, generate. **Circle the takes** that worked so the project remembers reality.

Projects are plain JSON in `~/Documents/Slate/` — yours to back up, sync, or version however you like.

## Working with agents (MCP)

Slate ships a [Model Context Protocol](https://modelcontextprotocol.io) server so agents and suite apps can read and write your projects while Slate runs:

```bash
claude mcp add slate -- node /absolute/path/to/slate/mcp/slate-mcp.mjs
```

Tools: `list_projects`, `get_project`, `create_project`, `list_shots`, `get_shot_prompt`, `set_shot_prompt` (with automatic versioning), `add_scene`, `list_characters`, `list_locations`, `list_lookbook`. The control channel is localhost-only with a per-session bearer token; the bridge is a single zero-dependency file.

It pairs naturally with the rest of [Wasserman's Filmmaker Suite](https://github.com/wassermanproductions/wassermans-filmmaker-suite) — break down the script in ScriptBreak, block the scene in Blockout, and craft every generator prompt in Slate.

## Rust engine / Hermes (film factory)

The **slate-engine** binary is a local film-factory control plane: project planning, shot continuity, and ComfyUI generation over MCP (primary front: Hermes). It is separate from the Electron prompt studio above.

**Build & run**

```bash
cargo build -p slate-engine --release
cargo run -p slate-engine -- mcp          # stdio MCP (Hermes / Claude Code)
cargo run -p slate-engine -- serve        # loopback HTTP control server
```

Binary name: `slate-engine` (release: `target/release/slate-engine`).

**ComfyUI** — default API `http://127.0.0.1:8188`. One GPU owner only; do not stack Video Buddy heavy jobs with Slate generations.

**Dry-run** (no GPU / no Comfy) — skip real generation:

```bash
SLATE_DRY_RUN=1 cargo run -p slate-engine -- mcp
```

**Hermes MCP registration**

```bash
hermes mcp add slate -- slate-engine mcp
# or: hermes mcp add slate -- /absolute/path/to/target/release/slate-engine mcp
```

Set the `slate_film_factory` tool timeout to **≥ 900s** (1800s on slow GPUs). Preflight with `slate_health` (Comfy ok + at least one brain).

Skill: [`skills/slate-film-factory/SKILL.md`](skills/slate-film-factory/SKILL.md). Design + plan: [spec](docs/superpowers/specs/2026-08-11-slate-rust-agent-film-factory-design.md) · [plan](docs/superpowers/plans/2026-08-12-slate-rust-film-factory.md).

## Development

| Command | What it does |
|---|---|
| `npm run dev` | Run from source with hot reload |
| `npm run build` | Production build (electron-vite) |
| `npm test` | Unit tests (export engine, action engine, audio DSP) |
| `npm run typecheck` | Strict TypeScript across main, preload, renderer |
| `node scripts/package-macos.mjs --install` | Build and install `/Applications/Slate.app` |
| `node scripts/snap.mjs` | Regenerate README screenshots headlessly |

Stack: Electron + TypeScript + React, CodeMirror 6 editor, zustand state, vitest. The audio reference analysis is local DSP (tempo, pitch, dynamics, structure) — measured on your machine, nothing uploaded.

## Support

**This fork** — issues and PRs: [github.com/thcabq352/slate](https://github.com/thcabq352/slate/issues). Maintainer: [thcabq352](https://github.com/thcabq352).

**Upstream author** — optional tips for Sam Wasserman’s open-source work (no pressure):

- [GitHub Sponsors](https://github.com/sponsors/wassermanproductions)
- [Ko-fi](https://ko-fi.com/samwasserman)

## License & credits

**Apache License 2.0** — see [LICENSE](LICENSE). Use, modify, fork, and redistribute, commercially or otherwise, under those terms.

**Attribution required (Apache-2.0 §4(d)):** retain [NOTICE](NOTICE) and credit the original author **Sam Wasserman** ([wassermanproductions.com](https://wassermanproductions.com)) in documentation and any about/credits surface. Full compliance notes: [docs/ATTRIBUTION.md](docs/ATTRIBUTION.md) · [MAINTAINERS.md](MAINTAINERS.md).

| Role | Who |
|------|-----|
| **Created by** | **Sam Wasserman** — [wassermanproductions.com](https://wassermanproductions.com) · [wasserman.ai](https://wasserman.ai) · upstream [wassermanproductions/slate](https://github.com/wassermanproductions/slate) |
| **Maintainer (this fork)** | **thcabq352** — [github.com/thcabq352](https://github.com/thcabq352) · [thcabq352/slate](https://github.com/thcabq352/slate) |

Fork-specific changes: [docs/CHANGELOG-FORK.md](docs/CHANGELOG-FORK.md).
