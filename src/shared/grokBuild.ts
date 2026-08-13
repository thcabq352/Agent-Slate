// Grok Build CLI (official `grok` from `grok login`) — higher priority than
// Cursor OAuth for grok-4.5 / grok-4.6 brains. Composer stays on cursor-agent.
// Never log tokens.

import type { BrainBackend } from './types'

export const GROK_BUILD_LOGIN_HINT =
  'Run grok login (Grok Build OAuth). If Grok Build is not installed, Agent-Slate falls back to cursor-agent login. Windows: irm https://x.ai/cli/install.ps1 | iex  · macOS/Linux: curl -fsSL https://x.ai/cli/install.sh | bash'

const AUTH_ERR =
  /authenticat|oauth|401|logged? ?in|revoked|not signed|unauthenticated|grok login|unauthorized/i

export function isGrokBuildBrain(backend: BrainBackend): boolean {
  return backend === 'grok-4.5' || backend === 'grok-4.6'
}

/** Model id passed to `grok -m`. Cursor fallback still uses cursor-grok-* slugs. */
export function grokBuildCliModel(backend: BrainBackend): string {
  if (backend === 'grok-4.5') return 'grok-4.5'
  if (backend === 'grok-4.6') return 'grok-4.6'
  return 'grok-build'
}

export function grokBuildPromptFileName(): string {
  return 'slate-brain-prompt.txt'
}

export function grokBuildHeadlessPrompt(): string {
  return `Read ${grokBuildPromptFileName()} in this directory and follow it exactly. Reply with only the answer — no preamble.`
}

function authError(detail: string): Error {
  return new Error(
    `Grok Build is not signed in. Open a terminal, run: grok login  — approve xAI OAuth, then retry. (${detail})`
  )
}

function asObj(v: unknown): Record<string, unknown> | null {
  return v && typeof v === 'object' && !Array.isArray(v) ? (v as Record<string, unknown>) : null
}

/** Parse `grok --output-format json` stdout (`.text`). Surface login errors. */
export function parseGrokBuildOutput(raw: string): string {
  const trimmed = raw.trim()
  try {
    const o = asObj(JSON.parse(trimmed))
    if (o) {
      const errStr = typeof o.error === 'string' ? o.error : ''
      const resultStr = typeof o.result === 'string' ? o.result : ''
      const fail = o.is_error === true || Boolean(errStr)
      const failMsg = errStr || resultStr || 'Grok Build returned an error.'
      if (fail) {
        if (AUTH_ERR.test(failMsg)) throw authError(failMsg)
        throw new Error(failMsg)
      }
      if (typeof o.text === 'string') return o.text
      if (resultStr) return resultStr
      if (typeof o.message === 'string') return o.message
    }
  } catch (e) {
    if (e instanceof Error && e.message.includes('grok login')) throw e
    if (e instanceof Error && !(e instanceof SyntaxError)) throw e
  }
  if (AUTH_ERR.test(trimmed)) throw authError(trimmed)
  return trimmed
}
