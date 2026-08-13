// Project persistence — one folder per project under ~/Documents/Slate.
// Projects are small JSON (prompts + sheets); reference media is linked, never copied.

import { app } from 'electron'
import { promises as fs } from 'fs'
import { existsSync } from 'fs'
import { join } from 'path'
import { randomUUID } from 'crypto'
import type { Project, ProjectMeta } from '../shared/types'
import { normalizeBrain } from '../shared/types'

export function projectsRoot(): string {
  // SLATE_DATA_DIR overrides the default location (portable installs, tests).
  return process.env.SLATE_DATA_DIR || join(app.getPath('documents'), 'Slate')
}

export function projectDir(id: string): string {
  return join(projectsRoot(), id)
}

function projectFile(id: string): string {
  return join(projectDir(id), 'project.json')
}

export function cacheDir(id: string): string {
  return join(projectDir(id), '.cache')
}

async function ensureRoot(): Promise<void> {
  await fs.mkdir(projectsRoot(), { recursive: true })
}

export async function listProjects(): Promise<ProjectMeta[]> {
  await ensureRoot()
  const entries = await fs.readdir(projectsRoot(), { withFileTypes: true })
  const metas: ProjectMeta[] = []
  for (const e of entries) {
    if (!e.isDirectory()) continue
    const file = projectFile(e.name)
    if (!existsSync(file)) continue
    try {
      const p = JSON.parse(await fs.readFile(file, 'utf8')) as Project
      metas.push({
        id: p.id,
        name: p.name,
        logline: p.logline,
        path: projectDir(p.id),
        updatedAt: p.updatedAt,
        sceneCount: p.scenes.length,
        shotCount: p.scenes.reduce((n, s) => n + s.shots.length, 0)
      })
    } catch {
      // Unreadable project file — skip rather than crash the list.
    }
  }
  metas.sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
  return metas
}

export function newProject(name: string): Project {
  const now = new Date().toISOString()
  return {
    id: randomUUID(),
    name,
    logline: '',
    world: '',
    defaults: {
      aspectRatio: '16:9',
      fps: 24,
      durationSec: 8,
      targetModel: 'seedance-2',
      brain: 'cursor'
    },
    scenes: [],
    characters: [],
    artDept: [],
    locations: [],
    lookbook: [],
    references: [],
    mySetups: [],
    music: [],
    voices: [],
    createdAt: now,
    updatedAt: now
  }
}

export async function createProject(name: string): Promise<Project> {
  await ensureRoot()
  const p = newProject(name)
  await fs.mkdir(projectDir(p.id), { recursive: true })
  await saveProject(p)
  return p
}

export async function openProject(id: string): Promise<Project | null> {
  try {
    const p = JSON.parse(await fs.readFile(projectFile(id), 'utf8')) as Project
    p.defaults.brain = normalizeBrain(p.defaults.brain)
    return p
  } catch {
    return null
  }
}

export async function saveProject(project: Project): Promise<void> {
  project.defaults.brain = normalizeBrain(project.defaults.brain)
  project.updatedAt = new Date().toISOString()
  const dir = projectDir(project.id)
  await fs.mkdir(dir, { recursive: true })
  // Atomic write: temp file + rename, so a crash never corrupts a project.
  const tmp = join(dir, `.project.${Date.now()}.tmp`)
  await fs.writeFile(tmp, JSON.stringify(project, null, 2), 'utf8')
  await fs.rename(tmp, projectFile(project.id))
}

export async function deleteProject(id: string): Promise<void> {
  await fs.rm(projectDir(id), { recursive: true, force: true })
}
