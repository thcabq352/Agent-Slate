#!/usr/bin/env node
// Unpacked desktop package for Windows, macOS, and Linux.
// macOS: delegates to package-macos.mjs (Agent-Slate.app).
// Windows/Linux: copies the local Electron runtime + built `out/` into dist/Agent-Slate/.
//
// Usage:
//   npm run build
//   node scripts/package-desktop.mjs [--install]
//
// --install
//   Windows: %LOCALAPPDATA%\Programs\Agent-Slate + Start Menu shortcut
//   macOS:   /Applications/Agent-Slate.app
//   Linux:   ~/.local/opt/agent-slate + ~/.local/bin/agent-slate + .desktop

import { execFileSync, spawnSync } from 'child_process'
import {
  cpSync,
  rmSync,
  mkdirSync,
  writeFileSync,
  existsSync,
  readFileSync,
  chmodSync,
  symlinkSync,
  unlinkSync
} from 'fs'
import { homedir } from 'os'
import { dirname, join, resolve } from 'path'
import { fileURLToPath } from 'url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const platform = process.platform
const wantInstall = process.argv.includes('--install')
const pkg = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'))
const outMain = join(root, 'out', 'main', 'index.js')
const electronDist = join(root, 'node_modules', 'electron', 'dist')
const distRoot = join(root, 'dist')

if (platform === 'darwin') {
  const args = ['scripts/package-macos.mjs', ...(wantInstall ? ['--install'] : [])]
  const r = spawnSync(process.execPath, args, { cwd: root, stdio: 'inherit' })
  process.exit(r.status ?? 1)
}

if (!existsSync(outMain)) {
  console.error('No build found — run `npm run build` first.')
  process.exit(1)
}
if (!existsSync(electronDist)) {
  console.error('Electron runtime missing — run `npm ci`.')
  process.exit(1)
}

const unpacked = join(distRoot, 'Agent-Slate')
console.log('→ unpacking Electron runtime')
rmSync(distRoot, { recursive: true, force: true })
mkdirSync(unpacked, { recursive: true })
cpSync(electronDist, unpacked, { recursive: true })

const res = join(unpacked, 'resources')
rmSync(join(res, 'default_app.asar'), { force: true })
const payload = join(res, 'app')
mkdirSync(payload, { recursive: true })
cpSync(join(root, 'out'), join(payload, 'out'), { recursive: true })
if (existsSync(join(root, 'build', 'icon.png'))) {
  mkdirSync(join(payload, 'build'), { recursive: true })
  cpSync(join(root, 'build', 'icon.png'), join(payload, 'build', 'icon.png'))
}
cpSync(join(root, 'mcp'), join(payload, 'mcp'), { recursive: true })
writeFileSync(
  join(payload, 'package.json'),
  JSON.stringify(
    {
      name: 'agent-slate',
      productName: 'Agent-Slate',
      version: pkg.version,
      main: 'out/main/index.js'
    },
    null,
    2
  )
)

const exeSrc = platform === 'win32' ? join(unpacked, 'electron.exe') : join(unpacked, 'electron')
const exeDst = platform === 'win32' ? join(unpacked, 'Agent-Slate.exe') : join(unpacked, 'Agent-Slate')
if (existsSync(exeSrc)) {
  cpSync(exeSrc, exeDst)
  if (platform !== 'win32') chmodSync(exeDst, 0o755)
}
const sandbox = join(unpacked, 'chrome-sandbox')
if (existsSync(sandbox) && platform === 'linux') {
  try {
    chmodSync(sandbox, 0o755)
  } catch {
    /* ignore */
  }
}

console.log('✓ packaged', unpacked)

if (!wantInstall) process.exit(0)

if (platform === 'win32') {
  const dest = join(process.env.LOCALAPPDATA || join(homedir(), 'AppData', 'Local'), 'Programs', 'Agent-Slate')
  console.log('→ installing to', dest)
  rmSync(dest, { recursive: true, force: true })
  mkdirSync(dirname(dest), { recursive: true })
  cpSync(unpacked, dest, { recursive: true })
  const exe = join(dest, 'Agent-Slate.exe')
  const programs = join(
    process.env.APPDATA || join(homedir(), 'AppData', 'Roaming'),
    'Microsoft',
    'Windows',
    'Start Menu',
    'Programs',
    'Agent-Slate.lnk'
  )
  const ps = `
    $ws = New-Object -ComObject WScript.Shell
    $s = $ws.CreateShortcut(${JSON.stringify(programs)})
    $s.TargetPath = ${JSON.stringify(exe)}
    $s.WorkingDirectory = ${JSON.stringify(dest)}
    $s.Description = 'Agent-Slate'
    $s.Save()
  `
  execFileSync('powershell', ['-NoProfile', '-Command', ps], { stdio: 'inherit' })
  console.log('✓ Start Menu shortcut Agent-Slate.lnk')
} else {
  const dest = join(homedir(), '.local', 'opt', 'agent-slate')
  const binDir = join(homedir(), '.local', 'bin')
  const link = join(binDir, 'agent-slate')
  console.log('→ installing to', dest)
  rmSync(dest, { recursive: true, force: true })
  mkdirSync(dirname(dest), { recursive: true })
  cpSync(unpacked, dest, { recursive: true })
  mkdirSync(binDir, { recursive: true })
  try {
    unlinkSync(link)
  } catch {
    /* missing */
  }
  const exe = join(dest, 'Agent-Slate')
  chmodSync(exe, 0o755)
  symlinkSync(exe, link)
  const apps = join(homedir(), '.local', 'share', 'applications')
  mkdirSync(apps, { recursive: true })
  const icon = join(dest, 'resources', 'app', 'build', 'icon.png')
  writeFileSync(
    join(apps, 'agent-slate.desktop'),
    `[Desktop Entry]
Type=Application
Name=Agent-Slate
Comment=Prompt studio + local film factory
Exec=env ELECTRON_DISABLE_SANDBOX=1 "${exe}"
Icon=${existsSync(icon) ? icon : ''}
Terminal=false
Categories=AudioVideo;Video;
`
  )
  console.log('✓ ~/.local/bin/agent-slate  (add ~/.local/bin to PATH if needed)')
}
