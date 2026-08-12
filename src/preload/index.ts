import { contextBridge, ipcRenderer, webUtils } from 'electron'
import type { BrainBackend, BrainRequest, Project, SlateApi } from '../shared/types'

const api: SlateApi & { brainRunWith: (req: BrainRequest, backend: BrainBackend) => Promise<unknown> } = {
  listProjects: () => ipcRenderer.invoke('projects:list'),
  createProject: (name: string) => ipcRenderer.invoke('projects:create', name),
  openProject: (id: string) => ipcRenderer.invoke('projects:open', id),
  saveProject: (project: Project) => ipcRenderer.invoke('projects:save', project),
  deleteProject: (id: string) => ipcRenderer.invoke('projects:delete', id),
  revealProject: (id: string) => ipcRenderer.invoke('projects:reveal', id),
  brainStatus: (localEndpoint?: string) => ipcRenderer.invoke('brain:status', localEndpoint),
  localModels: (endpoint?: string) => ipcRenderer.invoke('brain:localModels', endpoint),
  stillsDiscover: () => ipcRenderer.invoke('stills:discover'),
  stillsExtract: (projectId: string, mediaPath: string, inSec?: number | null, outSec?: number | null) =>
    ipcRenderer.invoke('stills:extract', projectId, mediaPath, inSec, outSec),
  brainRun: (req: BrainRequest) => ipcRenderer.invoke('brain:run', req),
  brainRunWith: (req: BrainRequest, backend: BrainBackend) =>
    ipcRenderer.invoke('brain:run', { ...req, backend }),
  brainCancel: (id: string) => ipcRenderer.invoke('brain:cancel', id),
  brainTest: (backend: BrainBackend, local?: { endpoint?: string; model?: string }) => ipcRenderer.invoke('brain:test', backend, local),
  pickMedia: () => ipcRenderer.invoke('media:pick'),
  pickAudio: () => ipcRenderer.invoke('media:pickAudio'),
  ingestMedia: (projectId: string, path: string) => ipcRenderer.invoke('media:ingest', projectId, path),
  analyzeAudio: (path: string) => ipcRenderer.invoke('sound:analyze', path),
  pathForFile: (file: File) => webUtils.getPathForFile(file),
  copyText: (text: string) => ipcRenderer.invoke('clipboard:copy', text),
  onProjectsChanged: (cb: () => void) => {
    const listener = (): void => cb()
    ipcRenderer.on('projects:changed', listener)
    return () => ipcRenderer.removeListener('projects:changed', listener)
  },
  onAboutOpen: (cb: () => void) => {
    const handler = (): void => cb()
    ipcRenderer.on('about:open', handler)
    return () => ipcRenderer.removeListener('about:open', handler)
  },
  onHelpOpen: (cb: () => void) => {
    const listener = (): void => cb()
    ipcRenderer.on('help:open', listener)
    return () => ipcRenderer.removeListener('help:open', listener)
  },
  engineEnsure: () => ipcRenderer.invoke('engine:ensure'),
  engineHealth: () => ipcRenderer.invoke('engine:health'),
  engineStatus: () => ipcRenderer.invoke('engine:status'),
  engineInvoke: (tool: string, args: Record<string, unknown> = {}) =>
    ipcRenderer.invoke('engine:invoke', tool, args)
}

contextBridge.exposeInMainWorld('slate', api)
