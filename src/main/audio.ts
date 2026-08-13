// Audio reference analysis — decode with the system ffmpeg, then measure the
// signal locally: tempo, pitch register, dynamics, brightness, structure.
// The numbers travel to the brain, which writes the style language.

import { spawn } from 'child_process'
import type { AudioFingerprint } from '../shared/types'
import { ffmpegBin, ffmpegInstallHint } from './ffmpeg'

export type { AudioFingerprint }

const SAMPLE_RATE = 16000
const FRAME = 512 // 32ms
const MAX_SECONDS = 90

function decodePcm(path: string): Promise<{ pcm: Int16Array; fullDurationSec: number }> {
  return new Promise((resolve, reject) => {
    // Duration probe piggybacks on stderr; decode capped sample from the start.
    const child = spawn(ffmpegBin(), [
      '-i', path,
      '-map', 'a:0',
      '-ac', '1',
      '-ar', String(SAMPLE_RATE),
      '-t', String(MAX_SECONDS),
      '-f', 's16le',
      'pipe:1'
    ], { windowsHide: true })
    const chunks: Buffer[] = []
    let err = ''
    child.stdout.on('data', (d: Buffer) => chunks.push(d))
    child.stderr.on('data', (d) => (err += d))
    child.on('error', (e) =>
      reject(new Error(`ffmpeg failed: ${e.message}. ${ffmpegInstallHint()}`))
    )
    child.on('close', (code) => {
      if (code !== 0 && chunks.length === 0) {
        reject(new Error(err.includes('does not contain any stream') || err.includes('Stream map')
          ? 'No audio track found in this file.'
          : `Could not decode audio (ffmpeg exit ${code}).`))
        return
      }
      const buf = Buffer.concat(chunks)
      const pcm = new Int16Array(buf.buffer, buf.byteOffset, Math.floor(buf.length / 2))
      const durMatch = err.match(/Duration:\s*(\d+):(\d+):(\d+\.?\d*)/)
      const fullDurationSec = durMatch
        ? Number(durMatch[1]) * 3600 + Number(durMatch[2]) * 60 + Number(durMatch[3])
        : pcm.length / SAMPLE_RATE
      resolve({ pcm, fullDurationSec })
    })
  })
}

/** Per-frame RMS (linear 0..1) and zero-crossing rate. */
function frameStats(pcm: Int16Array): { rms: Float64Array; zcr: Float64Array } {
  const n = Math.floor(pcm.length / FRAME)
  const rms = new Float64Array(n)
  const zcr = new Float64Array(n)
  for (let f = 0; f < n; f++) {
    let sum = 0
    let crossings = 0
    const base = f * FRAME
    for (let i = 0; i < FRAME; i++) {
      const s = pcm[base + i] / 32768
      sum += s * s
      if (i > 0 && ((pcm[base + i - 1] >= 0) !== (pcm[base + i] >= 0))) crossings++
    }
    rms[f] = Math.sqrt(sum / FRAME)
    zcr[f] = crossings / FRAME
  }
  return { rms, zcr }
}

/** Fundamental frequency of one window via normalized autocorrelation (50–400 Hz). */
function windowF0(pcm: Int16Array, start: number): number | null {
  const W = 1024
  if (start + W * 2 > pcm.length) return null
  const minLag = Math.floor(SAMPLE_RATE / 400)
  const maxLag = Math.floor(SAMPLE_RATE / 50)
  let energy = 0
  for (let i = 0; i < W; i++) energy += (pcm[start + i] / 32768) ** 2
  if (energy / W < 1e-5) return null
  let bestCorr = 0
  const corrs = new Float64Array(maxLag + 1)
  for (let lag = minLag; lag <= maxLag; lag++) {
    let corr = 0
    for (let i = 0; i < W; i++) corr += (pcm[start + i] / 32768) * (pcm[start + i + lag] / 32768)
    corr /= energy
    corrs[lag] = corr
    if (corr > bestCorr) bestCorr = corr
  }
  if (bestCorr <= 0.45) return null
  // Subharmonic suppression: a periodic signal correlates at every multiple of
  // its true period — take the SHORTEST lag that is a local peak near the max.
  for (let lag = minLag; lag <= maxLag; lag++) {
    if (
      corrs[lag] >= bestCorr * 0.9 &&
      corrs[lag] >= (corrs[lag - 1] ?? 0) &&
      corrs[lag] >= (corrs[lag + 1] ?? 0)
    ) {
      return SAMPLE_RATE / lag
    }
  }
  return null
}

/** Tempo from the onset-strength envelope via autocorrelation over 60–190 BPM. */
function estimateBpm(rms: Float64Array): { bpm: number | null; confidence: 'low' | 'medium' | 'high' } {
  const framesPerSec = SAMPLE_RATE / FRAME
  const onsets = new Float64Array(rms.length)
  for (let i = 1; i < rms.length; i++) onsets[i] = Math.max(0, rms[i] - rms[i - 1])
  const mean = onsets.reduce((a, b) => a + b, 0) / onsets.length
  if (mean < 1e-6) return { bpm: null, confidence: 'low' }
  for (let i = 0; i < onsets.length; i++) onsets[i] -= mean

  const minLag = Math.round((60 / 190) * framesPerSec)
  const maxLag = Math.round((60 / 60) * framesPerSec)
  let bestLag = 0
  let bestCorr = -Infinity
  let norm = 0
  for (let i = 0; i < onsets.length; i++) norm += onsets[i] * onsets[i]
  if (norm === 0) return { bpm: null, confidence: 'low' }
  for (let lag = minLag; lag <= maxLag; lag++) {
    let corr = 0
    for (let i = 0; i + lag < onsets.length; i++) corr += onsets[i] * onsets[i + lag]
    corr /= norm
    if (corr > bestCorr) {
      bestCorr = corr
      bestLag = lag
    }
  }
  if (bestLag === 0) return { bpm: null, confidence: 'low' }
  const bpm = Math.round((60 * framesPerSec) / bestLag)
  const confidence = bestCorr > 0.35 ? 'high' : bestCorr > 0.18 ? 'medium' : 'low'
  return { bpm: confidence === 'low' ? null : bpm, confidence }
}

function describeArc(rms: Float64Array): string {
  const thirds = 3
  const seg = Math.floor(rms.length / thirds)
  if (seg === 0) return 'too short to read'
  const levels: number[] = []
  for (let t = 0; t < thirds; t++) {
    let s = 0
    for (let i = t * seg; i < (t + 1) * seg; i++) s += rms[i]
    levels.push(s / seg)
  }
  const [a, b, c] = levels
  const rel = (x: number, y: number): string => (x > y * 1.35 ? '>' : y > x * 1.35 ? '<' : '=')
  const ab = rel(a, b)
  const bc = rel(b, c)
  if (ab === '<' && bc === '<') return 'steady build from quiet open to loud finish'
  if (ab === '>' && bc === '>') return 'loud open decaying to a quiet close'
  if (ab === '<' && bc === '>') return 'rises to a peak mid-way, then falls away'
  if (ab === '>' && bc === '<') return 'strong open, quiet middle, returns big at the end'
  return 'relatively even energy throughout'
}

export function computeFingerprint(pcm: Int16Array, fullDurationSec: number): AudioFingerprint {
  const { rms, zcr } = frameStats(pcm)

  // Silence
  const sorted = [...rms].sort((x, y) => x - y)
  const p95 = sorted[Math.floor(sorted.length * 0.95)] || 1e-6
  const silenceThresh = Math.max(p95 * 0.06, 0.004)
  let silent = 0
  let longest = 0
  let run = 0
  for (const v of rms) {
    if (v < silenceThresh) {
      silent++
      run++
      longest = Math.max(longest, run)
    } else run = 0
  }
  const framesPerSec = SAMPLE_RATE / FRAME

  // Dynamics: loud vs soft (of non-silent material)
  const active = sorted.filter((v) => v >= silenceThresh)
  const q = (arr: number[], p: number): number => arr[Math.min(arr.length - 1, Math.floor(arr.length * p))] || 1e-6
  const dynamicRangeDb = active.length > 4 ? 20 * Math.log10(q(active, 0.95) / Math.max(q(active, 0.2), 1e-6)) : 0

  // Pitch: sample every ~250ms on active frames
  const f0s: number[] = []
  const hop = Math.floor(SAMPLE_RATE / 4)
  for (let s = 0; s + 2048 < pcm.length; s += hop) {
    const frame = Math.floor(s / FRAME)
    if (rms[frame] < silenceThresh) continue
    const f0 = windowF0(pcm, s)
    if (f0) f0s.push(f0)
  }
  f0s.sort((a, b) => a - b)
  const pitchMedianHz = f0s.length > 6 ? f0s[Math.floor(f0s.length / 2)] : null
  let pitchSpreadSemitones: number | null = null
  if (pitchMedianHz && f0s.length > 10) {
    const lo = f0s[Math.floor(f0s.length * 0.1)]
    const hi = f0s[Math.floor(f0s.length * 0.9)]
    pitchSpreadSemitones = Math.round(12 * Math.log2(hi / lo))
  }

  // Brightness from mean ZCR of active frames
  let zsum = 0
  let zn = 0
  for (let i = 0; i < zcr.length; i++) {
    if (rms[i] >= silenceThresh) {
      zsum += zcr[i]
      zn++
    }
  }
  const meanZcr = zn ? zsum / zn : 0
  const brightness = meanZcr < 0.06 ? 'dark' : meanZcr > 0.14 ? 'bright' : 'balanced'

  const { bpm, confidence } = estimateBpm(rms)

  return {
    durationSec: Math.round(fullDurationSec * 10) / 10,
    sampledSec: Math.round((pcm.length / SAMPLE_RATE) * 10) / 10,
    bpmEstimate: bpm,
    bpmConfidence: confidence,
    pitchMedianHz: pitchMedianHz ? Math.round(pitchMedianHz) : null,
    pitchSpreadSemitones,
    voicedRatio: Math.round((f0s.length / Math.max(1, Math.floor(pcm.length / hop))) * 100) / 100,
    brightness,
    dynamicRangeDb: Math.round(dynamicRangeDb * 10) / 10,
    energyArc: describeArc(rms),
    silenceRatio: Math.round((silent / rms.length) * 100) / 100,
    longestSilenceSec: Math.round((longest / framesPerSec) * 10) / 10
  }
}

export async function analyzeAudio(path: string): Promise<AudioFingerprint> {
  const { pcm, fullDurationSec } = await decodePcm(path)
  if (pcm.length < SAMPLE_RATE) throw new Error('Audio is too short to analyze (need at least 1 second).')
  return computeFingerprint(pcm, fullDurationSec)
}
