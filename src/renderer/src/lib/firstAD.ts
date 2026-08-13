// The First AD — Slate's conversational operator. You talk through what you
// want; it answers, and when the intent is clear it runs the set: creates
// scenes and shots, sets specs, writes prompts, fills the bible.

import type { Project, Scene, Shot, BrainResult, ShotSpec, BeatDirection } from '../../../shared/types'
import { runBrain, projectContext } from './brainTasks'
import { blankShot, uid } from '../stores/project'

// ---------- Action contract ----------

interface SpecPatch {
  durationSec?: number | null
  fps?: number | null
  aspectRatio?: string | null
  size?: string | null
  angle?: string | null
  lens?: string | null
  movement?: string | null
}

export type AdAction =
  | { type: 'update_project'; logline?: string; world?: string; defaults?: Partial<Project['defaults']> }
  | { type: 'create_scene'; name: string; synopsis?: string }
  | { type: 'update_scene'; scene: string; name?: string; synopsis?: string }
  | {
      type: 'create_shot'
      scene: string
      name?: string
      intent?: string
      prompt?: string
      spec?: SpecPatch
      targetModel?: string
      maxChars?: number | null
      beatSheet?: BeatDirection[]
    }
  | {
      type: 'update_shot'
      shot: string
      name?: string
      intent?: string
      prompt?: string
      spec?: SpecPatch
      targetModel?: string
      maxChars?: number | null
      beatSheet?: BeatDirection[] | null
    }
  | { type: 'add_character'; [k: string]: unknown }
  | { type: 'add_location'; [k: string]: unknown }
  | { type: 'add_art'; [k: string]: unknown }
  | { type: 'add_music_cue'; [k: string]: unknown }
  | { type: 'add_voice'; [k: string]: unknown }
  | { type: 'select'; scene?: string; shot?: string }

export interface AdReply {
  reply: string
  actions: AdAction[]
}

const ACTION_DOC = `ACTIONS — when (and only when) the director's intent is clear enough to act, include actions. Available actions:

- {"type":"update_project","logline"?,"world"?,"defaults"?:{"aspectRatio"?,"fps"?,"durationSec"?,"targetModel"?}}
- {"type":"create_scene","name","synopsis"?}
- {"type":"update_scene","scene":"<name or id>","name"?,"synopsis"?}
- {"type":"create_shot","scene":"<name or id>","name"?,"intent"?,"prompt"?,"spec"?:{"durationSec"?,"fps"?,"aspectRatio"?,"size"?,"angle"?,"lens"?,"movement"?},"targetModel"?,"maxChars"?,"beatSheet"?:[{"from","to","text"}]}
- {"type":"update_shot","shot":"<name or id>","name"?,"intent"?,"prompt"?,"spec"?,"targetModel"?,"maxChars"?,"beatSheet"?}
- {"type":"add_character","name","age"?,"gender"?,"ethnicity"?,"faceFeatures"?,"hair"?,"clothing"?,"expression"?,"eyeDirection"?,"mood"?,"environment"?,"keyLightSide"?,"lightingMood"?}
- {"type":"add_location","name","interiorExterior"?("interior"|"exterior"|"both"),"description"?,"timeOfDay"?,"weather"?,"architecture"?,"textures"?,"practicalLights"?}
- {"type":"add_art","kind"("prop"|"wardrobe"|"vehicle"),"name","description"?,"materials"?,"condition"?,"era"?,"distinctive"?}
- {"type":"add_music_cue","name","sceneRef"?,"intent"?,"genre"?,"mood"?,"tempo"?,"instrumentation"?,"era"?,"structure"?,"vocals"?("instrumental"|"vocals"|"either"),"lyricTheme"?,"durationSec"?:number}
- {"type":"add_voice","name","characterName"?,"ageGender"?,"accent"?,"timbre"?,"pitch"?,"pacing"?,"energy"?,"texture"?,"emotionalRange"?,"sampleLine"?}
- {"type":"select","scene"?:"<name or id>","shot"?:"<name or id>"} — focus the UI on something you created or changed

Spec vocabulary: size EWS|WS|MWS|MS|MCU|CU|ECU|Insert; angle eye level|low|high|overhead|dutch|over-shoulder|POV; movement locked|handheld|push-in|pull-back|orbit|pan|tilt|track|crane|drone|steadicam; lens 14mm|24mm|35mm|50mm|85mm|135mm|anamorphic|macro|tilt-shift. targetModel ids: gpt-image-2, midjourney, krea, flux-3, seedance-2, minimax-h3, ltx-2.3, kling, sora, veo, comfyui.`

const AD_SYSTEM = `You are the First AD inside Agent-Slate, a prompt studio for AI filmmaking (fork of Slate). The director talks to you about what they want; you talk it through with them like a sharp, experienced first assistant director — then you run the set.

RESPONSE FORMAT — always return exactly this JSON, nothing else:
{"reply":"<what you say to the director — concise, collaborative, concrete>","actions":[...]}

CONVERSATION DISCIPLINE:
- If the intent is vague, ask the one or two questions that matter most (duration? model? tone? coverage style?) and return "actions": [].
- Don't interrogate — after at most one round of questions, make strong professional assumptions, state them briefly in the reply, and ACT.
- When you act, the reply should read like an AD reporting back: what you set up and why, in a few tight sentences. Never paste full prompts into the reply — they're in the shots.
- When the director asks for changes, use update_shot/update_scene on the existing work, not duplicates.

PROMPT QUALITY: every "prompt" you write uses the sectioned format below, in vivid, concrete, production-ready cinematography language — no vague adjectives, no "8k masterpiece" filler. Repeat character/location descriptions verbatim from the bible in every prompt (generators see one prompt at a time). Respect all continuity facts.

# Subject
...
# Composition
...
# Lighting
...
# Camera
...
# Style
...
# Mood
...

(Only include sections with content, keep this order.)

${ACTION_DOC}`

function inventory(p: Project): string {
  const lines: string[] = ['CURRENT SCENE/SHOT INVENTORY (ids in brackets):']
  if (!p.scenes.length) lines.push('- (no scenes yet)')
  for (const sc of p.scenes) {
    lines.push(`- Scene "${sc.name}" [${sc.id}]${sc.synopsis ? ` — ${sc.synopsis}` : ''}`)
    for (const sh of sc.shots) {
      const s = sh.spec
      lines.push(
        `    · Shot "${sh.name}" [${sh.id}] ${[
          s.durationSec ? `${s.durationSec}s` : null,
          s.size,
          s.movement,
          sh.targetModel
        ]
          .filter(Boolean)
          .join(', ')} — ${sh.intent || sh.prompt.slice(0, 80).replace(/\n/g, ' ') || 'empty'}`
      )
    }
  }
  return lines.join('\n')
}

export async function runFirstAD(
  p: Project,
  history: Array<{ role: 'user' | 'assistant'; text: string }>,
  message: string
): Promise<BrainResult> {
  const convo = history
    .slice(-14)
    .map((m) => `${m.role === 'user' ? 'DIRECTOR' : 'FIRST AD'}: ${m.text}`)
    .join('\n\n')
  return runBrain(p, {
    task: 'first-ad',
    tier: 'top',
    expectJson: true,
    system: AD_SYSTEM,
    prompt: `${projectContext(p)}\n\n${inventory(p)}\n\n${convo ? `CONVERSATION SO FAR:\n${convo}\n\n` : ''}DIRECTOR: ${message}`
  })
}

// ---------- Applying actions ----------

function findScene(p: Project, ref: string): Scene | undefined {
  const needle = ref.trim().toLowerCase()
  return p.scenes.find((s) => s.id === ref) ?? p.scenes.find((s) => s.name.toLowerCase() === needle)
}

function findShotAnyScene(p: Project, ref: string): { scene: Scene; shot: Shot } | undefined {
  const needle = ref.trim().toLowerCase()
  for (const scene of p.scenes) {
    const shot = scene.shots.find((s) => s.id === ref) ?? scene.shots.find((s) => s.name.toLowerCase() === needle)
    if (shot) return { scene, shot }
  }
  return undefined
}

function applySpec(spec: ShotSpec, patch?: SpecPatch): void {
  if (!patch) return
  for (const k of ['durationSec', 'fps', 'aspectRatio', 'size', 'angle', 'lens', 'movement'] as const) {
    if (k in patch) (spec[k] as unknown) = patch[k] ?? null
  }
}

const str = (v: unknown): string => (typeof v === 'string' ? v : '')

export interface ApplyResult {
  receipts: string[]
  focus: { sceneId: string; shotId: string | null } | null
}

/** Mutates the given project draft in place; returns receipts + UI focus. */
export function applyAdActions(p: Project, actions: AdAction[]): ApplyResult {
  const receipts: string[] = []
  let focus: ApplyResult['focus'] = null

  for (const a of actions) {
    try {
      switch (a.type) {
        case 'update_project': {
          if (a.logline !== undefined) p.logline = a.logline
          if (a.world !== undefined) p.world = a.world
          if (a.defaults) Object.assign(p.defaults, a.defaults)
          receipts.push('✓ Updated project bible')
          break
        }
        case 'create_scene': {
          if (findScene(p, a.name)) {
            receipts.push(`• Scene "${a.name}" already exists — skipped`)
            break
          }
          p.scenes.push({ id: uid('scene'), name: a.name, synopsis: a.synopsis ?? '', shots: [] })
          const sc = p.scenes[p.scenes.length - 1]
          focus = { sceneId: sc.id, shotId: null }
          receipts.push(`✓ Created scene "${a.name}"`)
          break
        }
        case 'update_scene': {
          const sc = findScene(p, a.scene)
          if (!sc) {
            receipts.push(`✗ Scene "${a.scene}" not found`)
            break
          }
          if (a.name) sc.name = a.name
          if (a.synopsis !== undefined) sc.synopsis = a.synopsis
          receipts.push(`✓ Updated scene "${sc.name}"`)
          break
        }
        case 'create_shot': {
          const sc = findScene(p, a.scene)
          if (!sc) {
            receipts.push(`✗ Scene "${a.scene}" not found for new shot`)
            break
          }
          const shot = blankShot(a.name ?? `Shot ${String(sc.shots.length + 1).padStart(2, '0')}`, p.defaults)
          if (a.intent) shot.intent = a.intent
          if (a.prompt) shot.prompt = a.prompt
          applySpec(shot.spec, a.spec)
          if (a.targetModel) shot.targetModel = a.targetModel
          if (a.maxChars !== undefined) shot.maxChars = a.maxChars
          if (a.beatSheet) shot.beatSheet = a.beatSheet
          sc.shots.push(shot)
          focus = { sceneId: sc.id, shotId: shot.id }
          receipts.push(`✓ Created "${shot.name}" in "${sc.name}"${a.prompt ? ' with prompt' : ''}`)
          break
        }
        case 'update_shot': {
          const hit = findShotAnyScene(p, a.shot)
          if (!hit) {
            receipts.push(`✗ Shot "${a.shot}" not found`)
            break
          }
          const { scene, shot } = hit
          if (a.prompt !== undefined && a.prompt !== shot.prompt && shot.prompt.trim()) {
            shot.history.unshift({
              id: uid('v'),
              savedAt: new Date().toISOString(),
              label: 'before First AD change',
              prompt: shot.prompt
            })
            shot.history = shot.history.slice(0, 50)
          }
          if (a.name) shot.name = a.name
          if (a.intent !== undefined) shot.intent = a.intent
          if (a.prompt !== undefined) shot.prompt = a.prompt
          applySpec(shot.spec, a.spec)
          if (a.targetModel !== undefined) shot.targetModel = a.targetModel
          if (a.maxChars !== undefined) shot.maxChars = a.maxChars
          if (a.beatSheet !== undefined) shot.beatSheet = a.beatSheet
          shot.updatedAt = new Date().toISOString()
          focus = { sceneId: scene.id, shotId: shot.id }
          receipts.push(`✓ Updated "${shot.name}"`)
          break
        }
        case 'add_character': {
          p.characters.push({
            id: uid('char'),
            name: str(a.name) || 'Unnamed',
            age: str(a.age),
            gender: str(a.gender),
            ethnicity: str(a.ethnicity),
            faceFeatures: str(a.faceFeatures),
            hair: str(a.hair),
            clothing: str(a.clothing),
            expression: str(a.expression),
            eyeDirection: str(a.eyeDirection),
            mood: str(a.mood),
            environment: str(a.environment),
            keyLightSide: str(a.keyLightSide) || 'Key light from left',
            lightingMood: str(a.lightingMood) || 'Natural soft light',
            scenario: 'cinematic',
            notes: ''
          })
          receipts.push(`✓ Cast "${str(a.name)}"`)
          break
        }
        case 'add_location': {
          const ie = str(a.interiorExterior)
          p.locations.push({
            id: uid('loc'),
            name: str(a.name) || 'Unnamed',
            interiorExterior: ie === 'interior' || ie === 'both' ? (ie as 'interior' | 'both') : 'exterior',
            description: str(a.description),
            timeOfDay: str(a.timeOfDay),
            weather: str(a.weather),
            architecture: str(a.architecture),
            textures: str(a.textures),
            practicalLights: str(a.practicalLights),
            notes: ''
          })
          receipts.push(`✓ Scouted "${str(a.name)}"`)
          break
        }
        case 'add_art': {
          const kind = str(a.kind)
          p.artDept.push({
            id: uid('art'),
            kind: kind === 'wardrobe' || kind === 'vehicle' ? (kind as 'wardrobe' | 'vehicle') : 'prop',
            name: str(a.name) || 'Unnamed',
            description: str(a.description),
            materials: str(a.materials),
            condition: str(a.condition),
            era: str(a.era),
            distinctive: str(a.distinctive),
            notes: ''
          })
          receipts.push(`✓ Added ${kind || 'prop'} "${str(a.name)}"`)
          break
        }
        case 'add_music_cue': {
          p.music = p.music ?? []
          const vocals = str(a.vocals)
          p.music.push({
            id: uid('cue'),
            name: str(a.name) || 'Untitled cue',
            sceneRef: str(a.sceneRef),
            intent: str(a.intent),
            genre: str(a.genre),
            mood: str(a.mood),
            tempo: str(a.tempo),
            instrumentation: str(a.instrumentation),
            era: str(a.era),
            structure: str(a.structure),
            vocals: vocals === 'vocals' || vocals === 'either' ? (vocals as 'vocals' | 'either') : 'instrumental',
            lyricTheme: str(a.lyricTheme),
            lyrics: '',
            durationSec: typeof a.durationSec === 'number' ? a.durationSec : null,
            notes: ''
          })
          receipts.push(`✓ Spotted cue "${str(a.name)}"`)
          break
        }
        case 'add_voice': {
          p.voices = p.voices ?? []
          const charName = str(a.characterName).toLowerCase()
          const character = charName ? p.characters.find((c) => c.name.toLowerCase() === charName) : undefined
          p.voices.push({
            id: uid('voice'),
            name: str(a.name) || 'Unnamed voice',
            characterId: character?.id ?? null,
            ageGender: str(a.ageGender),
            accent: str(a.accent),
            timbre: str(a.timbre),
            pitch: str(a.pitch),
            pacing: str(a.pacing),
            energy: str(a.energy),
            texture: str(a.texture),
            emotionalRange: str(a.emotionalRange),
            sampleLine: str(a.sampleLine),
            notes: ''
          })
          receipts.push(`✓ Cast voice "${str(a.name)}"`)
          break
        }
        case 'select': {
          if (a.shot) {
            const hit = findShotAnyScene(p, a.shot)
            if (hit) focus = { sceneId: hit.scene.id, shotId: hit.shot.id }
          } else if (a.scene) {
            const sc = findScene(p, a.scene)
            if (sc) focus = { sceneId: sc.id, shotId: sc.shots[0]?.id ?? null }
          }
          break
        }
        default:
          receipts.push(`✗ Unknown action ${(a as { type?: string }).type ?? '?'}`)
      }
    } catch (e) {
      receipts.push(`✗ ${a.type} failed: ${e instanceof Error ? e.message : String(e)}`)
    }
  }
  return { receipts, focus }
}
