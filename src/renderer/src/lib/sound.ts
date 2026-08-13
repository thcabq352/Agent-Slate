// Sound Department — music cues and voice casting, compiled per target tool.

import type { Project, MusicCue, VoiceSheet, BrainResult, Scene, AudioFingerprint } from '../../../shared/types'
import { runBrain, projectContext } from './brainTasks'
import soundData from '../../../../data/sound-profiles.json'

export interface MusicProfile {
  id: string
  label: string
  vendor: string
  dialect: { guidance: string }
  limits: { maxChars: number | null; maxDurationSec: number | null }
  features: {
    lyrics: boolean
    instrumental: boolean
    structureTags: boolean
    bpmControl: boolean
    referenceAudio: boolean
    stems: boolean
  }
  notes: string
}

export interface VoiceProfile {
  id: string
  label: string
  vendor: string
  dialect: { guidance: string }
  limits: { maxChars: number | null }
  features: { voiceDesign: boolean; cloning: boolean; emotionTags: boolean; multilingual: boolean }
  notes: string
}

const data = soundData as { music: MusicProfile[]; voice: VoiceProfile[] }
export const MUSIC_PROFILES: MusicProfile[] = data.music
export const VOICE_PROFILES: VoiceProfile[] = data.voice

// ---------- Auto-fill ----------

export async function fillMusicCue(p: Project, description: string, scene: Scene | null): Promise<BrainResult> {
  return runBrain(p, {
    task: 'score-fill',
    tier: 'top',
    expectJson: true,
    prompt: `${projectContext(p, scene)}\n\nThe director described a music cue: "${description}"\n\nDesign the cue like a film composer spotting a scene — concrete musical language, not vibes. Return JSON with exactly these string fields (plus durationSec as number|null): {"name","intent","genre","mood","tempo","instrumentation","era","structure","vocals"("instrumental"|"vocals"|"either"),"lyricTheme","durationSec"}. tempo: feel + rough BPM. structure: how it evolves against the scene (e.g. "sparse pulse building to full kit at the reveal, hard out on the cut"). If vocals, lyricTheme is what the lyrics circle — not the lyrics themselves.`
  })
}

export async function writeLyrics(p: Project, cue: MusicCue): Promise<BrainResult> {
  return runBrain(p, {
    task: 'score-lyrics',
    tier: 'top',
    prompt: `${projectContext(p)}\n\nMUSIC CUE "${cue.name}": ${cue.intent}. Genre ${cue.genre}; mood ${cue.mood}; theme: ${cue.lyricTheme || 'derive from the film'}.\n\nWrite original song lyrics for this cue with section tags in brackets ([Intro], [Verse], [Chorus], [Bridge], [Outro] as fits the structure: ${cue.structure || 'standard'}). Keep them singable, concrete, and in the film's voice — no clichés. Return ONLY the tagged lyrics.`
  })
}

export async function fillVoice(
  p: Project,
  description: string,
  characterName: string | null
): Promise<BrainResult> {
  return runBrain(p, {
    task: 'voice-fill',
    tier: 'top',
    expectJson: true,
    prompt: `${projectContext(p)}\n\nDesign a voice for ${characterName ? `the character "${characterName}"` : 'this description'}: "${description}"\n\nThink like a voice director casting for film — specific, castable, physical. Return JSON with exactly these string fields: {"name","ageGender","accent","timbre","pitch","pacing","energy","texture","emotionalRange","sampleLine"}. timbre: the core color (warm/reedy/resonant...). texture: grain (gravel, breath, smoke...). sampleLine: one line of test dialogue IN CHARACTER that exercises the voice's range — emotionally alive, 15-25 words.`
  })
}

// ---------- Audio reference analysis ----------

function fingerprintText(fp: AudioFingerprint): string {
  return `MEASURED ACOUSTIC FINGERPRINT (local signal analysis of the reference — objective numbers, no listening involved):
- Duration ${fp.durationSec}s (analyzed first ${fp.sampledSec}s)
- Tempo estimate: ${fp.bpmEstimate ? `${fp.bpmEstimate} BPM (confidence ${fp.bpmConfidence})` : 'no stable pulse detected'}
- Pitch: ${fp.pitchMedianHz ? `median fundamental ~${fp.pitchMedianHz} Hz${fp.pitchSpreadSemitones ? `, range ~${fp.pitchSpreadSemitones} semitones` : ''}` : 'no clear pitched content'} · voiced/pitched ratio ${fp.voicedRatio}
- Spectral character: ${fp.brightness}
- Dynamic range (active material): ${fp.dynamicRangeDb} dB
- Energy arc: ${fp.energyArc}
- Silence: ${Math.round(fp.silenceRatio * 100)}% of running time, longest gap ${fp.longestSilenceSec}s

Interpretation cheatsheet: ~85-110 Hz median pitch suggests a low male voice, ~110-140 low-mid male, ~165-255 typical female range; wide semitone range = expressive/melodic, narrow = monotone/spoken. High silence ratio with short gaps = conversational pacing. For music: dark + low dynamic range = dense/compressed production; bright + high dynamics = sparse/acoustic.`
}

export async function analyzeMusicRef(
  p: Project,
  fp: AudioFingerprint,
  hint: string
): Promise<BrainResult> {
  return runBrain(p, {
    task: 'score-ref-analysis',
    tier: 'top',
    expectJson: true,
    prompt: `${projectContext(p)}\n\nThe director dropped in a music reference to recreate its style.\n${hint ? `Their note about it: "${hint}"\n` : ''}\n${fingerprintText(fp)}\n\nReverse-engineer a cue design that would land in the same sonic territory. Ground every claim in the measurements + the director's note; where the numbers can't tell you something (exact instruments, era), make one confident professional inference and mark it with "(inferred)". Return JSON: {"name","intent","genre","mood","tempo","instrumentation","era","structure","vocals"("instrumental"|"vocals"|"either"),"lyricTheme","durationSec":number|null,"styleNotes":"<2-3 sentences on what defines this reference's sound and how to ask a generator for it>"}. tempo should quote the measured BPM when confidence isn't low.`
  })
}

export async function analyzeVoiceRef(
  p: Project,
  fp: AudioFingerprint,
  hint: string
): Promise<BrainResult> {
  return runBrain(p, {
    task: 'voice-ref-analysis',
    tier: 'top',
    expectJson: true,
    prompt: `${projectContext(p)}\n\nThe director dropped in a voice reference to recreate its character.\n${hint ? `Their note about it: "${hint}"\n` : ''}\n${fingerprintText(fp)}\n\nDesign a voice sheet matching this reference's measurable character — pitch register from the fundamental, expressiveness from the semitone range, pacing from the silence pattern, energy from dynamics. Where numbers can't reach (accent, texture), infer cautiously from the director's note or leave modest and mark "(inferred)". Describe a voice TYPE — never claim to identify a specific person. Return JSON: {"name","ageGender","accent","timbre","pitch","pacing","energy","texture","emotionalRange","sampleLine","styleNotes":"<2 sentences on what defines this voice and what to emphasize when designing it>"}.`
  })
}

// ---------- Per-target compile ----------

export async function compileMusicPrompt(
  p: Project,
  cue: MusicCue,
  profile: MusicProfile
): Promise<BrainResult> {
  const wantsLyrics = cue.vocals !== 'instrumental' && profile.features.lyrics
  return runBrain(p, {
    task: `score-compile:${profile.id}`,
    tier: 'standard',
    expectJson: true,
    prompt: `${projectContext(p)}

MUSIC CUE TO COMPILE:
Name: ${cue.name}${cue.sceneRef ? ` (for ${cue.sceneRef})` : ''}
Dramatic intent: ${cue.intent}
Genre: ${cue.genre} · Mood: ${cue.mood} · Tempo: ${cue.tempo}
Instrumentation: ${cue.instrumentation}
Era/character: ${cue.era}
Structure: ${cue.structure}
Vocals: ${cue.vocals}${cue.lyricTheme ? ` · Lyric theme: ${cue.lyricTheme}` : ''}${cue.durationSec ? ` · Target length: ${cue.durationSec}s` : ''}
${cue.lyrics ? `\nEXISTING LYRICS (use verbatim):\n${cue.lyrics}` : ''}

TARGET: ${profile.label}.
DIALECT: ${profile.dialect.guidance}
${profile.limits.maxChars ? `HARD LIMIT: main prompt under ${profile.limits.maxChars} characters.` : ''}
${profile.limits.maxDurationSec && cue.durationSec && cue.durationSec > profile.limits.maxDurationSec ? `NOTE: target caps at ${profile.limits.maxDurationSec}s — flag this in "warnings".` : ''}
${!profile.features.structureTags ? 'This target ignores bracketed section tags — express structure in prose.' : ''}

Return JSON: {"prompt":"<the main prompt in this target's dialect>"${wantsLyrics ? ',"lyrics":"<lyrics formatted for this target, or empty if instrumental>"' : ''},"params":{<any settings worth noting for this target>},"warnings":[<strings, empty if none>]}`
  })
}

export async function compileVoicePrompt(
  p: Project,
  v: VoiceSheet,
  profile: VoiceProfile
): Promise<BrainResult> {
  return runBrain(p, {
    task: `voice-compile:${profile.id}`,
    tier: 'standard',
    expectJson: true,
    prompt: `${projectContext(p)}

VOICE TO COMPILE:
Name: ${v.name}
Age/gender: ${v.ageGender} · Accent: ${v.accent}
Timbre: ${v.timbre} · Pitch: ${v.pitch} · Texture: ${v.texture}
Pacing: ${v.pacing} · Energy: ${v.energy}
Emotional range: ${v.emotionalRange}
${v.notes ? `Notes: ${v.notes}` : ''}

TARGET: ${profile.label}.
DIALECT: ${profile.dialect.guidance}
${profile.limits.maxChars ? `HARD LIMIT: description under ${profile.limits.maxChars} characters.` : ''}

Return JSON: {"prompt":"<the voice-design description in this target's dialect>","previewText":"<test line(s) for auditioning the voice — build from this sample line, extend to ~40 words exercising the emotional range: ${v.sampleLine || 'write one in character'}>","params":{<any settings worth noting>},"warnings":[<strings, empty if none>]}`
  })
}

/** Direct a speakable line for Grok TTS (inline [pause]/[laugh] and wrapping <whisper>). */
export async function directGrokVo(p: Project, v: VoiceSheet, line: string): Promise<BrainResult> {
  return runBrain(p, {
    task: 'voice-direct-grok',
    tier: 'standard',
    prompt: `${projectContext(p)}

VOICE: ${v.name}
Age/gender: ${v.ageGender} · Accent: ${v.accent}
Timbre: ${v.timbre} · Pitch: ${v.pitch} · Texture: ${v.texture}
Pacing: ${v.pacing} · Energy: ${v.energy}
Emotional range: ${v.emotionalRange}
${v.notes ? `Notes: ${v.notes}` : ''}

LINE TO SPEAK:
${line.trim()}

Rewrite this as a Grok TTS performance. Keep the words the character would actually say. You may add:
- inline tags: [pause] [laugh] [sigh] [breath]
- wrapping tags: <whisper>…</whisper>
Use tags only when they serve the read. No quotes, no stage directions outside tags, no commentary.
Return ONLY the speakable text.`
  })
}

// ---------- Factories ----------

export function blankCue(id: string): MusicCue {
  return {
    id,
    name: '',
    sceneRef: '',
    intent: '',
    genre: '',
    mood: '',
    tempo: '',
    instrumentation: '',
    era: '',
    structure: '',
    vocals: 'instrumental',
    lyricTheme: '',
    lyrics: '',
    durationSec: null,
    notes: ''
  }
}

export function blankVoice(id: string): VoiceSheet {
  return {
    id,
    name: '',
    characterId: null,
    ageGender: '',
    accent: '',
    timbre: '',
    pitch: '',
    pacing: '',
    energy: '',
    texture: '',
    emotionalRange: '',
    sampleLine: '',
    notes: '',
    grokVoiceId: 'eve',
    voText: '',
    voLanguage: 'en'
  }
}
