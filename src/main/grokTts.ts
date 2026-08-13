// Grok TTS — write an MP3 into the project `vo/` folder.

import { promises as fs } from 'fs'
import { join } from 'path'
import { pathToFileURL } from 'url'
import { projectDir } from './projects'
import { grokTtsReadyStatus, resolveGrokTtsAuth } from './grokAuth'
import type { GrokVoRenderRequest, GrokVoRenderResult } from '../shared/types'
import {
  GROK_TTS_URL,
  GROK_TTS_OAUTH_HINT,
  grokTtsAuthHeaders,
  grokTtsErrorMessage,
  grokTtsRequestBody,
  voFileSlug
} from '../shared/grokTts'

export function grokTtsStatus(): { ready: boolean; hint: string } {
  return grokTtsReadyStatus()
}

function decodeTtsAudio(buf: Buffer, contentType: string): Buffer {
  const ct = contentType.toLowerCase()
  if (ct.includes('application/json')) {
    const j = JSON.parse(buf.toString('utf8')) as { audio?: string; data?: string }
    const b64 = j.audio || j.data
    if (!b64 || typeof b64 !== 'string') throw new Error('Grok TTS JSON response had no audio.')
    return Buffer.from(b64, 'base64')
  }
  return buf
}

export async function renderGrokVo(req: GrokVoRenderRequest): Promise<GrokVoRenderResult> {
  const auth = resolveGrokTtsAuth()
  if (!auth) throw new Error(GROK_TTS_OAUTH_HINT)

  const body = grokTtsRequestBody({
    text: req.text,
    voiceId: req.voiceId,
    language: req.language
  })
  const payload = JSON.stringify(body)

  let res = await fetch(GROK_TTS_URL, {
    method: 'POST',
    headers: grokTtsAuthHeaders(auth),
    body: payload
  })

  if ((res.status === 401 || res.status === 403) && auth.kind === 'oauth' && !auth.cliTokenAuth) {
    res = await fetch(GROK_TTS_URL, {
      method: 'POST',
      headers: grokTtsAuthHeaders(auth, true),
      body: payload
    })
  }

  if (!res.ok) {
    const errText = await res.text().catch(() => '')
    throw new Error(grokTtsErrorMessage(res.status, errText))
  }

  const raw = Buffer.from(await res.arrayBuffer())
  const buf = decodeTtsAudio(raw, res.headers.get('content-type') || '')
  if (buf.length < 64) throw new Error('Grok TTS returned an empty audio payload.')

  const dir = join(projectDir(req.projectId), 'vo')
  await fs.mkdir(dir, { recursive: true })
  const stamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19)
  const slug = voFileSlug(req.name || 'voice', req.voiceSheetId)
  const path = join(dir, `${slug}-${stamp}.mp3`)
  await fs.writeFile(path, buf)

  return {
    path,
    fileUrl: pathToFileURL(path).href,
    bytes: buf.length,
    voiceId: body.voice_id
  }
}
