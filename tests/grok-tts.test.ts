import { describe, it, expect } from 'vitest'
import {
  GROK_TTS_OAUTH_HINT,
  GROK_TTS_MAX_CHARS,
  grokTtsApiKey,
  grokTtsAuthHeaders,
  grokTtsErrorMessage,
  grokTtsRequestBody,
  grokTtsStatus,
  parseGrokAuthJson,
  pickGrokVoice,
  voFileSlug,
  clampTtsText
} from '../src/shared/grokTts'

describe('grok TTS helpers', () => {
  it('prefers xAI OAuth over an API key in status', () => {
    expect(grokTtsApiKey({})).toBeNull()
    expect(grokTtsApiKey({ XAI_API_KEY: '  xai-plain  ' })).toBe('xai-plain')
    expect(grokTtsStatus({}).ready).toBe(false)
    expect(grokTtsStatus({}).hint).toBe(GROK_TTS_OAUTH_HINT)
    expect(grokTtsStatus({ oauth: true }).hint).toMatch(/OAuth/)
    expect(grokTtsStatus({ apiKey: true }).ready).toBe(true)
    expect(grokTtsStatus({ XAI_API_KEY: 'k' }).ready).toBe(true)
  })

  it('parses official grok login and grokcli auth.json without exposing tokens in errors', () => {
    const official = parseGrokAuthJson({
      'https://accounts.x.ai/sign-in': { key: 'oauth-session-token', refresh_token: 'r' }
    })
    expect(official?.token).toBe('oauth-session-token')
    expect(official?.cliTokenAuth).toBe(true)

    const keyed = parseGrokAuthJson({
      'https://auth.x.ai::client': {
        key: 'cli-key-token',
        refresh_token: 'r',
        oidc_issuer: 'https://auth.x.ai'
      }
    })
    expect(keyed?.token).toBe('cli-key-token')

    const grokcli = parseGrokAuthJson({
      tokens: { access_token: 'grokcli-access', refresh_token: 'r', token_type: 'Bearer' }
    })
    expect(grokcli?.token).toBe('grokcli-access')
    expect(grokcli?.cliTokenAuth).toBe(false)

    expect(parseGrokAuthJson({ nope: true })).toBeNull()
  })

  it('sends a Bearer header and optional Grok CLI token-auth', () => {
    const h = grokTtsAuthHeaders({ kind: 'oauth', token: 'sess', cliTokenAuth: true })
    expect(h.Authorization).toBe('Bearer sess')
    expect(h['X-XAI-Token-Auth']).toBe('xai-grok-cli')
    const plain = grokTtsAuthHeaders({ kind: 'oauth', token: 'sess' })
    expect(plain['X-XAI-Token-Auth']).toBeUndefined()
  })

  it('picks built-in voices from a casting sheet', () => {
    expect(pickGrokVoice({ ageGender: 'woman, 40s' })).toBe('eve')
    expect(pickGrokVoice({ ageGender: 'male', pitch: 'low baritone' })).toBe('rex')
    expect(pickGrokVoice({ ageGender: 'young boy', timbre: 'bright' })).toBe('sal')
    expect(pickGrokVoice({ ageGender: 'man, 50' })).toBe('leo')
    expect(pickGrokVoice({ name: 'Narrator' })).toBe('eve')
  })

  it('builds a Grok TTS body with film-ish mp3 settings', () => {
    const body = grokTtsRequestBody({
      text: '  Hold the line.  ',
      voiceId: 'Ara',
      language: 'en'
    })
    expect(body.text).toBe('Hold the line.')
    expect(body.voice_id).toBe('Ara')
    expect(body.language).toBe('en')
    expect(body.text_normalization).toBe(true)
    expect(body.output_format).toEqual({ codec: 'mp3', sample_rate: 44100, bit_rate: 192000 })
  })

  it('refuses empty text and clamps the 15k character cap', () => {
    expect(() => grokTtsRequestBody({ text: '   ', voiceId: 'eve' })).toThrow(/Nothing to speak/)
    const long = 'x'.repeat(GROK_TTS_MAX_CHARS + 40)
    expect(clampTtsText(long)).toHaveLength(GROK_TTS_MAX_CHARS)
  })

  it('surfaces 401 without asking for an API key', () => {
    const msg = grokTtsErrorMessage(401, '{"error":{"message":"Incorrect API key"}}')
    expect(msg).toContain('rejected the session')
    expect(msg).toContain('grok login')
    expect(msg).not.toContain('XAI_API_KEY')
    expect(msg).not.toContain('sk-')
  })

  it('slugs vo filenames', () => {
    expect(voFileSlug('Kaia VO', 'voice-abc123xyz')).toBe('kaia-vo-123xyz')
    expect(voFileSlug('!!!', 'id')).toBe('voice-id')
  })
})
