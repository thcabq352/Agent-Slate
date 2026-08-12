import { app, BrowserWindow, ipcMain, dialog, shell, clipboard, Menu } from 'electron'
import { join } from 'path'
import { brainRun, brainCancel, brainStatus, detectLocal } from './brain'
import { discoverCircledTakes, extractStills } from './stills'
import {
  listProjects,
  createProject,
  openProject,
  saveProject,
  deleteProject,
  projectsRoot,
  cacheDir
} from './projects'
import { startControlServer } from './control'
import { extractFrames, mediaKind } from './ingest'
import { analyzeAudio } from './audio'
import type { BrainBackend, BrainRequest, Project } from '../shared/types'

let win: BrowserWindow | null = null

function notifyProjectsChanged(): void {
  win?.webContents.send('projects:changed')
}

function createWindow(): void {
  win = new BrowserWindow({
    width: 1520,
    height: 940,
    minWidth: 1100,
    minHeight: 680,
    title: 'Slate',
    icon: join(__dirname, '../../build/icon.png'),
    titleBarStyle: 'hiddenInset',
    trafficLightPosition: { x: 18, y: 16 },
    backgroundColor: '#0c0d10',
    webPreferences: {
      preload: join(__dirname, '../preload/index.js'),
      contextIsolation: true,
      nodeIntegration: false
    }
  })

  if (process.env.ELECTRON_RENDERER_URL) {
    win.loadURL(process.env.ELECTRON_RENDERER_URL)
  } else {
    win.loadFile(join(__dirname, '../renderer/index.html'))
  }

  win.webContents.setWindowOpenHandler(({ url }) => {
    shell.openExternal(url)
    return { action: 'deny' }
  })
}

app.setName('Slate')

app.setAboutPanelOptions({
  applicationName: 'Slate',
  applicationVersion: app.getVersion(),
  copyright: 'Apache-2.0 · Sam Wasserman',
  credits: 'The prompt studio for AI filmmaking.\nPlan · Direct · Compile',
  iconPath: join(__dirname, '../../build/icon.png')
})

function buildMenu(): void {
  const openHelp = (): void => {
    win?.webContents.send('help:open')
    win?.show()
  }
  const template: Electron.MenuItemConstructorOptions[] = [
    {
      label: 'Slate',
      submenu: [
        {
          label: 'About Slate',
          click: () => {
            win?.webContents.send('about:open')
            win?.show()
          }
        },
        { type: 'separator' },
        { label: 'Slate Help', accelerator: 'CmdOrCtrl+/', click: openHelp },
        { type: 'separator' },
        { label: 'Support Slate on Ko-fi ♥', click: () => void shell.openExternal('https://ko-fi.com/samwasserman') },
        { label: 'wassermanproductions.com', click: () => void shell.openExternal('https://wassermanproductions.com') },
        { label: 'wasserman.ai', click: () => void shell.openExternal('https://wasserman.ai') },
        { type: 'separator' },
        { role: 'hide', label: 'Hide Slate' },
        { role: 'hideOthers' },
        { role: 'unhide' },
        { type: 'separator' },
        { role: 'quit', label: 'Quit Slate' }
      ]
    },
    { role: 'fileMenu' },
    { role: 'editMenu' },
    { role: 'viewMenu' },
    { role: 'windowMenu' },
    {
      role: 'help',
      submenu: [
        { label: 'Slate Help', accelerator: 'CmdOrCtrl+?', click: openHelp },
        { type: 'separator' },
        { label: 'Support Slate on Ko-fi ♥', click: () => void shell.openExternal('https://ko-fi.com/samwasserman') },
        { label: 'wassermanproductions.com', click: () => void shell.openExternal('https://wassermanproductions.com') },
        { label: 'wasserman.ai', click: () => void shell.openExternal('https://wasserman.ai') }
      ]
    }
  ]
  Menu.setApplicationMenu(Menu.buildFromTemplate(template))
}

// Headless capture mode (SLATE_SNAP_SCRIPT + SLATE_SNAP_OUT): drive the UI
// from a step script and save PNGs from a hidden window. Used by scripts/snap.
async function runSnap(): Promise<void> {
  const { readFileSync, writeFileSync, mkdirSync } = await import('fs')
  const scriptPath = process.env.SLATE_SNAP_SCRIPT!
  const outDir = process.env.SLATE_SNAP_OUT || '.'
  mkdirSync(outDir, { recursive: true })
  const steps = JSON.parse(readFileSync(scriptPath, 'utf8')) as Array<{
    js?: string
    wait?: number
    shot?: string
    startRecord?: { fps?: number }
    stopRecord?: string
  }>
  const w = new BrowserWindow({
    width: 1560,
    height: 975,
    show: false,
    backgroundColor: '#0a0b0d',
    webPreferences: {
      preload: join(__dirname, '../preload/index.js'),
      contextIsolation: true,
      nodeIntegration: false,
      backgroundThrottling: false
    }
  })
  if (process.env.ELECTRON_RENDERER_URL) await w.loadURL(process.env.ELECTRON_RENDERER_URL)
  else await w.loadFile(join(__dirname, '../renderer/index.html'))
  await new Promise((r) => setTimeout(r, 900))

  // Recording state: capture frames on an interval; each frame carries its
  // real timestamp so assembly can be timestamp-accurate.
  let recTimer: ReturnType<typeof setInterval> | null = null
  let recFrames: Array<{ t: number; buf: Buffer }> = []
  let capturing = false
  const startRecord = (fps: number): void => {
    recFrames = []
    recTimer = setInterval(() => {
      if (capturing) return
      capturing = true
      const t = Date.now()
      void w.webContents
        .capturePage()
        .then((img) => {
          recFrames.push({ t, buf: img.toJPEG(90) })
        })
        .finally(() => {
          capturing = false
        })
    }, Math.round(1000 / fps))
  }
  const stopRecord = (name: string): void => {
    if (recTimer) clearInterval(recTimer)
    recTimer = null
    const dir = join(outDir, name)
    mkdirSync(dir, { recursive: true })
    const meta: Array<{ file: string; t: number }> = []
    recFrames.forEach((f, i) => {
      const file = `f${String(i).padStart(5, '0')}.jpg`
      writeFileSync(join(dir, file), f.buf)
      meta.push({ file, t: f.t })
    })
    writeFileSync(join(dir, 'meta.json'), JSON.stringify(meta))
    console.log('rec:', name, recFrames.length, 'frames')
    recFrames = []
  }

  for (const step of steps) {
    try {
      if (step.startRecord) startRecord(step.startRecord.fps ?? 30)
      if (step.js) await w.webContents.executeJavaScript(step.js, true)
      if (step.wait) await new Promise((r) => setTimeout(r, step.wait))
      if (step.stopRecord) stopRecord(step.stopRecord)
      if (step.shot) {
        const img = await w.webContents.capturePage()
        writeFileSync(join(outDir, `${step.shot}.png`), img.toPNG())
        console.log('snap:', step.shot, img.getSize().width + 'x' + img.getSize().height)
      }
    } catch (e) {
      console.error('snap step failed:', JSON.stringify(step).slice(0, 80), e)
    }
  }
  app.exit(0)
}

app.whenReady().then(async () => {
  if (process.env.SLATE_SNAP_SCRIPT) {
    await runSnap()
    return
  }
  buildMenu()
  createWindow()
  await startControlServer(notifyProjectsChanged)

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow()
  })
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit()
})

// ---- IPC ----

ipcMain.handle('projects:list', () => listProjects())
ipcMain.handle('projects:create', (_e, name: string) => createProject(name))
ipcMain.handle('projects:open', (_e, id: string) => openProject(id))
ipcMain.handle('projects:save', (_e, project: Project) => saveProject(project))
ipcMain.handle('projects:delete', (_e, id: string) => deleteProject(id))
ipcMain.handle('projects:reveal', (_e, id: string) => {
  shell.showItemInFolder(join(projectsRoot(), id, 'project.json'))
})

ipcMain.handle('brain:status', (_e, localEndpoint?: string) => brainStatus(localEndpoint))
ipcMain.handle('brain:localModels', (_e, endpoint?: string) => detectLocal(endpoint))
ipcMain.handle('brain:test', async (_e, backend: BrainBackend, local?: { endpoint?: string; model?: string }) => {
  return brainRun(
    {
      id: `test-${Date.now()}`,
      task: 'self-test',
      system: 'You are a connectivity check. Reply with exactly one word.',
      prompt: 'Reply with exactly: READY',
      tier: 'fast',
      localEndpoint: local?.endpoint,
      localModel: local?.model
    },
    backend
  )
})
ipcMain.handle('brain:run', (_e, req: BrainRequest & { backend: BrainBackend }) =>
  brainRun(req, req.backend)
)
ipcMain.handle('brain:cancel', (_e, id: string) => brainCancel(id))

ipcMain.handle('stills:discover', () => discoverCircledTakes())
ipcMain.handle('stills:extract', (_e, projectId: string, mediaPath: string, inSec?: number | null, outSec?: number | null) =>
  extractStills(cacheDir(projectId), mediaPath, inSec, outSec)
)

ipcMain.handle('media:pick', async () => {
  const res = await dialog.showOpenDialog({
    properties: ['openFile', 'multiSelections'],
    filters: [
      { name: 'Media', extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif', 'mp4', 'mov', 'm4v', 'webm', 'mkv'] }
    ]
  })
  return res.canceled ? [] : res.filePaths
})

ipcMain.handle('media:ingest', async (_e, projectId: string, path: string) => {
  const kind = mediaKind(path)
  if (!kind) throw new Error('Unsupported media type')
  if (kind === 'image') return { kind, frames: [path] }
  const frames = await extractFrames(projectId, path)
  return { kind, frames }
})

ipcMain.handle('media:pickAudio', async () => {
  const res = await dialog.showOpenDialog({
    properties: ['openFile'],
    filters: [
      {
        name: 'Audio & Video',
        extensions: ['mp3', 'wav', 'm4a', 'aac', 'flac', 'ogg', 'aif', 'aiff', 'mp4', 'mov', 'm4v', 'webm', 'mkv']
      }
    ]
  })
  return res.canceled ? [] : res.filePaths
})

ipcMain.handle('sound:analyze', (_e, path: string) => analyzeAudio(path))

ipcMain.handle('clipboard:copy', (_e, text: string) => {
  clipboard.writeText(text)
})
