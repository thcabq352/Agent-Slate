import { create } from 'zustand'
import type {
  Project,
  ProjectMeta,
  Scene,
  Shot,
  BrainStatus,
  CharacterSheet,
  ArtDeptSheet,
  LocationSheet,
  StyleProfile,
  Reference,
  CustomSetup,
  PromptVersion
} from '../../../shared/types'

declare global {
  interface Window {
    slate: import('../../../shared/types').SlateApi
  }
}

let saveTimer: ReturnType<typeof setTimeout> | null = null

function uid(prefix: string): string {
  return `${prefix}-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`
}

export function blankShot(name: string, defaults: Project['defaults']): Shot {
  const now = new Date().toISOString()
  return {
    id: uid('shot'),
    name,
    intent: '',
    spec: {
      durationSec: defaults.durationSec,
      fps: defaults.fps,
      aspectRatio: defaults.aspectRatio,
      lens: null,
      movement: null,
      size: null,
      angle: null
    },
    prompt: '',
    lockedLines: [],
    mutedLines: [],
    beatSheet: null,
    targetModel: defaults.targetModel,
    maxChars: null,
    variants: [],
    history: [],
    takes: [],
    createdAt: now,
    updatedAt: now
  }
}

interface ProjectState {
  metas: ProjectMeta[]
  project: Project | null
  sceneId: string | null
  shotId: string | null
  brain: BrainStatus | null
  dirty: boolean

  refreshMetas(): Promise<void>
  refreshBrain(): Promise<void>
  create(name: string): Promise<void>
  open(id: string): Promise<void>
  close(): void
  remove(id: string): Promise<void>

  mutate(fn: (p: Project) => void): void
  persist(): Promise<void>
  selectScene(id: string | null): void
  selectShot(sceneId: string, shotId: string): void

  addScene(name: string): void
  renameScene(id: string, name: string): void
  deleteScene(id: string): void
  addShot(sceneId: string): void
  deleteShot(sceneId: string, shotId: string): void

  currentScene(): Scene | null
  currentShot(): Shot | null

  setPrompt(text: string, historyLabel?: string): void
  snapshotVersion(label: string): void
  restoreVersion(v: PromptVersion): void

  upsertCharacter(c: CharacterSheet): void
  removeCharacter(id: string): void
  upsertArtDept(a: ArtDeptSheet): void
  removeArtDept(id: string): void
  upsertLocation(l: LocationSheet): void
  removeLocation(id: string): void
  upsertStyle(s: StyleProfile): void
  removeStyle(id: string): void
  addReference(r: Reference): void
  removeReference(id: string): void
  upsertSetup(s: CustomSetup): void
  removeSetup(id: string): void
  upsertMusic(m: import('../../../shared/types').MusicCue): void
  removeMusic(id: string): void
  upsertVoice(v: import('../../../shared/types').VoiceSheet): void
  removeVoice(id: string): void
}

export const useProject = create<ProjectState>((set, get) => ({
  metas: [],
  project: null,
  sceneId: null,
  shotId: null,
  brain: null,
  dirty: false,

  async refreshMetas() {
    set({ metas: await window.slate.listProjects() })
  },

  async refreshBrain() {
    set({ brain: await window.slate.brainStatus() })
  },

  async create(name) {
    const p = await window.slate.createProject(name)
    set({ project: p, sceneId: null, shotId: null })
    await get().refreshMetas()
  },

  async open(id) {
    const p = await window.slate.openProject(id)
    if (p) {
      const firstScene = p.scenes[0] ?? null
      set({
        project: p,
        sceneId: firstScene?.id ?? null,
        shotId: firstScene?.shots[0]?.id ?? null
      })
    }
  },

  close() {
    set({ project: null, sceneId: null, shotId: null })
    void get().refreshMetas()
  },

  async remove(id) {
    await window.slate.deleteProject(id)
    if (get().project?.id === id) set({ project: null, sceneId: null, shotId: null })
    await get().refreshMetas()
  },

  mutate(fn) {
    const p = get().project
    if (!p) return
    const next = structuredClone(p)
    fn(next)
    set({ project: next, dirty: true })
    if (saveTimer) clearTimeout(saveTimer)
    saveTimer = setTimeout(() => {
      void window.slate.saveProject(next).then(() => set({ dirty: false }))
    }, 500)
  },

  async persist() {
    const p = get().project
    if (!p) return
    if (saveTimer) {
      clearTimeout(saveTimer)
      saveTimer = null
    }
    await window.slate.saveProject(p)
    set({ dirty: false })
  },

  selectScene(id) {
    const scene = get().project?.scenes.find((s) => s.id === id)
    set({ sceneId: id, shotId: scene?.shots[0]?.id ?? null })
  },

  selectShot(sceneId, shotId) {
    set({ sceneId, shotId })
  },

  addScene(name) {
    const id = uid('scene')
    get().mutate((p) => {
      p.scenes.push({ id, name, synopsis: '', shots: [] })
    })
    set({ sceneId: id, shotId: null })
  },

  renameScene(id, name) {
    get().mutate((p) => {
      const s = p.scenes.find((x) => x.id === id)
      if (s) s.name = name
    })
  },

  deleteScene(id) {
    get().mutate((p) => {
      p.scenes = p.scenes.filter((x) => x.id !== id)
    })
    if (get().sceneId === id) set({ sceneId: null, shotId: null })
  },

  addShot(sceneId) {
    const p = get().project
    if (!p) return
    const scene = p.scenes.find((s) => s.id === sceneId)
    const shot = blankShot(`Shot ${String((scene?.shots.length ?? 0) + 1).padStart(2, '0')}`, p.defaults)
    get().mutate((proj) => {
      proj.scenes.find((s) => s.id === sceneId)?.shots.push(shot)
    })
    set({ sceneId, shotId: shot.id })
  },

  deleteShot(sceneId, shotId) {
    get().mutate((p) => {
      const scene = p.scenes.find((s) => s.id === sceneId)
      if (scene) scene.shots = scene.shots.filter((s) => s.id !== shotId)
    })
    if (get().shotId === shotId) set({ shotId: null })
  },

  currentScene() {
    const { project, sceneId } = get()
    return project?.scenes.find((s) => s.id === sceneId) ?? null
  },

  currentShot() {
    const { shotId } = get()
    return get().currentScene()?.shots.find((s) => s.id === shotId) ?? null
  },

  setPrompt(text, historyLabel) {
    const { sceneId, shotId } = get()
    get().mutate((p) => {
      const shot = p.scenes.find((s) => s.id === sceneId)?.shots.find((s) => s.id === shotId)
      if (!shot) return
      if (historyLabel && shot.prompt.trim() && shot.prompt !== text) {
        shot.history.unshift({
          id: uid('v'),
          savedAt: new Date().toISOString(),
          label: historyLabel,
          prompt: shot.prompt
        })
        shot.history = shot.history.slice(0, 50)
      }
      shot.prompt = text
      shot.updatedAt = new Date().toISOString()
    })
  },

  snapshotVersion(label) {
    const { sceneId, shotId } = get()
    get().mutate((p) => {
      const shot = p.scenes.find((s) => s.id === sceneId)?.shots.find((s) => s.id === shotId)
      if (!shot || !shot.prompt.trim()) return
      shot.history.unshift({
        id: uid('v'),
        savedAt: new Date().toISOString(),
        label,
        prompt: shot.prompt
      })
      shot.history = shot.history.slice(0, 50)
    })
  },

  restoreVersion(v) {
    get().setPrompt(v.prompt, 'before restore')
  },

  upsertCharacter(c) {
    get().mutate((p) => {
      const i = p.characters.findIndex((x) => x.id === c.id)
      if (i >= 0) p.characters[i] = c
      else p.characters.push(c)
    })
  },
  removeCharacter(id) {
    get().mutate((p) => {
      p.characters = p.characters.filter((x) => x.id !== id)
    })
  },
  upsertArtDept(a) {
    get().mutate((p) => {
      const i = p.artDept.findIndex((x) => x.id === a.id)
      if (i >= 0) p.artDept[i] = a
      else p.artDept.push(a)
    })
  },
  removeArtDept(id) {
    get().mutate((p) => {
      p.artDept = p.artDept.filter((x) => x.id !== id)
    })
  },
  upsertLocation(l) {
    get().mutate((p) => {
      const i = p.locations.findIndex((x) => x.id === l.id)
      if (i >= 0) p.locations[i] = l
      else p.locations.push(l)
    })
  },
  removeLocation(id) {
    get().mutate((p) => {
      p.locations = p.locations.filter((x) => x.id !== id)
    })
  },
  upsertStyle(s) {
    get().mutate((p) => {
      const i = p.lookbook.findIndex((x) => x.id === s.id)
      if (i >= 0) p.lookbook[i] = s
      else p.lookbook.push(s)
    })
  },
  removeStyle(id) {
    get().mutate((p) => {
      p.lookbook = p.lookbook.filter((x) => x.id !== id)
    })
  },
  addReference(r) {
    get().mutate((p) => {
      p.references.push(r)
    })
  },
  removeReference(id) {
    get().mutate((p) => {
      p.references = p.references.filter((x) => x.id !== id)
    })
  },
  upsertSetup(s) {
    get().mutate((p) => {
      const i = p.mySetups.findIndex((x) => x.id === s.id)
      if (i >= 0) p.mySetups[i] = s
      else p.mySetups.push(s)
    })
  },
  removeSetup(id) {
    get().mutate((p) => {
      p.mySetups = p.mySetups.filter((x) => x.id !== id)
    })
  },
  upsertMusic(m) {
    get().mutate((p) => {
      p.music = p.music ?? []
      const i = p.music.findIndex((x) => x.id === m.id)
      if (i >= 0) p.music[i] = m
      else p.music.push(m)
    })
  },
  removeMusic(id) {
    get().mutate((p) => {
      p.music = (p.music ?? []).filter((x) => x.id !== id)
    })
  },
  upsertVoice(v) {
    get().mutate((p) => {
      p.voices = p.voices ?? []
      const i = p.voices.findIndex((x) => x.id === v.id)
      if (i >= 0) p.voices[i] = v
      else p.voices.push(v)
    })
  },
  removeVoice(id) {
    get().mutate((p) => {
      p.voices = (p.voices ?? []).filter((x) => x.id !== id)
    })
  }
}))

export { uid }
