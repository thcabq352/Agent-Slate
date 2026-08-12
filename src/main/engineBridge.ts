// Bridge to slate-engine HTTP control server (Phase 5).
// Reads control.json written by `slate-engine serve`, or starts a debug binary if present.

import { spawn, type ChildProcess } from 'child_process'
import { existsSync, readFileSync } from 'fs'
import { homedir } from 'os'
import { join } from 'path'

export interface EngineDescriptor {
  port: number
  token: string
  pid?: number
}

export interface EngineHealth {
  engine?: boolean
  comfy?: { ok: boolean; url?: string }
  vision?: {
    ready?: boolean
    model?: string
    hint?: string
  }
  qualityGate?: {
    passThreshold?: number
    maxRetries?: number
    judgeModel?: string
  }
  dryRun?: boolean
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

function descriptorPath(): string {
  const base =
    process.platform === 'win32'
      ? process.env.APPDATA || join(homedir(), 'AppData', 'Roaming')
      : join(homedir(), '.config')
  return join(base, 'slate', 'control.json')
}

export function readDescriptor(): EngineDescriptor | null {
  try {
    const d = JSON.parse(readFileSync(descriptorPath(), 'utf8')) as EngineDescriptor
    if (d && d.port && d.token) return d
  } catch {
    /* not running */
  }
  return null
}

function engineBinaryCandidates(): string[] {
  const root = join(__dirname, '../..')
  const names =
    process.platform === 'win32'
      ? ['slate-engine.exe', 'slate-engine']
      : ['slate-engine']
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

/** Best-effort: start slate-engine serve if binary exists and not already up. */
export async function ensureEngine(): Promise<{ ok: boolean; message: string; descriptor: EngineDescriptor | null }> {
  const existing = readDescriptor()
  if (existing) {
    try {
      await invoke('slate_health', {})
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
    child = spawn(bin, ['serve'], {
      detached: false,
      stdio: 'ignore',
      windowsHide: true,
      env: { ...process.env }
    })
    child.unref?.()
  }

  for (let i = 0; i < 40; i++) {
    await new Promise((r) => setTimeout(r, 250))
    const d = readDescriptor()
    if (d) {
      try {
        await invoke('slate_health', {})
        return { ok: true, message: `started ${bin}`, descriptor: d }
      } catch {
        /* keep waiting */
      }
    }
  }
  return { ok: false, message: 'timed out waiting for slate-engine control.json', descriptor: readDescriptor() }
}

export async function invoke(tool: string, args: Record<string, unknown> = {}): Promise<unknown> {
  const desc = readDescriptor()
  if (!desc) {
    throw new Error("Slate engine isn't running — start `slate-engine serve` or call engineEnsure first.")
  }
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

export async function health(): Promise<EngineHealth> {
  return (await invoke('slate_health', {})) as EngineHealth
}

export async function status(): Promise<EngineJobStatus> {
  return (await invoke('slate_status', {})) as EngineJobStatus
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
