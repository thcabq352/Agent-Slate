// Local brain adapter — protocol tests against an in-process OpenAI-compatible server.
import { describe, it, expect, beforeAll, afterAll } from 'vitest'
import { createServer, Server } from 'http'
import { AddressInfo } from 'net'
import { detectLocal, brainRun } from '../src/main/brain'
import { normalizeBrain } from '../src/shared/types'

let server: Server
let endpoint = ''
let lastBody: Record<string, unknown> = {}
let replyWith: (prompt: string) => string = () => 'READY'

beforeAll(async () => {
  server = createServer((req, res) => {
    let data = ''
    req.on('data', (c) => (data += c))
    req.on('end', () => {
      if (req.url?.endsWith('/models')) {
        res.setHeader('Content-Type', 'application/json')
        res.end(JSON.stringify({ data: [{ id: 'test-model-a' }, { id: 'test-model-b' }] }))
        return
      }
      if (req.url?.endsWith('/chat/completions')) {
        lastBody = JSON.parse(data)
        const messages = lastBody.messages as Array<{ content: string }>
        const text = replyWith(String(messages[messages.length - 1].content))
        res.setHeader('Content-Type', 'application/json')
        res.end(JSON.stringify({ choices: [{ message: { content: text } }] }))
        return
      }
      res.statusCode = 404
      res.end()
    })
  })
  await new Promise<void>((r) => server.listen(0, '127.0.0.1', r))
  endpoint = `http://127.0.0.1:${(server.address() as AddressInfo).port}/v1`
})

afterAll(() => server.close())

describe('local brain adapter', () => {
  it('detects a server and lists its models', async () => {
    const found = await detectLocal(endpoint)
    expect(found.endpoint).toBe(endpoint)
    expect(found.models.map((m) => m.id)).toEqual(['test-model-a', 'test-model-b'])
  })

  it('normalizes endpoints missing scheme or /v1', async () => {
    const bare = endpoint.replace('http://', '').replace(/\/v1$/, '')
    const found = await detectLocal(bare)
    expect(found.endpoint).toBe(endpoint)
  })

  it('reports nothing when no server answers', async () => {
    const found = await detectLocal('http://127.0.0.1:1/v1')
    expect(found.endpoint).toBeNull()
    expect(found.models).toEqual([])
  })

  it('completes a request, defaulting to the first listed model', async () => {
    replyWith = () => 'READY'
    const res = await brainRun(
      {
        id: 'local-1',
        task: 'self-test',
        system: 'You are a connectivity check.',
        prompt: 'Reply with exactly: READY',
        tier: 'fast',
        localEndpoint: endpoint
      },
      'local'
    )
    expect(res.ok).toBe(true)
    expect(res.text).toBe('READY')
    expect(lastBody.model).toBe('test-model-a')
  })

  it('honors an explicit model id', async () => {
    await brainRun(
      { id: 'local-2', task: 't', system: 's', prompt: 'p', tier: 'fast', localEndpoint: endpoint, localModel: 'test-model-b' },
      'local'
    )
    expect(lastBody.model).toBe('test-model-b')
  })

  it('retries with a nudge when JSON is expected but prose comes back', async () => {
    let calls = 0
    replyWith = () => (++calls === 1 ? 'Sure! Here you go.' : '{"ok": true}')
    const res = await brainRun(
      { id: 'local-3', task: 't', system: 's', prompt: 'p', tier: 'fast', expectJson: true, localEndpoint: endpoint },
      'local'
    )
    expect(res.ok).toBe(true)
    expect(res.json).toEqual({ ok: true })
    expect(calls).toBe(2)
  })

  it('fails with guidance when no server is reachable', async () => {
    const res = await brainRun(
      { id: 'local-4', task: 't', system: 's', prompt: 'p', tier: 'fast', localEndpoint: 'http://127.0.0.1:1/v1' },
      'local'
    )
    expect(res.ok).toBe(false)
    expect(res.error).toMatch(/No local model server found/)
  })
})

describe('normalizeBrain', () => {
  it('maps legacy claude onto cursor', () => {
    expect(normalizeBrain('claude')).toBe('cursor')
    expect(normalizeBrain('cursor')).toBe('cursor')
    expect(normalizeBrain('codex')).toBe('codex')
    expect(normalizeBrain('local')).toBe('local')
    expect(normalizeBrain('grok-4.5')).toBe('grok-4.5')
    expect(normalizeBrain('grok-4.6')).toBe('grok-4.6')
    expect(normalizeBrain(undefined)).toBe('cursor')
  })
})
