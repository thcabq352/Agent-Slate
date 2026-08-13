// Stills Library — harvest continuity stills from finished takes.
// Discovers circled takes in Circle Take projects (the suite's dailies app)
// and extracts representative frames from any clip with the system ffmpeg,
// honoring a take's select window when one is set.

import { execFile } from 'child_process'
import { promises as fs } from 'fs'
import { existsSync } from 'fs'
import { createHash } from 'crypto'
import { join, isAbsolute, basename } from 'path'
import { homedir } from 'os'
import type { CircledTake } from '../shared/types'
import { ffmpegBin, ffmpegInstallHint } from './ffmpeg'

const MAX_STILLS = 8

function run(cmd: string, args: string[]): Promise<void> {
  return new Promise((resolve, reject) => {
    execFile(cmd, args, { timeout: 120000, windowsHide: true }, (err, _out, stderr) =>
      err ? reject(new Error(stderr?.slice(0, 400) || err.message)) : resolve()
    )
  })
}

export interface CircleTakePaths {
  recentsFile: string
}

/** Circle Take writes recents.json in Electron userData (`app.getPath('userData')`). */
export function circleTakeRecentsCandidates(): string[] {
  const home = homedir()
  const out: string[] = []
  const add = (p?: string | null): void => {
    if (p && !out.includes(p)) out.push(p)
  }
  add(process.env.CIRCLE_TAKE_RECENTS || null)
  add(join(home, 'Library', 'Application Support', 'circle-take', 'recents.json'))
  add(join(home, 'Library', 'Application Support', 'Circle Take', 'recents.json'))
  const roaming =
    process.env.APPDATA ||
    (process.platform === 'win32' ? join(home, 'AppData', 'Roaming') : null)
  if (roaming) {
    add(join(roaming, 'circle-take', 'recents.json'))
    add(join(roaming, 'Circle Take', 'recents.json'))
  }
  const local =
    process.env.LOCALAPPDATA ||
    (process.platform === 'win32' ? join(home, 'AppData', 'Local') : null)
  if (local) {
    add(join(local, 'circle-take', 'recents.json'))
    add(join(local, 'Circle Take', 'recents.json'))
  }
  add(join(home, '.config', 'circle-take', 'recents.json'))
  add(join(home, '.config', 'Circle Take', 'recents.json'))
  return out
}

export function resolveCircleTakeRecents(
  candidates: string[] = circleTakeRecentsCandidates()
): string {
  return (
    candidates.find((p) => existsSync(p)) ??
    candidates[0] ??
    join(homedir(), 'Library', 'Application Support', 'circle-take', 'recents.json')
  )
}

export function defaultCircleTakePaths(): CircleTakePaths {
  return { recentsFile: resolveCircleTakeRecents() }
}

interface CtakeTake {
  id?: string
  shotId?: string
  mediaPath?: string
  fileName?: string
  circled?: boolean
  rating?: number
  inSec?: number
  outSec?: number
}

interface CtakeProject {
  name?: string
  shots?: Array<{ id?: string; name?: string }>
  takes?: CtakeTake[]
}

/** All circled takes across every Circle Take project in recents, newest
 *  project first. Pure disk reads; missing files simply drop out. */
export async function discoverCircledTakes(
  paths: CircleTakePaths = defaultCircleTakePaths()
): Promise<CircledTake[]> {
  let recents: Array<{ path?: string; openedAt?: string }> = []
  try {
    recents = JSON.parse(await fs.readFile(paths.recentsFile, 'utf8'))
  } catch {
    return []
  }
  const seen = new Set<string>()
  const out: CircledTake[] = []
  for (const r of recents) {
    if (!r.path || seen.has(r.path) || !existsSync(r.path)) continue
    seen.add(r.path)
    try {
      const doc = JSON.parse(await fs.readFile(join(r.path, 'project.json'), 'utf8')) as CtakeProject
      const shotName = new Map<string, string>()
      for (const s of doc.shots ?? []) if (s.id && s.name) shotName.set(s.id, s.name)
      for (const t of doc.takes ?? []) {
        if (!t.circled || !t.mediaPath) continue
        const media = isAbsolute(t.mediaPath) ? t.mediaPath : join(r.path, t.mediaPath)
        if (!existsSync(media)) continue
        out.push({
          project: doc.name ?? basename(r.path, '.ctake'),
          shot: (t.shotId && shotName.get(t.shotId)) || null,
          mediaPath: media,
          fileName: t.fileName ?? basename(media),
          rating: typeof t.rating === 'number' ? t.rating : 0,
          inSec: typeof t.inSec === 'number' ? t.inSec : null,
          outSec: typeof t.outSec === 'number' ? t.outSec : null
        })
      }
    } catch {
      /* unreadable project — skip */
    }
  }
  return out
}

/** Extract up to MAX_STILLS sharp frames from a clip into cacheRoot/stills.
 *  When a select window is given, only that slice is sampled. */
export async function extractStills(
  cacheRoot: string,
  mediaPath: string,
  inSec?: number | null,
  outSec?: number | null
): Promise<string[]> {
  const hash = createHash('sha1')
    .update(`${mediaPath}:${inSec ?? ''}:${outSec ?? ''}`)
    .digest('hex')
    .slice(0, 12)
  const outDir = join(cacheRoot, 'stills', hash)
  await fs.mkdir(outDir, { recursive: true })

  const existing = (await fs.readdir(outDir)).filter((f) => f.endsWith('.jpg'))
  if (existing.length > 0) return existing.sort().map((f) => join(outDir, f))

  const windowArgs: string[] = []
  if (inSec != null && inSec > 0) windowArgs.push('-ss', String(inSec))
  if (outSec != null && outSec > 0) windowArgs.push('-to', String(outSec))

  // Pass 1 — scene changes inside the window.
  await run(ffmpegBin(), [
    ...windowArgs,
    '-i', mediaPath,
    '-vf', "select='gt(scene,0.24)',scale=768:-2",
    '-vsync', 'vfr',
    '-frames:v', String(MAX_STILLS),
    '-q:v', '3',
    join(outDir, 'scene-%02d.jpg')
  ]).catch(() => undefined)

  let frames = (await fs.readdir(outDir)).filter((f) => f.endsWith('.jpg'))

  // Pass 2 — steady sampling when the clip cuts too little.
  if (frames.length < 4) {
    await run(ffmpegBin(), [
      ...windowArgs,
      '-i', mediaPath,
      '-vf', 'fps=1,scale=768:-2',
      '-frames:v', String(MAX_STILLS - frames.length),
      '-q:v', '3',
      join(outDir, 'tick-%02d.jpg')
    ]).catch(() => undefined)
    frames = (await fs.readdir(outDir)).filter((f) => f.endsWith('.jpg'))
  }

  if (frames.length === 0) {
    throw new Error(`No frames could be extracted — ${ffmpegInstallHint()}`)
  }
  return frames.sort().map((f) => join(outDir, f))
}
