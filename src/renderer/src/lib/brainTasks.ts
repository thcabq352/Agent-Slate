// Brain task layer — builds the system + task prompts for every agent feature.
// Tier policy: mechanical text work runs fast/standard; creative direction runs top.

import type {
  Project,
  Scene,
  Shot,
  BrainRequest,
  BrainResult,
  BrainTier,
  CharacterSheet,
  ElementSheet,
  StyleProfile,
  BeatDirection
} from '../../../shared/types'
import { normalizeBrain } from '../../../shared/types'
import { uid } from '../stores/project'
import profilesData from '../../../../data/model-profiles.json'
import type { ModelProfile } from './compile'

const MODEL_PROFILES = (profilesData as { profiles: ModelProfile[] }).profiles

// ---------- Context assembly ----------

function targetProfile(p: Project, shot: Shot | null): ModelProfile | null {
  const id = shot?.targetModel ?? p.defaults.targetModel
  return MODEL_PROFILES.find((m) => m.id === id) ?? null
}

/** One line telling the brain which generator this prompt is destined for, so
 *  writing (not just the final compile) already leans toward its dialect. */
export function modelHint(p: Project, shot: Shot | null): string {
  const prof = targetProfile(p, shot)
  if (!prof) return ''
  const budget = prof.limits.maxChars ? ` Budget: ${prof.limits.maxChars} chars.` : ''
  return `\nTARGET GENERATOR: ${prof.label} (${prof.kind}).${budget} Keep phrasing compatible with its dialect: ${prof.dialect.guidance.slice(0, 260)}…`
}

function describeSpec(shot: Shot): string {
  const s = shot.spec
  const parts = [
    s.durationSec ? `${s.durationSec}s` : null,
    s.fps ? `${s.fps}fps` : null,
    s.aspectRatio,
    s.size,
    s.angle ? `${s.angle} angle` : null,
    s.lens,
    s.movement
  ].filter(Boolean)
  return parts.length ? parts.join(', ') : 'unspecified'
}

export function projectContext(p: Project, scene?: Scene | null): string {
  const lines: string[] = []
  lines.push(`PROJECT: ${p.name}`)
  if (p.logline) lines.push(`Logline: ${p.logline}`)
  if (p.world) lines.push(`World & tone: ${p.world}`)
  if (p.characters.length) {
    lines.push('\nCHARACTERS (keep these descriptions verbatim-consistent in every prompt):')
    for (const c of p.characters) {
      const bits = [c.age && `${c.age}`, c.gender, c.ethnicity, c.faceFeatures, c.hair, c.clothing]
        .filter(Boolean)
        .join('; ')
      lines.push(`- ${c.name}: ${bits}`)
    }
  }
  if (p.locations.length) {
    lines.push('\nLOCATIONS:')
    for (const l of p.locations)
      lines.push(`- ${l.name} (${l.interiorExterior}): ${l.description}${l.timeOfDay ? `; ${l.timeOfDay}` : ''}`)
  }
  if (p.artDept.length) {
    lines.push('\nPROPS / WARDROBE / VEHICLES:')
    for (const a of p.artDept) lines.push(`- ${a.name} (${a.kind}): ${a.description}${a.distinctive ? `; distinctive: ${a.distinctive}` : ''}`)
  }
  if (p.lookbook.length) {
    lines.push('\nACTIVE STYLE PROFILES (apply their language where relevant):')
    for (const s of p.lookbook)
      lines.push(`- ${s.source}: tone ${s.tone}; palette ${s.palette}; lighting ${s.lighting}; lenses ${s.lensLanguage}; movement ${s.movement}`)
  }
  if (p.music?.length) {
    lines.push('\nSCORE (music cues):')
    for (const m of p.music) lines.push(`- ${m.name}${m.sceneRef ? ` (${m.sceneRef})` : ''}: ${[m.genre, m.mood].filter(Boolean).join(', ')}`)
  }
  if (p.voices?.length) {
    lines.push('\nVOICE CAST:')
    for (const v of p.voices) lines.push(`- ${v.name}: ${[v.ageGender, v.accent, v.timbre].filter(Boolean).join(', ')}`)
  }
  if (scene) {
    lines.push(`\nCURRENT SCENE: ${scene.name}${scene.synopsis ? ` — ${scene.synopsis}` : ''}`)
    if (scene.shots.length) {
      lines.push('Shots so far in this scene (for continuity):')
      for (const sh of scene.shots) lines.push(`- ${sh.name} [${describeSpec(sh)}]: ${sh.intent || sh.prompt.slice(0, 110).replace(/\n/g, ' ') || 'empty'}`)
    }
  }
  return lines.join('\n')
}

const EDITOR_FORMAT = `PROMPT FORMAT — the app's editor uses sectioned prompts. Write prompts exactly in this shape (only include sections that have content, keep this order):

# Subject
<who/what the shot is about, action, wardrobe — concrete and visual>

# Composition
<framing, shot size, blocking, foreground/background layers, negative space>

# Lighting
<key quality/direction, practicals, color temperature, contrast, atmosphere>

# Camera
<lens focal length, camera movement, perspective height, depth of field, shutter feel>

# Style
<medium, film stock or digital character, grade, era, texture>

# Mood
<the emotional register, rendered as visual language>`

const CINEMA_SYSTEM = `You are the prompt-craft brain inside Agent-Slate, a professional prompt studio for filmmakers who generate images and video with AI models. You are an expert cinematographer, director, and prompt engineer. You write vivid, concrete, production-ready visual language — never vague adjectives, never filler like "high quality, 8k, masterpiece". Physical, photographable descriptions only. Respect every continuity fact you are given about characters, wardrobe, props, locations, lighting state, and time of day; if the user's request conflicts with continuity, follow the request and flag the conflict in one short line at the end prefixed with "CONTINUITY:".

${EDITOR_FORMAT}`

// ---------- Runner ----------

export interface RunOpts {
  task: string
  prompt: string
  system?: string
  tier?: BrainTier
  expectJson?: boolean
  images?: string[]
}

export async function runBrain(p: Project, opts: RunOpts): Promise<BrainResult> {
  const req: BrainRequest & { backend: Project['defaults']['brain'] } = {
    id: uid('req'),
    task: opts.task,
    system: opts.system ?? CINEMA_SYSTEM,
    prompt: opts.prompt,
    tier: opts.tier ?? 'top',
    expectJson: opts.expectJson,
    images: opts.images,
    backend: normalizeBrain(p.defaults.brain),
    localEndpoint: p.defaults.localEndpoint,
    localModel: p.defaults.localModel
  }
  return (await window.slate.brainRun(req)) as BrainResult
}

// ---------- Editor transforms ----------

export type TransformKind = 'structure' | 'tighten' | 'enrich' | 'distill' | 'shot' | 'angle'

const TRANSFORM_INSTRUCTIONS: Record<TransformKind, { label: string; tier: BrainTier; text: string }> = {
  structure: {
    label: 'Structure',
    tier: 'standard',
    text: 'Reorganize this prompt into the sectioned format. Preserve every concrete detail; move each to its correct section; do not invent new content beyond minimal connective phrasing.'
  },
  tighten: {
    label: 'Tighten',
    tier: 'standard',
    text: 'Shorten this prompt by roughly a third. Remove redundancy and weak modifiers first. Keep every concrete visual fact. Keep the section structure.'
  },
  enrich: {
    label: 'Enrich',
    tier: 'top',
    text: 'Deepen this prompt with specific, professional cinematographic detail — lens behavior, light quality, texture, atmosphere — without changing its intent or subject. Stay disciplined: enrich, do not pad.'
  },
  distill: {
    label: 'Distill',
    tier: 'standard',
    text: 'Strip this prompt to its essential core — the single strongest version of the idea in the fewest words that still carry all critical visual information. Keep the section structure with only the sections that matter.'
  },
  shot: {
    label: 'Shot',
    tier: 'top',
    text: 'Rewrite the Composition and Camera sections to make this a stronger, more intentional shot: clarify shot size, framing logic and blocking. Leave other sections untouched unless they contradict the new framing.'
  },
  angle: {
    label: 'Angle',
    tier: 'top',
    text: 'Propose a more dramatic camera angle and perspective for this shot and rewrite the Camera section (and Composition where needed) accordingly. Keep subject and continuity identical.'
  }
}

function lockNotice(shot: Shot): string {
  if (!shot.lockedLines.length) return ''
  const lines = shot.prompt.split('\n')
  const locked = shot.lockedLines
    .filter((n) => n >= 1 && n <= lines.length)
    .map((n) => `${n}: ${lines[n - 1]}`)
  if (!locked.length) return ''
  return `\n\nPICTURE-LOCKED LINES — reproduce these lines EXACTLY, character for character, in your output:\n${locked.join('\n')}`
}

export async function transformPrompt(
  p: Project,
  scene: Scene | null,
  shot: Shot,
  kind: TransformKind
): Promise<BrainResult> {
  const t = TRANSFORM_INSTRUCTIONS[kind]
  return runBrain(p, {
    task: `transform:${kind}`,
    tier: t.tier,
    prompt: `${projectContext(p, scene)}\n\nSHOT SPEC: ${describeSpec(shot)}${modelHint(p, shot)}\n\nTASK: ${t.text}${lockNotice(shot)}\n\nCURRENT PROMPT:\n${shot.prompt}\n\nReturn ONLY the rewritten prompt in the sectioned format — no commentary.`
  })
}

// ---------- Pickups (selective span rewrite) ----------

export async function pickupSpan(
  p: Project,
  scene: Scene | null,
  shot: Shot,
  span: string,
  instruction: string
): Promise<BrainResult> {
  return runBrain(p, {
    task: 'pickup',
    tier: 'standard',
    prompt: `${projectContext(p, scene)}\n\nThe user highlighted this exact span inside their prompt:\n<<<${span}>>>\n\nInstruction for the span: ${instruction}\n\nRewrite ONLY that span. Return ONLY the replacement text for the span — no quotes, no commentary, no surrounding prompt. The replacement must read naturally in place and respect all continuity.`
  })
}

// ---------- Director's Notes (chat) ----------

export async function directorsNote(
  p: Project,
  scene: Scene | null,
  shot: Shot | null,
  history: Array<{ role: 'user' | 'assistant'; text: string }>,
  note: string
): Promise<BrainResult> {
  const convo = history
    .slice(-10)
    .map((m) => `${m.role === 'user' ? 'DIRECTOR' : 'SLATE'}: ${m.text}`)
    .join('\n\n')
  return runBrain(p, {
    task: 'directors-note',
    tier: 'top',
    prompt: `${projectContext(p, scene)}\n\n${shot ? `CURRENT SHOT "${shot.name}" [${describeSpec(shot)}]:\n${shot.prompt || '(empty)'}${lockNotice(shot)}\n\n` : ''}${convo ? `CONVERSATION SO FAR:\n${convo}\n\n` : ''}DIRECTOR'S NOTE: ${note}\n\nRespond as a sharp, collaborative creative partner. If the note asks for prompt changes, include the FULL updated prompt in the sectioned format inside a fenced code block (\`\`\`prompt ... \`\`\`). If it's a question or creative discussion, answer concisely without a code block. Never lose locked lines.`
  })
}

/** Extract a \`\`\`prompt fenced block from a Director's Notes reply, if present. */
export function extractPromptBlock(text: string): string | null {
  const m = text.match(/```(?:prompt)?\n([\s\S]*?)```/)
  return m ? m[1].trim() : null
}

// ---------- Alt Takes (roll with rules) ----------

export async function altTake(
  p: Project,
  scene: Scene | null,
  shot: Shot,
  roll: string[],
  keep: string[],
  rules: string
): Promise<BrainResult> {
  return runBrain(p, {
    task: 'alt-take',
    tier: 'top',
    prompt: `${projectContext(p, scene)}\n\nCURRENT PROMPT:\n${shot.prompt}${lockNotice(shot)}\n\nROLL AN ALT TAKE. Re-imagine ONLY these elements with a fresh, bold choice: ${roll.join(', ') || 'any one element of your choosing'}.\nKeep these elements exactly as they are: ${keep.join(', ') || 'everything else'}.\n${rules ? `Conditional rules to honor: ${rules}` : ''}\n\nReturn ONLY the full rewritten prompt in the sectioned format.`
  })
}

// ---------- Variants ----------

export async function promptVariants(
  p: Project,
  scene: Scene | null,
  shot: Shot,
  count: number
): Promise<BrainResult> {
  return runBrain(p, {
    task: 'variants',
    tier: 'top',
    expectJson: true,
    prompt: `${projectContext(p, scene)}\n\nCURRENT PROMPT:\n${shot.prompt}${lockNotice(shot)}\n\nWrite ${count} distinct variants of this prompt, each weighting the shot differently (e.g. one composition-forward, one lighting-forward, one motion-forward, one mood-forward). Same subject, same continuity, same spec — different emphasis and language. Return JSON: {"variants":[{"label":"<2-4 word handle>","prompt":"<full sectioned prompt>"}]}`
  })
}

// ---------- Coverage ----------

export interface CoverageShotPlan {
  slot: string
  size: string
  angle: string
  movement: string
  intent: string
  promptGuidance: string
}

export async function generateCoverage(
  p: Project,
  scene: Scene,
  sceneDescription: string,
  plan: { label: string; shots: CoverageShotPlan[] } | null,
  customAsk: string | null
): Promise<BrainResult> {
  const planText = plan
    ? `COVERAGE PLAN "${plan.label}" — create exactly these shots:\n${plan.shots
        .map(
          (s, i) =>
            `${i + 1}. ${s.slot}: ${s.size}, ${s.angle}, ${s.movement}. Intent: ${s.intent} Guidance: ${s.promptGuidance}`
        )
        .join('\n')}`
    : `The director asked for this coverage: ${customAsk}\nDesign the right set of shots (4-8) a seasoned director would want, varied in size, angle and movement.`
  return runBrain(p, {
    task: 'coverage',
    tier: 'top',
    expectJson: true,
    prompt: `${projectContext(p, scene)}\n\nSCENE TO COVER:\n${sceneDescription}\n\n${planText}\n\nFor EVERY shot write a complete sectioned prompt, fully self-contained (a generator sees one prompt at a time — repeat character/location descriptions verbatim in each). Return JSON: {"shots":[{"name":"<slot name>","intent":"<one line>","size":"<size>","angle":"<angle>","movement":"<movement>","prompt":"<full sectioned prompt>"}]}`
  })
}

// ---------- Sequence chunking ----------

export async function chunkSequence(
  p: Project,
  scene: Scene,
  sequenceDescription: string,
  totalSeconds: number,
  chunkSeconds: number,
  beatDirected: boolean
): Promise<BrainResult> {
  return runBrain(p, {
    task: 'chunk-sequence',
    tier: 'top',
    expectJson: true,
    prompt: `${projectContext(p, scene)}\n\nSEQUENCE (${totalSeconds}s total) TO BREAK INTO ~${chunkSeconds}s GENERATION CHUNKS:\n${sequenceDescription}\n\nRules:
- Each chunk is one video-generation prompt of about ${chunkSeconds}s (never exceed it by more than 2s).
- CONTINUITY HANDOFF: each chunk's opening state must exactly match the previous chunk's closing state — describe the carry-over explicitly (positions, damage, weather, light).
- Repeat character/wardrobe/location descriptions verbatim in every chunk; a generator sees one chunk at a time.
- Escalate rhythm and stakes across chunks like a real action editor would.
${beatDirected ? `- Inside each chunk, direct beats with timecodes ("0-3s: ...", "4-${chunkSeconds}s: ...").` : ''}
Return JSON: {"chunks":[{"name":"Chunk 1 (0:00-0:${String(chunkSeconds).padStart(2, '0')})","startSec":0,"endSec":${chunkSeconds},"prompt":"<full sectioned prompt${beatDirected ? ' with timecoded beats inside the Subject section' : ''}>","handoff":"<one line: the exact end-state the next chunk must open on>"}]}`
  })
}

// ---------- Beat sheet for a single shot ----------

export async function beatSheetForShot(p: Project, scene: Scene | null, shot: Shot): Promise<BrainResult> {
  const dur = shot.spec.durationSec ?? p.defaults.durationSec
  return runBrain(p, {
    task: 'beat-sheet',
    tier: 'top',
    expectJson: true,
    prompt: `${projectContext(p, scene)}\n\nSHOT (${dur}s) PROMPT:\n${shot.prompt}\n\nBreak this shot into 2-5 timecoded beats that a video model can follow — motivated micro-actions and camera evolution, not arbitrary slices. Return JSON: {"beats":[{"from":0,"to":3,"text":"<what happens and how the camera behaves>"}]} covering 0 to ${dur} seconds with no gaps.`
  })
}

// ---------- Studios ----------

export async function fillCharacter(
  p: Project,
  description: string,
  scenario: string,
  stills?: string[]
): Promise<BrainResult> {
  const fromStills = stills?.length
    ? '\n\nContinuity stills of this exact character are attached — describe the person you SEE (face, hair, wardrobe as shot), not an invention. The sheet must match the footage.'
    : ''
  return runBrain(p, {
    task: 'casting-fill',
    tier: 'top',
    expectJson: true,
    images: stills?.slice(0, 6),
    prompt: `${projectContext(p)}\n\nThe director described a character: "${description}" (sheet context: ${scenario}).${fromStills}\n\nProduce a complete, castable person — specific and photographable, no clichés. Return JSON with exactly these string fields: {"name","age","gender","ethnicity","faceFeatures","hair","clothing","expression","eyeDirection","mood","environment","keyLightSide","lightingMood"}. faceFeatures: skin, face shape, distinguishing marks, facial hair. Keep each field a compact comma-separated phrase list like a casting sheet, not sentences.`
  })
}

export function characterSheetPrompt(c: CharacterSheet, scenario: string): string {
  return [
    '# Subject',
    `${c.name}, ${c.age ? `${c.age} years old, ` : ''}${[c.gender, c.ethnicity].filter(Boolean).join(', ')}. ${c.faceFeatures}. Hair: ${c.hair}. Wearing ${c.clothing}. ${c.expression}. ${c.eyeDirection}.`,
    '',
    '# Composition',
    scenario === 'portrait' || scenario === 'cinematic'
      ? 'Character reference sheet framing: centered head-and-shoulders portrait, direct framing, plain seamless backdrop, generous headroom, no props.'
      : `${scenario} framing of the same person, identity-consistent, face clearly visible.`,
    '',
    '# Lighting',
    `${c.keyLightSide || 'Key light from left'}, ${c.lightingMood || 'natural soft light'}, clean catchlights, gentle fill, no colored gels.`,
    '',
    '# Style',
    'Photoreal, neutral grade, true-to-skin color rendering, medium format character, crisp detail for identity reference.',
    '',
    '# Mood',
    c.mood || 'calm, neutral, professional'
  ].join('\n')
}

export async function studyStyle(p: Project, source: string, kind: string): Promise<BrainResult> {
  return runBrain(p, {
    task: 'lookbook-study',
    tier: 'top',
    expectJson: true,
    prompt: `Study the visual style of the ${kind} "${source}" from your knowledge of cinema. Distill it into reusable prompt language — concrete visual vocabulary, not names (the profile will be inserted into generation prompts, so describe the LOOK, never cite the person/film). Return JSON with string fields: {"tone","palette","lighting","lensLanguage","movement","blocking","editorial","notes"}. Each field: 1-2 sentences of specific, physical description. In "notes" put 2-3 signature tricks worth stealing.`
  })
}

// ---------- Reference element breakdown ----------

export async function analyzeReference(
  p: Project,
  frames: string[],
  kind: 'image' | 'video'
): Promise<BrainResult> {
  return runBrain(p, {
    task: 'reference-analysis',
    tier: 'top',
    expectJson: true,
    images: frames.slice(0, 12),
    prompt: `Analyze the attached ${kind === 'video' ? 'frames extracted from a video clip (in order)' : 'reference image(s)'} like a cinematographer breaking down a reference. Describe only what is visually present. Return JSON with string fields: {"lensing":"<focal feel, DOF, distortion, coverage style>","lighting":"<key direction/quality, practicals, contrast ratio, color temp>","palette":"<dominant colors, grade, saturation character>","composition":"<framing habits, blocking, layers, headroom>","movement":"<camera + subject motion${kind === 'image' ? ', or implied motion' : ', cut rhythm if multiple shots'}>","texture":"<grain, sharpness, atmosphere, era>","mood":"<emotional register as visual language>","notes":"<2-3 things worth stealing for prompts>"}`
  })
}

export function elementSheetToSetups(e: ElementSheet): Array<{ section: string; label: string; snippet: string }> {
  return [
    { section: 'camera', label: 'Ref: lensing', snippet: e.lensing },
    { section: 'lighting', label: 'Ref: lighting', snippet: e.lighting },
    { section: 'style', label: 'Ref: palette & texture', snippet: `${e.palette}. ${e.texture}` },
    { section: 'composition', label: 'Ref: composition', snippet: e.composition },
    { section: 'camera', label: 'Ref: movement', snippet: e.movement },
    { section: 'mood', label: 'Ref: mood', snippet: e.mood }
  ]
}

// ---------- Punch-Ups & Tone Dial & Second Unit ----------

export async function punchUps(p: Project, scene: Scene | null, shot: Shot): Promise<BrainResult> {
  return runBrain(p, {
    task: 'punch-ups',
    tier: 'top',
    expectJson: true,
    prompt: `${projectContext(p, scene)}\n\nCURRENT SHOT PROMPT:\n${shot.prompt}\n\nPitch 4 bold "what if" riffs on this shot — surprising but shootable creative swings (a different vantage, a reveal, an environmental event, a lens trick). Return JSON: {"riffs":[{"pitch":"<one-line pitch>","prompt":"<full sectioned prompt>"}]}`
  })
}

export async function toneDial(
  p: Project,
  scene: Scene | null,
  shot: Shot,
  emotion: string,
  intensity: number
): Promise<BrainResult> {
  return runBrain(p, {
    task: 'tone-dial',
    tier: 'top',
    prompt: `${projectContext(p, scene)}\n\nCURRENT PROMPT:\n${shot.prompt}${lockNotice(shot)}\n\nDial the emotional tone to "${emotion}" at intensity ${intensity}/10. Re-voice Lighting, Style and Mood (and Composition if needed) so the frame FEELS it — through physical visual choices, not adjective piles. Subject and continuity stay identical. Return ONLY the full rewritten sectioned prompt.`
  })
}

export async function secondUnit(p: Project, scene: Scene, afterShot: Shot | null): Promise<BrainResult> {
  return runBrain(p, {
    task: 'second-unit',
    tier: 'top',
    expectJson: true,
    prompt: `${projectContext(p, scene)}\n\n${afterShot ? `The scene currently ends on:\n${afterShot.prompt}\n\n` : ''}Extend this scene with the 3 shots a second-unit director would grab next — continuations that deepen geography, texture, or tension while matching all continuity. Return JSON: {"shots":[{"name":"<name>","intent":"<one line>","prompt":"<full sectioned prompt>"}]}`
  })
}

// ---------- Continuity check ----------

export async function continuityCheck(p: Project, scene: Scene): Promise<BrainResult> {
  const shots = scene.shots.map((s) => `--- ${s.name} ---\n${s.prompt}`).join('\n\n')
  return runBrain(p, {
    task: 'continuity-check',
    tier: 'top',
    expectJson: true,
    prompt: `${projectContext(p)}\n\nSCENE "${scene.name}" — ALL SHOT PROMPTS IN ORDER:\n\n${shots}\n\nAudit continuity across these prompts like a script supervisor: character descriptions, wardrobe, props, lighting state, time of day, weather, geography/eyelines. Return JSON: {"issues":[{"severity":"error"|"warn","shots":["<shot names>"],"issue":"<what mismatches>","fix":"<concrete fix>"}]} — empty array if clean.`
  })
}

export type { BeatDirection }
