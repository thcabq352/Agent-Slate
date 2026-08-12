// Browser-preview mock — lets the renderer run in a plain browser (no Electron)
// for UI development. Projects live in memory; brain calls return a stub notice.

import type { Project, SlateApi } from '../../../shared/types'
import { newProjectShape } from './newProject'

export function installDevMock(): void {
  if (typeof window === 'undefined' || (window as unknown as { slate?: SlateApi }).slate) return
  const projects = new Map<string, Project>()

  const api: SlateApi = {
    async listProjects() {
      return [...projects.values()].map((p) => ({
        id: p.id,
        name: p.name,
        logline: p.logline,
        path: `/dev-mock/${p.id}`,
        updatedAt: p.updatedAt,
        sceneCount: p.scenes.length,
        shotCount: p.scenes.reduce((n, s) => n + s.shots.length, 0)
      }))
    },
    async createProject(name) {
      const p = newProjectShape(name)
      projects.set(p.id, p)
      return structuredClone(p)
    },
    async openProject(id) {
      const p = projects.get(id)
      return p ? structuredClone(p) : null
    },
    async saveProject(project) {
      projects.set(project.id, structuredClone(project))
    },
    async deleteProject(id) {
      projects.delete(id)
    },
    async brainStatus() {
      return {
        claude: { available: false, version: null },
        codex: { available: false, version: null },
        local: { available: false, version: null, endpoint: null }
      }
    },
    async brainRun(req) {
      return {
        id: req.id,
        ok: false,
        text: '',
        error: 'Browser preview — the brain runs only in the desktop app.',
        elapsedMs: 0
      }
    },
    async brainCancel() {},
    onAboutOpen() {
      return () => {}
    },
    async localModels() {
      return { endpoint: null, models: [] }
    },
    async stillsDiscover() {
      return []
    },
    async stillsExtract() {
      return []
    },
    async brainTest(): Promise<import('../../../shared/types').BrainResult> {
      return {
        id: 'test',
        ok: false,
        text: '',
        error: 'Browser preview — the brain runs only in the desktop app.',
        elapsedMs: 0
      }
    },
    async pickMedia() {
      return []
    },
    async pickAudio() {
      return []
    },
    async ingestMedia() {
      return { kind: 'image' as const, frames: [] }
    },
    async analyzeAudio() {
      throw new Error('Browser preview — audio analysis runs only in the desktop app.')
    },
    pathForFile() {
      return ''
    },
    async copyText(text) {
      await navigator.clipboard.writeText(text).catch(() => undefined)
    },
    async revealProject() {},
    onProjectsChanged() {
      return () => {}
    },
    onHelpOpen() {
      return () => {}
    },
    async engineEnsure() {
      return { ok: false, message: 'Browser preview — slate-engine not available.', descriptor: null }
    },
    async engineHealth() {
      return { engine: false }
    },
    async engineStatus() {
      return { active: false, step: 'idle', message: 'preview' }
    },
    async engineInvoke() {
      throw new Error('Browser preview — slate-engine not available.')
    }
  }
  ;(window as unknown as { slate: SlateApi }).slate = api
}
