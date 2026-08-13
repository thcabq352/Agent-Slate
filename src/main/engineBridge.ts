// Bridge to slate-engine HTTP control server (Phase 5).
// Reads engine-control.json written by `slate-engine serve` (not Electron's
// electron-control.json), or starts a debug binary if present.

import { spawn, type ChildProcess } from 'child_process'
import { existsSync, openSync, readFileSync } from 'fs'
import { homedir } from 'os'
import { join } from 'path'
import { ffmpegStatusAsync, pathWithFfmpeg, resolveFfmpegBin, type FfmpegStatus } from './ffmpeg'

export interface EngineDescriptor {
  app?: string
  port: number
  token: string
  pid?: number
}

const ENGINE_APP = 'slate-engine'
const LEGACY_ENGINE_APP = 'slate'
const PREFERRED_JUDGE = 'qwen3.5:9b'
const JUDGE_FALLBACKS = [
  'qwen3.5:9b',
  'qwen3-vl:8b',
  'qwen3-vl:30b',
  'qwen3.6:35b',
  'llava',
  'llava:latest'
]
const COMFY_CANDIDATES = ['http://127.0.0.1:8188', 'http://localhost:8188', 'http://127.0.0.1:8000']
const OLLAMA_CANDIDATES = ['http://127.0.0.1:11434', 'http://localhost:11434']

export interface EngineHealth {
  engine?: boolean
  comfy?: { ok: boolean; url?: string; error?: string }
  vision?: {
    ready?: boolean
    model?: string
    hint?: string
    endpoint?: string
    preferredModel?: string
  }
  qualityGate?: {
    passThreshold?: number
    maxRetries?: number
    judgeModel?: string
  }
  dryRun?: boolean
  packsDir?: string
  packsOk?: boolean
  ffmpeg?: FfmpegStatus
  [key: string]: unknown
}

export interface EngineJobStatus {
  active?: boolean
  step?: string
  projectId?: string
  message?: string
  continuitySummary?: string
  lastShotId?: string
  scenePlan?: string
  cancelRequested?: boolean
}

let child: ChildProcess | null = null

function configBase(): string {
  return process.platform === 'win32'
    ? process.env.APPDATA || join(homedir(), 'AppData', 'Roaming')
    : join(homedir(), '.config')
}

function descriptorPath(): string {
  return join(configBase(), 'slate', 'engine-control.json')
}

function legacyDescriptorPath(): string {
  return join(configBase(), 'slate', 'control.json')
}

function parseDescriptor(raw: string): EngineDescriptor | null {
  try {
    const d = JSON.parse(raw) as EngineDescriptor
    if (!d?.port || !d?.token) return null
    if (d.app && d.app !== ENGINE_APP && d.app !== LEGACY_ENGINE_APP) return null
    return d
  } catch {
    return null
  }
}

export function readDescriptor(): EngineDescriptor | null {
  for (const path of [descriptorPath(), legacyDescriptorPath()]) {
    try {
      const d = parseDescriptor(readFileSync(path, 'utf8'))
      if (d) return d
    } catch {
      /* missing */
    }
  }
  return null
}

function repoRoot(): string {
  return join(__dirname, '../..')
}

function engineBinaryCandidates(): string[] {
  const root = repoRoot()
  const names =
    process.platform === 'win32' ? ['slate-engine.exe', 'slate-engine'] : ['slate-engine']
  const dirs = [
    join(root, 'target', 'release'),
    join(root, 'target', 'debug'),
    join(process.cwd(), 'target', 'release'),
    join(process.cwd(), 'target', 'debug')
  ]
  const out: string[] = []
  for (const dir of dirs) {
    for (const n of names) {
      out.push(join(dir, n))
    }
  }
  return out
}

async function fetchOk(url: string, ms = 2500): Promise<{ ok: boolean; status: number; text: string }> {
  const ctrl = new AbortController()
  const t = setTimeout(() => ctrl.abort(), ms)
  try {
    const res = await fetch(url, { signal: ctrl.signal })
    const text = await res.text()
    return { ok: res.ok, status: res.status, text }
  } catch (e) {
    return { ok: false, status: 0, text: e instanceof Error ? e.message : String(e) }
  } finally {
    clearTimeout(t)
  }
}

function looksLikeVision(id: string): boolean {
  const l = id.toLowerCase()
  return (
    l.includes('vl') ||
    l.includes('vision') ||
    l.includes('llava') ||
    l.includes('minicpm-v') ||
    l.includes('qwen3.5') ||
    l.includes('qwen2.5-vl') ||
    l.includes('qwen2-vl') ||
    l.includes('gemma3') ||
    l.includes('pixtral')
  )
}

function pickVisionModel(ids: string[]): string | null {
  const lower = ids.map((id) => id.toLowerCase())
  const find = (want: string): string | undefined => {
    const w = want.toLowerCase()
    const i = lower.findIndex((id) => id === w || id.startsWith(`${w}:`) || id.startsWith(w))
    return i >= 0 ? ids[i] : undefined
  }
  for (const tag of JUDGE_FALLBACKS) {
    const hit = find(tag)
    if (hit) return hit
  }
  return ids.find(looksLikeVision) ?? null
}

function parseModelIds(body: string): string[] {
  try {
    const v = JSON.parse(body) as {
      data?: Array<{ id?: string }>
      models?: Array<{ name?: string; model?: string }>
    }
    if (Array.isArray(v.data)) {
      return v.data.map((m) => m.id).filter((id): id is string => Boolean(id))
    }
    if (Array.isArray(v.models)) {
      return v.models
        .map((m) => m.name || m.model)
        .filter((id): id is string => Boolean(id))
    }
  } catch {
    /* ignore */
  }
  return []
}

/** Direct loopback probes so the dock can see Comfy/Ollama even if the engine descriptor is stale. */
export async function probeSidecars(): Promise<Pick<EngineHealth, 'comfy' | 'vision'>> {
  let comfy: EngineHealth['comfy'] = {
    ok: false,
    url: COMFY_CANDIDATES[0],
    error: 'ComfyUI not reachable on 8188 (start the API server).'
  }
  for (const base of COMFY_CANDIDATES) {
    const stats = await fetchOk(`${base}/system_stats`)
    if (stats.ok) {
      comfy = { ok: true, url: base }
      break
    }
    const queue = await fetchOk(`${base}/queue`)
    if (queue.ok) {
      comfy = { ok: true, url: base }
      break
    }
    comfy = { ok: false, url: base, error: `ComfyUI ${base}: ${stats.text || `HTTP ${stats.status}`}` }
  }

  let vision: EngineHealth['vision'] = {
    ready: false,
    preferredModel: PREFERRED_JUDGE,
    hint: `No local model server found. Start Ollama and pull a vision model: ollama pull ${PREFERRED_JUDGE}`
  }
  for (const base of OLLAMA_CANDIDATES) {
    const openai = await fetchOk(`${base}/v1/models`, 4000)
    let ids = openai.ok ? parseModelIds(openai.text) : []
    if (!ids.length) {
      const tags = await fetchOk(`${base}/api/tags`, 4000)
      if (tags.ok) ids = parseModelIds(tags.text)
    }
    if (!openai.ok && ids.length === 0) continue
    const model = pickVisionModel(ids)
    if (model) {
      vision = { ready: true, model, endpoint: `${base}/v1`, preferredModel: PREFERRED_JUDGE }
      break
    }
    vision = {
      ready: false,
      endpoint: `${base}/v1`,
      preferredModel: PREFERRED_JUDGE,
      hint: ids.length
        ? `Ollama is up but no vision judge matched. Prefer \`${PREFERRED_JUDGE}\`. Installed: ${ids.slice(0, 6).join(', ')}`
        : `Ollama is up at ${base} but listed no models. Run: ollama pull ${PREFERRED_JUDGE}`
    }
    break
  }

  return { comfy, vision }
}

async function invokeLive(desc: EngineDescriptor, tool: string, args: Record<string, unknown>): Promise<unknown> {
  const res = await fetch(`http://127.0.0.1:${desc.port}/invoke`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${desc.token}`
    },
    body: JSON.stringify({ tool, args })
  })
  const body = (await res.json()) as { result?: unknown; error?: string }
  if (!res.ok) throw new Error(body.error || `HTTP ${res.status}`)
  return body.result
}

/** Best-effort: start slate-engine serve if binary exists and not already up. */
export async function ensureEngine(): Promise<{
  ok: boolean
  message: string
  descriptor: EngineDescriptor | null
}> {
  const existing = readDescriptor()
  if (existing) {
    try {
      await invokeLive(existing, 'slate_health', {})
      return { ok: true, message: 'engine already running', descriptor: existing }
    } catch {
      /* stale descriptor */
    }
  }

  const bin = engineBinaryCandidates().find((p) => existsSync(p))
  if (!bin) {
    return {
      ok: false,
      message:
        'slate-engine binary not found. Build with `cargo build -p slate-engine` and run `cargo run -p slate-engine -- serve`, or keep serve running.',
      descriptor: null
    }
  }

  if (child && !child.killed) {
    // wait for descriptor
  } else {
    const packsDir = join(repoRoot(), 'workflows', 'packs')
    const logPath = join(configBase(), 'slate', 'engine-serve.log')
    let logFd: number | 'ignore' = 'ignore'
    try {
      logFd = openSync(logPath, 'a')
    } catch {
      logFd = 'ignore'
    }
    const ffmpegResolved = resolveFfmpegBin()
    child = spawn(bin, ['serve'], {
      detached: false,
      stdio: ['ignore', logFd, logFd],
      windowsHide: true,
      cwd: repoRoot(),
      env: {
        ...process.env,
        PATH: pathWithFfmpeg(process.env.PATH),
        SLATE_PACKS_DIR: process.env.SLATE_PACKS_DIR || packsDir,
        ...(ffmpegResolved ? { SLATE_FFMPEG: ffmpegResolved } : {})
      }
    })
    child.unref?.()
  }

  for (let i = 0; i < 40; i++) {
    await new Promise((r) => setTimeout(r, 250))
    const d = readDescriptor()
    if (d) {
      try {
        await invokeLive(d, 'slate_health', {})
        return { ok: true, message: `started ${bin}`, descriptor: d }
      } catch {
        /* keep waiting */
      }
    }
  }
  return {
    ok: false,
    message:
      'timed out waiting for slate-engine engine-control.json. Rebuild with `cargo build -p slate-engine` (the dock reads engine-control.json, not leftover control.json).',
    descriptor: readDescriptor()
  }
}

export async function invoke(tool: string, args: Record<string, unknown> = {}): Promise<unknown> {
  const desc = readDescriptor()
  if (!desc) {
    throw new Error("Agent-Slate engine isn't running — start `slate-engine serve` or call engineEnsure first.")
  }
  return invokeLive(desc, tool, args)
}

export async function health(): Promise<EngineHealth> {
  const sidecars = await probeSidecars()
  const ffmpeg = await ffmpegStatusAsync()
  const desc = readDescriptor()
  if (!desc) {
    return { engine: false, ...sidecars, ffmpeg }
  }
  try {
    const h = (await invokeLive(desc, 'slate_health', {})) as EngineHealth
    return {
      ...sidecars,
      ...h,
      engine: true,
      comfy: h.comfy?.ok ? h.comfy : sidecars.comfy,
      vision: h.vision?.ready ? h.vision : sidecars.vision,
      ffmpeg: h.ffmpeg?.ok ? h.ffmpeg : ffmpeg
    }
  } catch {
    return { engine: false, ...sidecars, ffmpeg }
  }
}

export async function status(): Promise<EngineJobStatus> {
  try {
    return (await invoke('slate_status', {})) as EngineJobStatus
  } catch (e) {
    return {
      active: false,
      step: 'offline',
      message: e instanceof Error ? e.message : String(e)
    }
  }
}

export function stopChild(): void {
  if (child && !child.killed) {
    try {
      child.kill()
    } catch {
      /* ignore */
    }
  }
  child = null
}
