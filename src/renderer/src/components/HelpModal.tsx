// Help — the full guided tour of Slate, organized like a crew handbook.

import React, { useState } from 'react'

interface Section {
  id: string
  label: string
  title: string
  body: React.ReactNode
}

const K = ({ children }: { children: React.ReactNode }): React.JSX.Element => (
  <span className="help-key">{children}</span>
)

const SECTIONS: Section[] = [
  {
    id: 'start',
    label: 'Getting Started',
    title: 'Your first five minutes',
    body: (
      <>
        <p>
          Slate doesn&apos;t generate images or video — it makes the <b>prompts</b> you paste into
          your generators dramatically better, faster, and consistent across a whole film. The flow
          is always: <b>craft in Slate → copy → paste into your generator</b>.
        </p>
        <ol>
          <li>
            <b>Create a project</b> on the home screen — one project per film or campaign.
          </li>
          <li>
            <b>Open the Project Bible</b> (bottom of the left rail) and fill in your logline, world
            &amp; tone, and defaults (aspect ratio, clip length, target model, brain). Everything
            the brain writes uses this context.
          </li>
          <li>
            <b>Add a scene</b> (+ Scene, top left), then <b>add a shot</b> — or skip straight to
            the Coverage tab and let the brain build the whole scene.
          </li>
          <li>
            <b>Test the brain</b> — click the pill in the titlebar (&quot;Brain: Claude Code —
            test&quot;). Green means you&apos;re live.
          </li>
        </ol>
        <p className="help-tip">
          💡 The brain runs on your own Claude Code or Codex sign-in — or fully offline on a local
          model (Ollama, LM Studio, vLLM, llama.cpp…). No API keys, ever. Switch brains per project
          in the Project Bible; pick &quot;Local model&quot; and Slate auto-detects your server and
          lists its models. For reference breakdowns, load a vision-capable local model.
        </p>
        <p className="help-tip">
          🎞 <b>Stills Library</b> (Refs tab): scan your dailies for circled takes — or add any
          clip — extract stills, and pin them to a character, location, or look. ✦ Fill on a sheet
          with stills describes the person or place you actually shot, and the sheet keeps the
          images for continuity.
        </p>
      </>
    )
  },
  {
    id: 'editor',
    label: 'The Prompt Editor',
    title: 'Writing shots like a filmmaker',
    body: (
      <>
        <p>
          Prompts are structured into sections — <b># Subject, # Composition, # Lighting, # Camera,
          # Style, # Mood</b> — the same categories a crew thinks in. As you type, cinematic terms
          light up by category: camera language in cyan, lighting in gold, color in magenta, motion
          in green, mood in violet.
        </p>
        <p>
          <b>The gutter</b> (left of each line) holds two toggles:
        </p>
        <ul>
          <li>
            <K>●</K> <b>Picture Lock</b> — lock a line and no transform, roll, or rewrite will ever
            touch it. Locked lines even survive character-budget compression verbatim.
          </li>
          <li>
            <K>−</K> <b>Mute</b> — keep a line in the editor but exclude it from exports. Great for
            A/B-ing an idea without deleting it.
          </li>
        </ul>
        <p>
          <b>Pickups</b> — highlight any phrase and a bar appears at the bottom. Type what should
          change about <i>just that span</i> (&quot;make it dusk&quot;) and hit Reshoot Span.
          Everything else stays untouched.
        </p>
        <p>
          <b>Spec bar</b> — length in seconds (any number), fps, aspect ratio, shot size, angle,
          lens, movement, and an optional max character budget. These are real fields, not prose —
          the compiler uses them when you export.
        </p>
      </>
    )
  },
  {
    id: 'toolbar',
    label: 'Transforms & Tools',
    title: 'One-click craft moves',
    body: (
      <>
        <ul>
          <li>
            <b>Structure</b> — organizes a messy prompt into clean sections without losing detail.
          </li>
          <li>
            <b>Tighten</b> — cuts about a third, dropping weak modifiers first, keeping every fact.
          </li>
          <li>
            <b>Enrich</b> — deepens the craft: lens behavior, light quality, texture, atmosphere.
          </li>
          <li>
            <b>Distill</b> — strips to the essential core.
          </li>
          <li>
            <b>Shot / Angle</b> — reworks framing &amp; blocking, or proposes a bolder camera
            perspective.
          </li>
          <li>
            <b>Variants</b> — four differently-weighted versions (composition-forward,
            lighting-forward…) to A/B in your generator.
          </li>
          <li>
            <b>Punch-Ups</b> — four bold &quot;what if&quot; riffs on the shot.
          </li>
          <li>
            <b>Alt Take</b> — roll the dice on chosen elements (angle, lighting, weather…) with
            optional rules like &quot;if the angle goes low, keep the lens wide.&quot;
          </li>
          <li>
            <b>Tone</b> — dial an emotion (dread, wonder, chaos…) to an intensity; lighting, style
            and mood re-voice while the subject stays identical.
          </li>
          <li>
            <b>Beats</b> — break the shot into timecoded beats (0–3s… 4–8s…) that video models
            follow.
          </li>
        </ul>
        <p className="help-tip">
          💡 Every change saves the previous prompt to <b>Version History</b> (Deliver tab) — roll
          back anytime.
        </p>
      </>
    )
  },
  {
    id: 'notes',
    label: "Director's Notes & First AD",
    title: 'Two ways to talk to the brain',
    body: (
      <>
        <p>
          <b>Director&apos;s Notes</b> (dock under the editor) is a conversation about <i>the
          current shot</i>. Give a note — &quot;make it rain, keep the neon&quot; — and the prompt
          updates; ask a question and it answers without touching anything.
        </p>
        <p>
          <b>✦ First AD</b> (titlebar button) is <i>optional</i> and project-wide: describe what
          you&apos;re after and hone it in a back-and-forth. When intent is clear, it runs the set —
          creating scenes, shots, specs, prompts, cast, locations, even music cues and voices — with
          a receipt for every action. It focuses the UI on what it built; everything it writes goes
          through version history like your own edits.
        </p>
        <p className="help-tip">
          💡 Use First AD to block out fast (&quot;90-second chase, Seedance, 10s chunks&quot;),
          then refine shots by hand. They&apos;re designed to hand off to each other.
        </p>
      </>
    )
  },
  {
    id: 'coverage',
    label: 'Coverage & Chunks',
    title: 'From one description to a full scene',
    body: (
      <>
        <p>
          Open the <b>Coverage</b> tab, describe the scene (or rely on the scene synopsis), then:
        </p>
        <ul>
          <li>
            <b>Coverage plans</b> — Full, Dialogue, Motion, Extreme Action, Establishing,
            Surveillance, Entrance, Parallel Action, Dance, Angle, Orbit, Story Beats. One click ⚡
            generates a complete set of shots, each with a full self-contained prompt.
          </li>
          <li>
            <b>Call Your Own</b> — describe exactly the coverage you want in plain English
            (&quot;5 shots, mostly long lens, no drone&quot;).
          </li>
          <li>
            <b>Sequence Chunks</b> — for long sequences: set total length and chunk size (say 180s
            into 20s pieces) and Slate writes one generation prompt per chunk with explicit
            <b> continuity handoffs</b> — each chunk opens exactly where the last ended. Optionally
            beat-directed inside each chunk.
          </li>
          <li>
            <b>Second Unit</b> — three shots a second-unit director would grab to extend the scene.
          </li>
          <li>
            <b>Continuity Check</b> — a script-supervisor audit across all shots in the scene:
            wardrobe, props, light, weather, geography. Errors and warnings with concrete fixes.
          </li>
        </ul>
      </>
    )
  },
  {
    id: 'studios',
    label: 'Studios',
    title: 'Casting, Art Dept, Locations, Lookbook, Sound',
    body: (
      <>
        <p>
          The <b>Studios</b> tab holds your film&apos;s bible departments. Anything saved here is
          woven into every prompt the brain writes — that&apos;s how consistency happens.
        </p>
        <ul>
          <li>
            <b>Casting</b> — structured character sheets. Type a description (&quot;weathered
            getaway driver, 60s&quot;) and hit ✦ Fill to auto-complete every field. Copy a{' '}
            <b>Sheet Prompt</b> to generate an identity-reference sheet in your image generator.
            Scenario tabs (Cinematic, Portrait, Fashion…) reframe the same person.
          </li>
          <li>
            <b>Art Dept</b> — the hero car, the lucky lighter, the bloodstained jacket. Props,
            wardrobe, vehicles — kept identical in every shot.
          </li>
          <li>
            <b>Locations</b> — scout a place once; every prompt shoots the same place.
          </li>
          <li>
            <b>Lookbook</b> — study a cinematographer, director, film, or series into a reusable
            style profile (tone, palette, lighting, lens language, movement). Profiles shape every
            prompt; names never appear in output.
          </li>
          <li>
            <b>Sound</b> — see the Sound Department section.
          </li>
        </ul>
      </>
    )
  },
  {
    id: 'sound',
    label: 'Sound Department',
    title: 'Score & voices',
    body: (
      <>
        <p>
          <b>Score</b> — design music cues like a composer spotting a scene: intent, genre, mood,
          tempo, instrumentation, structure. ✦ Fill designs a cue from one line; the brain can also
          write tagged <b>lyrics</b>. Pick a target — Suno, Eleven Music, Lyria (Google Flow), Udio,
          Stable Audio — and <b>Prompt →</b> compiles the cue in that tool&apos;s exact dialect,
          with warnings if the cue exceeds the tool&apos;s limits.
        </p>
        <p>
          <b>Voices</b> — voice sheets for your characters (timbre, accent, pacing, texture,
          emotional range), linkable to your cast. Compiling produces a voice-design prompt{' '}
          <i>plus audition text</i> for ElevenLabs, Hume, or MiniMax — paste both and judge the
          voice on lines that exercise its range.
        </p>
        <p>
          <b>♫ Match a reference</b> — drop any audio file (or video with music/voice) onto the
          drop zone. Slate measures the signal locally — tempo, pitch register, dynamics,
          brightness, energy arc — and the brain reverse-engineers a matching cue or voice sheet.
          Add a hint (&quot;vault-heist score&quot;) for sharper results. Nothing is uploaded.
        </p>
      </>
    )
  },
  {
    id: 'refs',
    label: 'References',
    title: 'Steal like a cinematographer',
    body: (
      <>
        <p>
          The <b>Refs</b> tab takes stills and video clips. Clips are broken into key frames
          locally (ffmpeg). Hit <b>Break Down</b> and the brain writes an <b>element sheet</b> —
          lensing, lighting, palette, composition, movement, texture, mood.
        </p>
        <p>
          <b>Save Elements as Setups</b> turns each element into a one-click insert — so &quot;the
          lighting from that frame in that film you love&quot; becomes a reusable ingredient you
          can drop into any prompt.
        </p>
        <p className="help-tip">
          💡 Media is linked, never copied — your footage stays where it lives.
        </p>
      </>
    )
  },
  {
    id: 'setups',
    label: 'Setups',
    title: '165+ professional ingredients',
    body: (
      <>
        <p>
          The <b>Setups</b> tab is a library of prompt fragments in professional cinematography
          language: film stocks, lenses, lighting rigs, composition patterns, moods, single-shot
          moves. Click <K>↩</K> and it inserts into the correct section of the current prompt.
        </p>
        <p>
          <b>My Setups</b> — select any text in your prompt and hit &quot;Save selection as My
          Setup&quot; to build your personal library of best ingredients. Reference-derived
          elements land here too.
        </p>
      </>
    )
  },
  {
    id: 'deliver',
    label: 'Deliver',
    title: 'Compile for the exact generator',
    body: (
      <>
        <p>
          The <b>Deliver</b> tab is where a shot becomes a paste-ready prompt for a specific model
          — Midjourney, GPT Image, Krea, Flux, Seedance, Hailuo, LTX, Kling, Sora, Veo, or ComfyUI.
        </p>
        <ul>
          <li>
            <b>Preflight warnings</b> — before you paste, Slate checks your shot against the
            model&apos;s real limits (max duration, allowed aspect ratios, fps) and warns you.
          </li>
          <li>
            <b>Compile</b> — rewrites the prompt in the target&apos;s dialect: terse tags for tag
            models, flowing prose for cinematic models, timecoded beats kept for models that honor
            them, woven into prose for ones that don&apos;t. Character budgets are enforced by
            smart compression — locked lines survive verbatim. Negative prompts are written where
            the model supports them.
          </li>
          <li>
            <b>Takes Log</b> — after generating, log the take: ⭕ Circle the keeper. The project
            remembers what actually worked.
          </li>
          <li>
            <b>Export Scene</b> — the whole scene as a Markdown shot list or CSV.
          </li>
        </ul>
      </>
    )
  },
  {
    id: 'brains',
    label: 'Brains & Troubleshooting',
    title: 'If something feels off',
    body: (
      <>
        <ul>
          <li>
            <b>Test the brain</b> — the titlebar pill fires a real, tiny call and reports exactly
            what&apos;s wrong if it fails.
          </li>
          <li>
            <b>&quot;Sign-in expired/revoked&quot;</b> — open Terminal, run{' '}
            <K>claude auth login</K>, approve in the browser.
          </li>
          <li>
            <b>Codex</b> — Slate uses the codex bundled with the ChatGPT desktop app, which shares
            your ChatGPT sign-in automatically. If Codex fails, open the ChatGPT app once and make
            sure you&apos;re signed in.
          </li>
          <li>
            <b>Slow responses</b> — creative work (coverage, First AD) uses the strongest model and
            can take a minute; mechanical transforms use faster tiers.
          </li>
          <li>
            <b>Other apps / agents</b> — Slate runs an MCP server while open, so Claude Code and
            suite apps can read and write your projects. Connect with:{' '}
            <K>claude mcp add slate -- node …/slate/mcp/slate-mcp.mjs</K>
          </li>
          <li>
            <b>Your data</b> — projects are plain JSON in <K>~/Documents/Slate/</K>. Back up, sync,
            or version them however you like.
          </li>
        </ul>
      </>
    )
  }
]

export default function HelpModal({ onClose }: { onClose(): void }): React.JSX.Element {
  const [active, setActive] = useState('start')
  const section = SECTIONS.find((s) => s.id === active) ?? SECTIONS[0]
  return (
    <div className="modal-scrim" onClick={onClose}>
      <div className="modal help-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-head">
          <h2>Slate Help</h2>
          <button className="btn btn-ghost btn-sm" onClick={onClose}>
            ✕
          </button>
        </div>
        <div className="help-body">
          <nav className="help-nav">
            {SECTIONS.map((s) => (
              <button
                key={s.id}
                className={`help-nav-item ${active === s.id ? 'active' : ''}`}
                onClick={() => setActive(s.id)}
              >
                {s.label}
              </button>
            ))}
          </nav>
          <article className="help-content" key={section.id}>
            <h3>{section.title}</h3>
            {section.body}
          </article>
        </div>
        <div className="help-foot">
          <span>
            Created by <b>Sam Wasserman</b> ·{' '}
            <a href="https://wassermanproductions.com" target="_blank" rel="noreferrer">
              wassermanproductions.com
            </a>{' '}
            ·{' '}
            <a href="https://wasserman.ai" target="_blank" rel="noreferrer">
              wasserman.ai
            </a>
          </span>
          <a className="btn btn-sm btn-key" href="https://ko-fi.com/samwasserman" target="_blank" rel="noreferrer">
            ♥ Support on Ko-fi
          </a>
        </div>
      </div>
    </div>
  )
}
