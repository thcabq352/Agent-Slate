// Grok Text-to-Speech (xAI). Built-in voices + request shaping.
// Audio bytes come from POST https://api.x.ai/v1/tts using xAI OAuth (grok login)
// or an optional API key fallback.

export const GROK_TTS_URL = 'https://api.x.ai/v1/tts'
export const GROK_TTS_MAX_CHARS = 15_000
export const GROK_TTS_OAUTH_HINT =
  'Run grok login (xAI OAuth). That same Grok Build session is preferred for Grok 4.5/4.6 brains; Composer still uses cursor-agent login.'
export const GROK_TTS_KEY_HINT = GROK_TTS_OAUTH_HINT

export const GROK_TTS_VOICES = [
  { id: 'eve', label: 'Eve', hint: 'Clear default' },
  { id: 'ara', label: 'Ara', hint: 'Warm' },
  { id: 'leo', label: 'Leo', hint: 'Steady' },
  { id: 'rex', label: 'Rex', hint: 'Low, dry' },
  { id: 'sal', label: 'Sal', hint: 'Bright' }
] as const

export type GrokTtsVoiceId = (typeof GROK_TTS_VOICES)[number]['id']

export interface GrokTtsVoiceHint {
  ageGender?: string
  pitch?: string
  timbre?: string
  name?: string
}

export type GrokTtsEnv = { SLATE_XAI_API_KEY?: string; XAI_API_KEY?: string }

export type GrokTtsAuthKind = 'oauth' | 'api-key'

export interface GrokTtsAuth {
  kind: GrokTtsAuthKind
  /** Never log this. */
  token: string
  /** Official Grok CLI sessions need this on some xAI endpoints. */
  cliTokenAuth?: boolean
}

export function grokTtsApiKey(env: GrokTtsEnv): string | null {
  const k = (env.SLATE_XAI_API_KEY || env.XAI_API_KEY || '').trim()
  return k || null
}

export function grokTtsStatus(opts: { oauth?: boolean; apiKey?: boolean } | GrokTtsEnv): {
  ready: boolean
  hint: string
} {
  const oauth = 'oauth' in opts ? Boolean(opts.oauth) : false
  const apiKey =
    'apiKey' in opts ? Boolean(opts.apiKey) : Boolean(grokTtsApiKey(opts as GrokTtsEnv))
  if (oauth) return { ready: true, hint: 'Grok TTS ready (xAI OAuth).' }
  if (apiKey) return { ready: true, hint: 'Grok TTS ready (API key).' }
  return { ready: false, hint: GROK_TTS_OAUTH_HINT }
}

/** Pull an access token from grokcli or official `~/.grok/auth.json` shapes. Never returns secrets in errors. */
export function parseGrokAuthJson(data: unknown): { token: string; cliTokenAuth: boolean } | null {
  if (!data || typeof data !== 'object') return null
  const o = data as Record<string, unknown>

  const grokcli = o.tokens
  if (grokcli && typeof grokcli === 'object') {
    const at = String((grokcli as Record<string, unknown>).access_token || '').trim()
    if (at) return { token: at, cliTokenAuth: false }
  }

  const flat = String(o.access_token || '').trim()
  if (flat && typeof o.tokens !== 'object') return { token: flat, cliTokenAuth: false }

  const signIn = o['https://accounts.x.ai/sign-in']
  const fromSignIn = entryAccess(signIn)
  if (fromSignIn) return { token: fromSignIn, cliTokenAuth: true }

  for (const v of Object.values(o)) {
    if (!v || typeof v !== 'object' || Array.isArray(v)) continue
    const e = v as Record<string, unknown>
    const issuer = String(e.oidc_issuer || e.issuer || '')
    const key = entryAccess(e)
    if (!key) continue
    if (issuer.includes('x.ai') || e.refresh_token) return { token: key, cliTokenAuth: true }
  }
  return null
}

function entryAccess(entry: unknown): string | null {
  if (!entry || typeof entry !== 'object') return null
  const e = entry as Record<string, unknown>
  const key = String(e.key || e.access_token || '').trim()
  return key || null
}

export function grokTtsAuthHeaders(auth: GrokTtsAuth, extra?: boolean): Record<string, string> {
  const headers: Record<string, string> = {
    Authorization: `Bearer ${auth.token}`,
    'Content-Type': 'application/json',
    Accept: 'audio/*, application/json'
  }
  if (auth.cliTokenAuth || extra) headers['X-XAI-Token-Auth'] = 'xai-grok-cli'
  return headers
}

/** Pick a built-in Grok voice from a casting sheet. Unknown / custom ids pass through elsewhere. */
export function pickGrokVoice(sheet: GrokTtsVoiceHint): GrokTtsVoiceId {
  const blob = [sheet.ageGender, sheet.pitch, sheet.timbre, sheet.name].filter(Boolean).join(' ').toLowerCase()
  if (/\b(female|woman|girl|she|her|alto|soprano)\b/.test(blob)) return 'eve'
  if (/\b(low|baritone|bass|gravel|gravelly|deep)\b/.test(blob)) return 'rex'
  if (/\b(bright|young|boy|teen)\b/.test(blob)) return 'sal'
  if (/\b(male|man|him|his|tenor)\b/.test(blob)) return 'leo'
  return 'eve'
}

export function clampTtsText(text: string): string {
  const t = text.trim()
  if (t.length <= GROK_TTS_MAX_CHARS) return t
  return t.slice(0, GROK_TTS_MAX_CHARS)
}

export function voFileSlug(name: string, sheetId: string): string {
  const slug = name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 40)
  const tail = sheetId.replace(/[^a-z0-9]/gi, '').slice(-6) || 'voice'
  return `${slug || 'voice'}-${tail}`
}

export interface GrokTtsRequestBody {
  text: string
  voice_id: string
  language: string
  text_normalization: boolean
  output_format: { codec: 'mp3'; sample_rate: number; bit_rate: number }
}

export function grokTtsRequestBody(opts: {
  text: string
  voiceId: string
  language?: string
}): GrokTtsRequestBody {
  const text = clampTtsText(opts.text)
  if (!text) throw new Error('Nothing to speak — write a sample line or VO text first.')
  const voice_id = (opts.voiceId || 'eve').trim() || 'eve'
  const language = (opts.language || 'en').trim() || 'en'
  return {
    text,
    voice_id,
    language,
    text_normalization: true,
    output_format: { codec: 'mp3', sample_rate: 44100, bit_rate: 192000 }
  }
}

/** Surface xAI JSON errors without dumping the request or tokens. */
export function grokTtsErrorMessage(status: number, body: string): string {
  const trimmed = body.trim()
  let detail = trimmed.slice(0, 280)
  try {
    const j = JSON.parse(trimmed) as {
      error?: string | { message?: string }
      message?: string
    }
    if (typeof j.error === 'string') detail = j.error
    else if (j.error && typeof j.error === 'object' && j.error.message) detail = j.error.message
    else if (typeof j.message === 'string') detail = j.message
  } catch {
    /* keep slice */
  }
  if (status === 401 || status === 403) {
    return `xAI rejected the session (${status}). ${GROK_TTS_OAUTH_HINT}`
  }
  return `Grok TTS failed (${status}): ${detail}`
}
