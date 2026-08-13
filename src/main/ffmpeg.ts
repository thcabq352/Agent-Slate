// Resolve ffmpeg for GUI apps that inherit a stripped PATH (Electron / Finder / Start Menu).
// Prefer SLATE_FFMPEG, then common Windows/macOS/Linux install locations, then PATH.

import { execFile } from 'child_process'
import { readdirSync, statSync } from 'fs'
import { homedir } from 'os'
import { delimiter, dirname, join } from 'path'
import { ffmpegInstallHintFor, hostOsFromPlatform } from '../shared/installHints'

export interface FfmpegStatus {
  ok: boolean
  path: string | null
  version: string | null
  hint?: string
}

function isFile(p: string): boolean {
  try {
    return statSync(p).isFile()
  } catch {
    return false
  }
}

function ffmpegNames(): string[] {
  return process.platform === 'win32' ? ['ffmpeg.exe'] : ['ffmpeg']
}

/** Directories that often hold ffmpeg when PATH is empty. */
export function ffmpegCandidateDirs(): string[] {
  const home = homedir()
  const dirs: string[] = []
  const add = (p?: string | null): void => {
    if (p && !dirs.includes(p)) dirs.push(p)
  }

  add('/opt/homebrew/bin')
  add('/usr/local/bin')
  add('/usr/bin')
  add('/snap/bin')
  add(join(home, '.local', 'bin'))
  add(join(home, 'bin'))

  const pf = process.env.ProgramFiles || 'C:\\Program Files'
  const pf86 = process.env['ProgramFiles(x86)'] || 'C:\\Program Files (x86)'
  const local = process.env.LOCALAPPDATA
  add(join(home, 'scoop', 'shims'))
  add('C:\\ffmpeg\\bin')
  add(join(pf, 'ffmpeg', 'bin'))
  add(join(pf86, 'ffmpeg', 'bin'))
  add(join(pf, 'Gyan', 'ffmpeg', 'bin'))
  add('C:\\ProgramData\\chocolatey\\bin')
  add(join(pf, 'Git', 'usr', 'bin'))
  if (local) add(join(local, 'Microsoft', 'WinGet', 'Links'))

  for (const d of (process.env.PATH ?? '').split(delimiter)) add(d || null)
  return dirs
}

function findNamed(dir: string, names: string[], depth: number): string | null {
  if (depth < 0) return null
  let entries: string[]
  try {
    entries = readdirSync(dir)
  } catch {
    return null
  }
  for (const n of names) {
    const p = join(dir, n)
    if (isFile(p)) return p
  }
  if (depth === 0) return null
  for (const ent of entries) {
    const p = join(dir, ent)
    try {
      if (!statSync(p).isDirectory()) continue
    } catch {
      continue
    }
    const found = findNamed(p, names, depth - 1)
    if (found) return found
  }
  return null
}

function probeWingetFfmpeg(): string | null {
  const local = process.env.LOCALAPPDATA
  if (!local) return null
  const root = join(local, 'Microsoft', 'WinGet', 'Packages')
  let entries: string[]
  try {
    entries = readdirSync(root)
  } catch {
    return null
  }
  for (const ent of entries) {
    if (!/ffmpeg/i.test(ent)) continue
    const found = findNamed(join(root, ent), ffmpegNames(), 3)
    if (found) return found
  }
  return null
}

function envFfmpeg(): string | null {
  const raw = process.env.SLATE_FFMPEG || process.env.FFMPEG
  if (!raw) return null
  if (isFile(raw)) return raw
  for (const n of ffmpegNames()) {
    const nested = join(raw, n)
    if (isFile(nested)) return nested
  }
  return null
}

/** Absolute ffmpeg path, or null if nothing is installed. */
export function resolveFfmpegBin(): string | null {
  const fromEnv = envFfmpeg()
  if (fromEnv) return fromEnv
  const names = ffmpegNames()
  for (const dir of ffmpegCandidateDirs()) {
    for (const n of names) {
      const p = join(dir, n)
      if (isFile(p)) return p
    }
  }
  return probeWingetFfmpeg()
}

/** Path to pass to spawn/execFile. Bare `ffmpeg` if unresolved (PATH fallback). */
export function ffmpegBin(): string {
  return resolveFfmpegBin() ?? (process.platform === 'win32' ? 'ffmpeg.exe' : 'ffmpeg')
}

export function ffmpegInstallHint(): string {
  return ffmpegInstallHintFor(hostOsFromPlatform(process.platform))
}

/** PATH with ffmpeg candidate dirs prepended (for child processes). */
export function pathWithFfmpeg(base = process.env.PATH ?? ''): string {
  const extra = ffmpegCandidateDirs()
  const resolved = resolveFfmpegBin()
  if (resolved) extra.unshift(dirname(resolved))
  const parts = [...extra, ...base.split(delimiter).filter(Boolean)]
  const seen = new Set<string>()
  const out: string[] = []
  for (const p of parts) {
    if (!p || seen.has(p)) continue
    seen.add(p)
    out.push(p)
  }
  return out.join(delimiter)
}

export function ffmpegStatus(): FfmpegStatus {
  const path = resolveFfmpegBin()
  if (!path) return { ok: false, path: null, version: null, hint: ffmpegInstallHint() }
  return { ok: true, path, version: null }
}

/** Probe `-version` once (used by health). */
export function ffmpegStatusAsync(): Promise<FfmpegStatus> {
  const snap = ffmpegStatus()
  if (!snap.path) return Promise.resolve(snap)
  return new Promise((resolve) => {
    execFile(snap.path!, ['-version'], { timeout: 8000, windowsHide: true }, (err, stdout) => {
      if (err) {
        resolve({
          ok: false,
          path: snap.path,
          version: null,
          hint: `${ffmpegInstallHint()} (${err.message})`
        })
        return
      }
      const version = stdout.toString().split('\n')[0]?.trim() || 'available'
      resolve({ ok: true, path: snap.path, version })
    })
  })
}
