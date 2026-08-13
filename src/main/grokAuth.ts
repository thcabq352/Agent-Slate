// Resolve a Grok TTS bearer from xAI OAuth (grok login) without an API key.
// Never log tokens.

import { existsSync, readFileSync } from 'fs'
import { homedir } from 'os'
import { join } from 'path'
import {
  grokTtsApiKey,
  grokTtsStatus as statusFromFlags,
  parseGrokAuthJson,
  type GrokTtsAuth
} from '../shared/grokTts'

export function grokAuthFileCandidates(): string[] {
  const home = homedir()
  const grokcliHome = process.env.GROKCLI_HOME
  return [
    join(home, '.grok', 'auth.json'),
    grokcliHome ? join(grokcliHome, 'auth.json') : '',
    join(home, '.config', 'grokcli', 'auth.json'),
    process.env.APPDATA ? join(process.env.APPDATA, 'grokcli', 'auth.json') : '',
    join(home, '.grok-cli', 'auth.json')
  ].filter(Boolean)
}

function readAuthFile(path: string): unknown | null {
  try {
    if (!existsSync(path)) return null
    return JSON.parse(readFileSync(path, 'utf8')) as unknown
  } catch {
    return null
  }
}

export function resolveGrokTtsAuth(): GrokTtsAuth | null {
  for (const file of grokAuthFileCandidates()) {
    const parsed = parseGrokAuthJson(readAuthFile(file))
    if (parsed) {
      return { kind: 'oauth', token: parsed.token, cliTokenAuth: parsed.cliTokenAuth }
    }
  }
  const key = grokTtsApiKey(process.env)
  if (key) return { kind: 'api-key', token: key }
  return null
}

export function grokTtsReadyStatus(): { ready: boolean; hint: string } {
  const auth = resolveGrokTtsAuth()
  return statusFromFlags({
    oauth: auth?.kind === 'oauth',
    apiKey: auth?.kind === 'api-key'
  })
}

/** Official Grok Build session only (`~/.grok/auth.json` from `grok login`). */
export function grokBuildAuthPath(): string {
  return join(homedir(), '.grok', 'auth.json')
}

export function grokBuildOauthPresent(): boolean {
  return parseGrokAuthJson(readAuthFile(grokBuildAuthPath())) !== null
}
