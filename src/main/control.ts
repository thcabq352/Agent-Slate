// Control server — localhost-only HTTP endpoint with a bearer token, so the
// bundled MCP bridge (mcp/slate-mcp.mjs) and suite apps can read/write projects.
// Pattern shared across the suite: random port + token descriptor on disk.

import { createServer, IncomingMessage, ServerResponse } from 'http'
import { promises as fs } from 'fs'
import { join } from 'path'
import { homedir } from 'os'
import { randomBytes } from 'crypto'
import { listProjects, openProject, saveProject, createProject } from './projects'
import { brainStatus, detectLocal } from './brain'
import type { Project, Scene, Shot } from '../shared/types'
import { BRAIN_BACKENDS, normalizeBrain } from '../shared/types'

type Notify = () => void

const token = randomBytes(24).toString('hex')

/** Distinct from slate-engine's `engine-control.json` — last-writer-wins collision. */
const CONTROL_APP = 'slate-electron'

function descriptorPath(): string {
  const base =
    process.platform === 'win32'
      ? process.env.APPDATA || join(homedir(), 'AppData', 'Roaming')
      : join(homedir(), '.config')
  return join(base, 'slate', 'electron-control.json')
}

async function readBody(req: IncomingMessage): Promise<string> {
  let data = ''
  for await (const chunk of req) data += chunk
  return data
}

function findShot(p: Project, shotId: string): { scene: Scene; shot: Shot } | null {
  for (const scene of p.scenes) {
    const shot = scene.shots.find((s) => s.id === shotId)
    if (shot) return { scene, shot }
  }
  return null
}

// Tools exposed to MCP clients. Each takes args, returns a JSON-able result.
const tools: Record<string, (args: Record<string, unknown>, notify: Notify) => Promise<unknown>> = {
  async list_projects() {
    return await listProjects()
  },
  async get_project(args) {
    const p = await openProject(String(args.projectId))
    if (!p) throw new Error('Project not found')
    return p
  },
  async create_project(args, notify) {
    const p = await createProject(String(args.name || 'Untitled'))
    notify()
    return { id: p.id, name: p.name }
  },
  async list_shots(args) {
    const p = await openProject(String(args.projectId))
    if (!p) throw new Error('Project not found')
    return p.scenes.map((sc) => ({
      sceneId: sc.id,
      scene: sc.name,
      shots: sc.shots.map((s) => ({ id: s.id, name: s.name, intent: s.intent, spec: s.spec }))
    }))
  },
  async get_shot_prompt(args) {
    const p = await openProject(String(args.projectId))
    if (!p) throw new Error('Project not found')
    const hit = findShot(p, String(args.shotId))
    if (!hit) throw new Error('Shot not found')
    return { prompt: hit.shot.prompt, spec: hit.shot.spec, beatSheet: hit.shot.beatSheet }
  },
  async set_shot_prompt(args, notify) {
    const p = await openProject(String(args.projectId))
    if (!p) throw new Error('Project not found')
    const hit = findShot(p, String(args.shotId))
    if (!hit) throw new Error('Shot not found')
    hit.shot.history.unshift({
      id: `v${Date.now()}`,
      savedAt: new Date().toISOString(),
      label: 'external edit (MCP)',
      prompt: hit.shot.prompt
    })
    hit.shot.prompt = String(args.prompt ?? '')
    hit.shot.updatedAt = new Date().toISOString()
    await saveProject(p)
    notify()
    return { ok: true }
  },
  async add_scene(args, notify) {
    const p = await openProject(String(args.projectId))
    if (!p) throw new Error('Project not found')
    const scene: Scene = {
      id: `scene-${Date.now()}`,
      name: String(args.name || `Scene ${p.scenes.length + 1}`),
      synopsis: String(args.synopsis || ''),
      shots: []
    }
    p.scenes.push(scene)
    await saveProject(p)
    notify()
    return { sceneId: scene.id }
  },
  async list_characters(args) {
    const p = await openProject(String(args.projectId))
    if (!p) throw new Error('Project not found')
    return p.characters
  },
  async list_locations(args) {
    const p = await openProject(String(args.projectId))
    if (!p) throw new Error('Project not found')
    return p.locations
  },
  async list_lookbook(args) {
    const p = await openProject(String(args.projectId))
    if (!p) throw new Error('Project not found')
    return p.lookbook
  },
  async brain_status() {
    return await brainStatus()
  },
  async list_local_models(args) {
    return await detectLocal(args.endpoint ? String(args.endpoint) : undefined)
  },
  async set_brain(args, notify) {
    const p = await openProject(String(args.projectId))
    if (!p) throw new Error('Project not found')
    const raw = String(args.brain)
    if (!(BRAIN_BACKENDS as readonly string[]).includes(raw) && raw !== 'claude') {
      throw new Error('brain must be one of: cursor, grok-4.5, grok-4.6, codex, local')
    }
    const brain = normalizeBrain(raw)
    p.defaults.brain = brain
    if (args.endpoint !== undefined) p.defaults.localEndpoint = String(args.endpoint)
    if (args.model !== undefined) p.defaults.localModel = String(args.model)
    await saveProject(p)
    notify()
    return { ok: true, brain: p.defaults.brain }
  }
}

export function toolCatalog(): Array<{ name: string; description: string }> {
  return [
    { name: 'list_projects', description: 'List Agent-Slate projects with scene/shot counts' },
    { name: 'get_project', description: 'Get a full Agent-Slate project (args: projectId)' },
    { name: 'create_project', description: 'Create a new Agent-Slate project (args: name)' },
    { name: 'list_shots', description: 'List scenes and shots (args: projectId)' },
    { name: 'get_shot_prompt', description: 'Get a shot prompt + spec (args: projectId, shotId)' },
    { name: 'set_shot_prompt', description: 'Set a shot prompt, versioning the old one (args: projectId, shotId, prompt)' },
    { name: 'add_scene', description: 'Add a scene (args: projectId, name, synopsis)' },
    { name: 'list_characters', description: 'List character sheets (args: projectId)' },
    { name: 'list_locations', description: 'List location sheets (args: projectId)' },
    { name: 'list_lookbook', description: 'List style profiles (args: projectId)' },
    { name: 'brain_status', description: 'Report which brains are available: Grok Build, Cursor CLI, Codex, and any local model server' },
    { name: 'list_local_models', description: 'List models on the local OpenAI-compatible server (args: endpoint?)' },
    { name: 'set_brain', description: 'Set a project brain: cursor | grok-4.5 | grok-4.6 | codex | local (args: projectId, brain, endpoint?, model?)' }
  ]
}

export async function startControlServer(notify: Notify): Promise<void> {
  const server = createServer(async (req: IncomingMessage, res: ServerResponse) => {
    const send = (code: number, body: unknown): void => {
      res.writeHead(code, { 'Content-Type': 'application/json' })
      res.end(JSON.stringify(body))
    }
    if (req.headers.authorization !== `Bearer ${token}`) {
      send(401, { error: 'unauthorized' })
      return
    }
    if (req.method === 'GET' && req.url === '/tools') {
      send(200, { tools: toolCatalog() })
      return
    }
    if (req.method === 'POST' && req.url === '/invoke') {
      try {
        const { tool, args } = JSON.parse(await readBody(req))
        const fn = tools[tool]
        if (!fn) {
          send(404, { error: `Unknown tool: ${tool}` })
          return
        }
        send(200, { result: await fn(args ?? {}, notify) })
      } catch (e) {
        send(400, { error: e instanceof Error ? e.message : String(e) })
      }
      return
    }
    send(404, { error: 'not found' })
  })

  await new Promise<void>((resolve) => server.listen(0, '127.0.0.1', resolve))
  const address = server.address()
  const port = typeof address === 'object' && address ? address.port : 0

  const desc = descriptorPath()
  await fs.mkdir(join(desc, '..'), { recursive: true })
  await fs.writeFile(
    desc,
    JSON.stringify({ v: 1, app: CONTROL_APP, port, token, pid: process.pid }, null, 2),
    { mode: 0o600 }
  )
}
