// Brain — runs the user's own local agent CLIs (Grok Build, Cursor CLI, Codex) in print mode,
// or any OpenAI-compatible local model server (Ollama, LM Studio, vLLM, llama.cpp…).
// Grok 4.5 / 4.6 prefer `grok login` (Grok Build OAuth) over `cursor-agent login`.
// Composer stays on Cursor. No API keys are stored or used.

import { execFile, spawn, ChildProcess } from 'child_process'
import { existsSync, readFileSync, readdirSync, rmSync, mkdirSync, statSync, writeFileSync } from 'fs'
import { join, extname, delimiter } from 'path'
import { homedir, tmpdir } from 'os'
import type { BrainBackend, BrainRequest, BrainResult, BrainStatus, BrainTier, LocalModelInfo } from '../shared/types'
import { isCursorBrain, isGrokBrain, normalizeBrain } from '../shared/types'
import {
  grokBuildCliModel,
  grokBuildHeadlessPrompt,
  grokBuildPromptFileName,
  parseGrokBuildOutput
} from '../shared/grokBuild'
import { grokBuildOauthPresent } from './grokAuth'

// Electron apps launched from Finder/Dock inherit a minimal PATH that misses
// Homebrew and user bins — resolve the CLIs explicitly and augment PATH.
const CLI_DIRS = [
  join(homedir(), '.grok', 'bin'),
  '/opt/homebrew/bin',
  '/usr/local/bin',
  '/snap/bin',
  join(homedir(), '.local', 'bin'),
  join(homedir(), 'bin'),
  join(homedir(), '.npm-global', 'bin'),
  '/usr/bin',
  ...(process.env.APPDATA ? [join(process.env.APPDATA, 'npm')] : []),
  ...(process.env.LOCALAPPDATA
    ? [
        join(process.env.LOCALAPPDATA, 'cursor-agent'),
        join(process.env.LOCALAPPDATA, 'Programs', 'cursor', 'resources', 'app', 'bin'),
        join(process.env.LOCALAPPDATA, 'Microsoft', 'WinGet', 'Links')
      ]
    : []),
  join(homedir(), 'scoop', 'shims'),
  'C:\\ffmpeg\\bin',
  join(process.env.ProgramFiles || 'C:\\Program Files', 'ffmpeg', 'bin'),
  'C:\\ProgramData\\chocolatey\\bin',
  join(process.env.ProgramFiles || 'C:\\Program Files', 'Git', 'usr', 'bin')
]

function codexBundledCandidates(): string[] {
  const local = process.env.LOCALAPPDATA
  const pf = process.env.ProgramFiles || 'C:\\Program Files'
  return [
    '/Applications/ChatGPT.app/Contents/Resources/codex',
    ...(local
      ? [
          join(local, 'Programs', 'ChatGPT', 'resources', 'codex.exe'),
          join(local, 'Programs', 'ChatGPT', 'resources', 'codex'),
          join(local, 'Programs', 'ChatGPT', 'resources', 'app', 'codex.exe'),
          join(local, 'Programs', 'chatgpt', 'resources', 'codex.exe')
        ]
      : []),
    join(pf, 'ChatGPT', 'resources', 'codex.exe'),
    join(pf, 'ChatGPT', 'resources', 'codex')
  ]
}

/** Windows: only .exe/.cmd/.bat files. Extensionless Git-Bash shims spawn EINVAL. */
function isSpawnableCli(p: string): boolean {
  try {
    if (!statSync(p).isFile()) return false
  } catch {
    return false
  }
  if (process.platform !== 'win32') return true
  return /\.(exe|cmd|bat)$/i.test(p)
}

function resolveCodexBundled(): string | null {
  return codexBundledCandidates().find((p) => isSpawnableCli(p)) ?? null
}

function resolveCli(name: string): string {
  if (name === 'codex') {
    const bundled = resolveCodexBundled()
    if (bundled) return bundled
  }
  // Prefer .exe, then .cmd — never the extensionless bash shim on Windows.
  const names = process.platform === 'win32' ? [`${name}.exe`, `${name}.cmd`, `${name}.bat`] : [name]
  const dirs = [...CLI_DIRS, ...(process.env.PATH ?? '').split(delimiter)]
  for (const dir of dirs) {
    if (!dir) continue
    for (const n of names) {
      const p = join(dir, n)
      if (isSpawnableCli(p)) return p
    }
  }
  return process.platform === 'win32' ? `${name}.cmd` : name
}

/** Launch Cursor via bundled node.exe + index.js so we never spawn a .cmd/.sh shim. */
function resolveCursorLaunch(): { file: string; prefixArgs: string[] } {
  const local = process.env.LOCALAPPDATA
  if (local) {
    const versionsRoot = join(local, 'cursor-agent', 'versions')
    try {
      const versions = readdirSync(versionsRoot)
        .filter((n) => /^\d{4}\.\d{1,2}\.\d{1,2}/.test(n))
        .sort()
        .reverse()
      for (const ver of versions) {
        const dir = join(versionsRoot, ver)
        const node = join(dir, 'node.exe')
        const index = join(dir, 'index.js')
        if (isSpawnableCli(node) && existsSync(index)) {
          return { file: node, prefixArgs: [index] }
        }
      }
    } catch {
      /* no versions dir */
    }
  }
  return { file: resolveCli('cursor-agent'), prefixArgs: [] }
}

/** Official Grok Build binary. Never spawn the colliding `agent` CLI. */
function resolveGrokBin(): string | null {
  const names = process.platform === 'win32' ? ['grok.exe'] : ['grok']
  const dirs = [join(homedir(), '.grok', 'bin'), ...CLI_DIRS, ...(process.env.PATH ?? '').split(delimiter)]
  const seen = new Set<string>()
  for (const dir of dirs) {
    if (!dir || seen.has(dir)) continue
    seen.add(dir)
    for (const n of names) {
      const p = join(dir, n)
      if (isSpawnableCli(p)) return p
    }
  }
  return null
}

function grokBuildReady(): boolean {
  return resolveGrokBin() !== null && grokBuildOauthPresent()
}

function isWinBatch(file: string): boolean {
  return process.platform === 'win32' && /\.(cmd|bat)$/i.test(file)
}

function brainEnv(): NodeJS.ProcessEnv {
  const extra = CLI_DIRS.filter(Boolean).join(delimiter)
  const path = process.env.PATH ?? ''
  return { ...process.env, PATH: path ? `${extra}${delimiter}${path}` : extra }
}

function cursorCliModel(backend: BrainBackend, tier: BrainTier): string {
  if (backend === 'grok-4.5') {
    return tier === 'fast' ? 'cursor-grok-4.5-high-fast' : 'cursor-grok-4.5-high'
  }
  if (backend === 'grok-4.6') {
    if (tier === 'fast') return 'cursor-grok-4.6-xhigh-fast'
    if (tier === 'top') return 'cursor-grok-4.6-xhigh'
    return 'cursor-grok-4.6-high'
  }
  return CURSOR_COMPOSER_TIER[tier]
}

const CURSOR_COMPOSER_TIER: Record<BrainTier, string> = {
  fast: 'composer-2.5-fast',
  standard: 'composer-2.5',
  top: 'composer-2.5'
}

const running = new Map<string, ChildProcess>()
const runningLocal = new Map<string, AbortController>()

// ---- Local model backend (OpenAI-compatible localhost server) ----
// One adapter covers every mainstream local runtime — they all expose the same
// /v1/chat/completions protocol on a localhost port.
const LOCAL_CANDIDATES = [
  'http://127.0.0.1:11434/v1', // Ollama IPv4
  'http://localhost:11434/v1',
  'http://127.0.0.1:1234/v1',
  'http://localhost:1234/v1',
  'http://127.0.0.1:8000/v1',
  'http://localhost:8000/v1',
  'http://127.0.0.1:8080/v1',
  'http://localhost:8080/v1'
]

function normalizeEndpoint(url: string): string {
  let u = url.trim().replace(/\/+$/, '')
  if (!/^https?:\/\//.test(u)) u = `http://${u}`
  if (!/\/v1$/.test(u)) u = `${u}/v1`
  return u
}

async function probeLocal(endpoint: string): Promise<LocalModelInfo[] | null> {
  try {
    const ctrl = new AbortController()
    const t = setTimeout(() => ctrl.abort(), 4000)
    const res = await fetch(`${endpoint}/models`, {
      headers: { Authorization: 'Bearer slate' },
      signal: ctrl.signal
    })
    clearTimeout(t)
    if (!res.ok) return null
    const body = (await res.json()) as { data?: Array<{ id?: string }> }
    const models = (body.data ?? []).filter((m) => m.id).map((m) => ({ id: String(m.id) }))
    return models
  } catch {
    return null
  }
}

/** Find a live local server: the user's configured endpoint first, then common ports. */
export async function detectLocal(
  preferred?: string
): Promise<{ endpoint: string | null; models: LocalModelInfo[] }> {
  const candidates = preferred
    ? [normalizeEndpoint(preferred)]
    : LOCAL_CANDIDATES
  for (const ep of candidates) {
    const models = await probeLocal(ep)
    if (models !== null) return { endpoint: ep, models }
  }
  return { endpoint: null, models: [] }
}

const IMAGE_MIME: Record<string, string> = {
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.webp': 'image/webp',
  '.gif': 'image/gif'
}

type ChatContent = string | Array<{ type: string; text?: string; image_url?: { url: string } }>

function localMessages(req: BrainRequest): Array<{ role: string; content: ChatContent }> {
  let user: ChatContent = req.prompt
  if (req.images && req.images.length > 0) {
    const parts: Array<{ type: string; text?: string; image_url?: { url: string } }> = [
      { type: 'text', text: req.prompt }
    ]
    for (const img of req.images) {
      const mime = IMAGE_MIME[extname(img).toLowerCase()]
      if (!mime || !existsSync(img)) continue
      const b64 = readFileSync(img).toString('base64')
      parts.push({ type: 'image_url', image_url: { url: `data:${mime};base64,${b64}` } })
    }
    user = parts
  }
  return [
    { role: 'system', content: req.system },
    { role: 'user', content: user }
  ]
}

async function runLocalOnce(
  req: BrainRequest,
  endpoint: string,
  model: string,
  extraNudge?: string
): Promise<string> {
  const messages = localMessages(req)
  if (extraNudge) messages.push({ role: 'user', content: extraNudge })
  const ctrl = new AbortController()
  runningLocal.set(req.id, ctrl)
  try {
    const res = await fetch(`${endpoint}/chat/completions`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: 'Bearer slate' },
      body: JSON.stringify({ model, messages, stream: false }),
      signal: ctrl.signal
    })
    if (!res.ok) {
      const detail = (await res.text().catch(() => '')).slice(0, 300)
      throw new Error(`Local model server responded ${res.status} at ${endpoint}. ${detail}`)
    }
    const body = (await res.json()) as {
      choices?: Array<{ message?: { content?: string } }>
      error?: { message?: string }
    }
    if (body.error?.message) throw new Error(body.error.message)
    const text = body.choices?.[0]?.message?.content
    if (typeof text !== 'string' || !text.trim()) {
      throw new Error('Local model returned an empty response.')
    }
    return text.trim()
  } finally {
    runningLocal.delete(req.id)
  }
}

async function runLocal(req: BrainRequest, started: number): Promise<BrainResult> {
  const { endpoint, models } = await detectLocal(req.localEndpoint)
  if (!endpoint) {
    return {
      id: req.id,
      ok: false,
      text: '',
      error:
        'No local model server found. Start Ollama, LM Studio, vLLM, or llama.cpp (or set a custom endpoint in Project Settings → Brain), then retry.',
      elapsedMs: Date.now() - started
    }
  }
  const model = req.localModel?.trim() || models[0]?.id
  if (!model) {
    return {
      id: req.id,
      ok: false,
      text: '',
      error: `Local server at ${endpoint} has no models loaded. Pull or load a model, then retry.`,
      elapsedMs: Date.now() - started
    }
  }
  try {
    let text = await runLocalOnce(req, endpoint, model)
    let json: unknown
    if (req.expectJson) {
      try {
        json = extractJson(text)
      } catch {
        text = await runLocalOnce(
          req,
          endpoint,
          model,
          'IMPORTANT: Respond with ONLY the requested JSON. No prose, no code fences.'
        )
        json = extractJson(text)
      }
    }
    return { id: req.id, ok: true, text, json, elapsedMs: Date.now() - started }
  } catch (e) {
    return {
      id: req.id,
      ok: false,
      text: '',
      error: e instanceof Error ? e.message : String(e),
      elapsedMs: Date.now() - started
    }
  }
}

function which(cmd: string, args: string[]): Promise<string | null> {
  return new Promise((resolve) => {
    const launch =
      cmd === 'cursor-agent'
        ? resolveCursorLaunch()
        : cmd === 'grok'
          ? { file: resolveGrokBin() ?? '', prefixArgs: [] as string[] }
          : { file: resolveCli(cmd), prefixArgs: [] as string[] }
    if (!launch.file) {
      resolve(null)
      return
    }
    try {
      execFile(
        launch.file,
        [...launch.prefixArgs, ...args],
        { timeout: 15000, env: brainEnv(), windowsHide: true, shell: isWinBatch(launch.file) },
        (err, stdout) => {
          if (err) resolve(null)
          else resolve(stdout.trim().split('\n')[0] || 'available')
        }
      )
    } catch {
      resolve(null)
    }
  })
}

export async function brainStatus(localEndpoint?: string): Promise<BrainStatus> {
  const [cursorV, grokV, grokOauth, codexV, local] = await Promise.all([
    which('cursor-agent', ['--version']),
    which('grok', ['--version']),
    Promise.resolve(grokBuildOauthPresent()),
    which('codex', ['--version']),
    detectLocal(localEndpoint)
  ])
  return {
    cursor: { available: cursorV !== null, version: cursorV },
    grok: { available: grokV !== null && grokOauth, version: grokV },
    codex: { available: codexV !== null, version: codexV },
    local: {
      available: local.endpoint !== null,
      version: local.endpoint
        ? `${local.models.length} model(s) @ ${local.endpoint.replace(/^https?:\/\//, '')}`
        : null,
      endpoint: local.endpoint
    }
  }
}

/** Extract the first balanced JSON object or array from text. */
export function extractJson(text: string): unknown {
  const cleaned = text.replace(/```(?:json)?/g, '').trim()
  const starts: Array<[string, string]> = [
    ['{', '}'],
    ['[', ']']
  ]
  for (const [open, close] of starts) {
    const i = cleaned.indexOf(open)
    if (i === -1) continue
    let depth = 0
    let inStr = false
    let esc = false
    for (let j = i; j < cleaned.length; j++) {
      const ch = cleaned[j]
      if (esc) {
        esc = false
        continue
      }
      if (ch === '\\') {
        esc = true
        continue
      }
      if (ch === '"') inStr = !inStr
      if (inStr) continue
      if (ch === open) depth++
      else if (ch === close) {
        depth--
        if (depth === 0) {
          try {
            return JSON.parse(cleaned.slice(i, j + 1))
          } catch {
            break
          }
        }
      }
    }
  }
  throw new Error('No valid JSON found in response')
}

interface CliCall {
  cmd: string
  args: string[]
  input?: string
  cwd?: string
}

function cursorWorkspace(): string {
  const dir = join(tmpdir(), 'slate-cursor-brain')
  mkdirSync(dir, { recursive: true })
  return dir
}

function grokWorkspace(): string {
  const dir = join(tmpdir(), 'slate-grok-brain')
  mkdirSync(dir, { recursive: true })
  return dir
}

function buildGrokBuildCall(req: BrainRequest, backend: BrainBackend): CliCall {
  const workspace = grokWorkspace()
  const grok = resolveGrokBin()
  if (!grok) {
    throw new Error('Grok Build CLI not found. Install it, run grok login, then retry.')
  }
  let prompt = `${req.system}\n\n---\n\n${req.prompt}`
  if (req.images && req.images.length > 0) {
    prompt +=
      '\n\nReference media frames to view (read each file before answering):\n' +
      req.images.map((p) => `- ${p}`).join('\n')
  }
  writeFileSync(join(workspace, grokBuildPromptFileName()), prompt, 'utf8')
  return {
    cmd: grok,
    args: [
      '-p',
      grokBuildHeadlessPrompt(),
      '--output-format',
      'json',
      '--always-approve',
      '--cwd',
      workspace,
      '-m',
      grokBuildCliModel(backend),
      '--tools',
      'read_file',
      '--max-turns',
      '8'
    ],
    cwd: workspace
  }
}

function buildCursorCall(req: BrainRequest, backend: BrainBackend): CliCall {
  const workspace = cursorWorkspace()
  const launch = resolveCursorLaunch()
  const args = [
    ...launch.prefixArgs,
    '-p',
    '--output-format',
    'json',
    '--mode',
    'ask',
    '--trust',
    '--workspace',
    workspace,
    '--model',
    cursorCliModel(backend, req.tier)
  ]
  let prompt = `${req.system}\n\n---\n\n${req.prompt}`
  if (req.images && req.images.length > 0) {
    prompt +=
      '\n\nReference media frames to view (read each file before answering):\n' +
      req.images.map((p) => `- ${p}`).join('\n')
  }
  return { cmd: launch.file, args, input: prompt, cwd: workspace }
}

function buildCodexCall(req: BrainRequest, lastMessageFile: string): CliCall {
  // codex exec runs a one-shot task; the clean final answer lands in
  // --output-last-message (stdout interleaves streaming/progress noise).
  const args = ['exec', '--skip-git-repo-check', '--output-last-message', lastMessageFile]
  if (req.images && req.images.length > 0) {
    for (const img of req.images) args.push('-i', img)
  }
  args.push('-')
  const prompt = `${req.system}\n\n---\n\n${req.prompt}`
  return { cmd: 'codex', args, input: prompt }
}

function parseCursorOutput(raw: string): string {
  const trimmed = raw.trim()
  try {
    const parsed = JSON.parse(trimmed)
    if (parsed?.is_error) {
      const msg: string =
        typeof parsed.result === 'string'
          ? parsed.result
          : typeof parsed.error === 'string'
            ? parsed.error
            : 'Cursor CLI returned an error.'
      if (/authenticat|oauth|401|logged? ?in|revoked|not signed|unauthenticated/i.test(msg)) {
        throw new Error(
          `Cursor CLI is not signed in. Open a terminal, run: cursor-agent login  — approve in the browser (Cursor OAuth), then retry. (${msg})`
        )
      }
      throw new Error(msg)
    }
    if (typeof parsed?.result === 'string') return parsed.result
  } catch (e) {
    if (e instanceof Error && e.message.includes('cursor-agent login')) throw e
    if (e instanceof Error && !(e instanceof SyntaxError)) throw e
    if (/authenticat|oauth|401|not signed|unauthenticated|cursor-agent login/i.test(trimmed)) {
      throw new Error(
        `Cursor CLI is not signed in. Open a terminal, run: cursor-agent login  — approve in the browser (Cursor OAuth), then retry. (${trimmed})`
      )
    }
  }
  return trimmed
}

export async function brainRun(req: BrainRequest, backend: BrainBackend): Promise<BrainResult> {
  const started = Date.now()

  // Demo mode (SLATE_BRAIN_MOCK=<dir>): serve canned responses keyed by task
  // prefix after a short realistic delay. Used by the capture rig only.
  if (process.env.SLATE_BRAIN_MOCK) {
    const dir = process.env.SLATE_BRAIN_MOCK
    const key = ['first-ad', 'reference-analysis', 'score-compile', 'voice-compile', 'compile', 'directors-note']
      .find((k) => req.task.startsWith(k))
    const file = join(dir, `${key ?? 'default'}.json`)
    if (existsSync(file)) {
      await new Promise((r) => setTimeout(r, 1200))
      const canned = JSON.parse((await import('fs')).readFileSync(file, 'utf8'))
      return {
        id: req.id,
        ok: true,
        text: typeof canned === 'string' ? canned : JSON.stringify(canned),
        json: typeof canned === 'string' ? undefined : canned,
        elapsedMs: Date.now() - started
      }
    }
  }
  if (backend === 'local') return runLocal(req, started)
  backend = normalizeBrain(backend)

  const lastMessageFile = join(tmpdir(), `slate-codex-${req.id}.txt`)
  const useGrokBuild = isGrokBrain(backend) && grokBuildReady()
  const cursor = isCursorBrain(backend) && !useGrokBuild
  const call = useGrokBuild
    ? buildGrokBuildCall(req, backend)
    : cursor
      ? buildCursorCall(req, backend)
      : buildCodexCall(req, lastMessageFile)

  const codexResult = (rawStdout: string): string => {
    try {
      const msg = readFileSync(lastMessageFile, 'utf8').trim()
      rmSync(lastMessageFile, { force: true })
      if (msg) return msg
    } catch {
      /* fall back to stdout */
    }
    return rawStdout.trim()
  }

  const runOnce = (extraNudge?: string): Promise<string> =>
    new Promise((resolve, reject) => {
      const file = cursor || useGrokBuild ? call.cmd : resolveCli(call.cmd)
      if (useGrokBuild && extraNudge && call.cwd) {
        try {
          const promptFile = join(call.cwd, grokBuildPromptFileName())
          const prev = readFileSync(promptFile, 'utf8')
          writeFileSync(promptFile, `${prev}\n\n${extraNudge}`, 'utf8')
        } catch {
          /* prompt file already written */
        }
      }
      let child: ChildProcess
      try {
        child = spawn(file, call.args, {
          env: cursor ? { ...brainEnv(), CURSOR_INVOKED_AS: 'cursor-agent' } : brainEnv(),
          stdio: ['pipe', 'pipe', 'pipe'],
          cwd: call.cwd,
          windowsHide: true,
          shell: isWinBatch(file)
        })
      } catch (e) {
        const detail = e instanceof Error ? e.message : String(e)
        reject(new Error(`Could not launch ${file}: ${detail}`))
        return
      }
      running.set(req.id, child)
      let out = ''
      let errOut = ''
      child.stdout?.on('data', (d) => (out += d))
      child.stderr?.on('data', (d) => (errOut += d))
      child.on('error', (e) => {
        running.delete(req.id)
        reject(new Error(`Could not launch ${file}: ${e.message}`))
      })
      child.on('close', (code) => {
        running.delete(req.id)
        if (code !== 0 && !out.trim()) {
          reject(new Error(errOut.trim() || `${file} exited with code ${code}`))
        } else {
          try {
            resolve(
              useGrokBuild
                ? parseGrokBuildOutput(out)
                : cursor
                  ? parseCursorOutput(out)
                  : codexResult(out)
            )
          } catch (e) {
            reject(e instanceof Error ? e : new Error(String(e)))
          }
        }
      })
      const input = extraNudge ? `${call.input}\n\n${extraNudge}` : call.input
      child.stdin?.write(input ?? '')
      child.stdin?.end()
    })

  try {
    let text = await runOnce()
    let json: unknown
    if (req.expectJson) {
      try {
        json = extractJson(text)
      } catch {
        // One retry with an explicit nudge — models occasionally wrap JSON in prose.
        text = await runOnce('IMPORTANT: Respond with ONLY the requested JSON. No prose, no code fences.')
        json = extractJson(text)
      }
    }
    return { id: req.id, ok: true, text, json, elapsedMs: Date.now() - started }
  } catch (e) {
    return {
      id: req.id,
      ok: false,
      text: '',
      error: e instanceof Error ? e.message : String(e),
      elapsedMs: Date.now() - started
    }
  }
}

export function brainCancel(id: string): void {
  const child = running.get(id)
  if (child) {
    child.kill('SIGTERM')
    running.delete(id)
  }
  const ctrl = runningLocal.get(id)
  if (ctrl) {
    ctrl.abort()
    runningLocal.delete(id)
  }
}
