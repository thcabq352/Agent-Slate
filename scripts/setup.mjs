#!/usr/bin/env node
// Cross-platform from-source setup for Agent-Slate (Windows, macOS, Linux).
// Usage: node scripts/setup.mjs [--ffmpeg] [--grok] [--cursor] [--engine] [--skip-npm]
//
// Does not store API keys. Grok/Cursor installers are the official vendor scripts.

import { spawnSync } from 'child_process'
import { existsSync, statSync } from 'fs'
import { homedir } from 'os'
import { dirname, join } from 'path'
import { fileURLToPath } from 'url'

const root = dirname(fileURLToPath(new URL('../package.json', import.meta.url)))
const platform = process.platform
const isWin = platform === 'win32'
const isMac = platform === 'darwin'
const args = new Set(process.argv.slice(2))
const wantFfmpeg = args.has('--ffmpeg') || !args.has('--skip-ffmpeg')
const wantGrok = args.has('--grok')
const wantCursor = args.has('--cursor')
const wantEngine = args.has('--engine')
const skipNpm = args.has('--skip-npm')

function log(msg) {
  console.log(`→ ${msg}`)
}

function warn(msg) {
  console.warn(`! ${msg}`)
}

function run(cmd, argv, opts = {}) {
  const r = spawnSync(cmd, argv, {
    cwd: root,
    stdio: 'inherit',
    windowsHide: true,
    ...opts
  })
  return r.status === 0
}

function which(name) {
  const probe = isWin ? 'where' : 'which'
  const r = spawnSync(probe, [name], { encoding: 'utf8', windowsHide: true })
  return r.status === 0
}

function isFile(p) {
  try {
    return statSync(p).isFile()
  } catch {
    return false
  }
}

function grokBin() {
  const home = homedir()
  return isWin ? join(home, '.grok', 'bin', 'grok.exe') : join(home, '.grok', 'bin', 'grok')
}

function ffmpegPresent() {
  if (which(isWin ? 'ffmpeg.exe' : 'ffmpeg')) return true
  const home = homedir()
  const cands = isWin
    ? [
        'C:\\ffmpeg\\bin\\ffmpeg.exe',
        join(process.env.ProgramFiles || 'C:\\Program Files', 'ffmpeg', 'bin', 'ffmpeg.exe'),
        join(process.env.LOCALAPPDATA || '', 'Microsoft', 'WinGet', 'Links', 'ffmpeg.exe')
      ]
    : ['/opt/homebrew/bin/ffmpeg', '/usr/local/bin/ffmpeg', '/usr/bin/ffmpeg', '/snap/bin/ffmpeg', join(home, '.local', 'bin', 'ffmpeg')]
  return cands.some((p) => p && isFile(p))
}

function requireNode20() {
  const m = Number(process.versions.node.split('.')[0])
  if (m < 20) {
    console.error(`Node 20+ required (found ${process.version}).`)
    if (isWin) console.error('  winget install OpenJS.NodeJS.LTS')
    else if (isMac) console.error('  brew install node')
    else console.error('  sudo apt install nodejs   # or install from https://nodejs.org')
    process.exit(1)
  }
}

function installFfmpeg() {
  if (ffmpegPresent()) {
    log('ffmpeg already on PATH or in a known location')
    return
  }
  log('installing ffmpeg')
  let ok = false
  if (isWin) {
    ok = run('winget', [
      'install',
      '--id',
      'Gyan.FFmpeg',
      '-e',
      '--accept-package-agreements',
      '--accept-source-agreements'
    ])
  } else if (isMac) {
    ok = which('brew') && run('brew', ['install', 'ffmpeg'])
    if (!ok) warn('Install Homebrew, then: brew install ffmpeg')
  } else if (which('apt-get')) {
    warn('Linux ffmpeg needs a package manager. Run: sudo apt install ffmpeg')
  } else if (which('dnf')) {
    warn('Run: sudo dnf install ffmpeg')
  } else if (which('pacman')) {
    warn('Run: sudo pacman -S ffmpeg')
  } else {
    warn('Install ffmpeg and re-run, or set SLATE_FFMPEG')
  }
  if (ok && !ffmpegPresent()) warn('ffmpeg installed — open a new terminal so PATH updates')
}

function installGrok() {
  if (isFile(grokBin()) || which(isWin ? 'grok.exe' : 'grok')) {
    log('Grok Build CLI already present')
    return
  }
  log('installing Grok Build CLI (official xAI installer)')
  let ok = false
  if (isWin) {
    ok = run('powershell', [
      '-NoProfile',
      '-ExecutionPolicy',
      'Bypass',
      '-Command',
      "irm https://x.ai/cli/install.ps1 | iex"
    ])
  } else {
    ok = run('bash', ['-lc', 'curl -fsSL https://x.ai/cli/install.sh | bash'])
  }
  if (!ok) {
    warn(
      isWin
        ? 'Grok install failed. In PowerShell: irm https://x.ai/cli/install.ps1 | iex'
        : 'Grok install failed. Run: curl -fsSL https://x.ai/cli/install.sh | bash'
    )
  }
}

function installCursor() {
  log('installing Cursor CLI (official installer)')
  let ok = false
  if (isWin) {
    ok = run('powershell', [
      '-NoProfile',
      '-ExecutionPolicy',
      'Bypass',
      '-Command',
      "irm 'https://cursor.com/install?win32=true' | iex"
    ])
  } else {
    ok = run('bash', ['-lc', 'curl https://cursor.com/install -fsS | bash'])
  }
  if (!ok) {
    warn(
      isWin
        ? "Cursor CLI install failed. In PowerShell: irm 'https://cursor.com/install?win32=true' | iex"
        : 'Cursor CLI install failed. Run: curl https://cursor.com/install -fsS | bash'
    )
  }
}

function npmCi() {
  log('npm ci')
  const npmCmd = isWin ? 'npm.cmd' : 'npm'
  if (!run(npmCmd, ['ci'])) {
    console.error('npm ci failed')
    process.exit(1)
  }
}

function buildEngine() {
  if (!which('cargo')) {
    warn('Rust/cargo not found. Install https://rustup.rs then: cargo build -p slate-engine')
    return
  }
  log('cargo build -p slate-engine')
  if (!run('cargo', ['build', '-p', 'slate-engine'])) {
    warn('engine build failed — studio still runs without the film factory')
  }
}

requireNode20()
console.log(`Agent-Slate setup (${platform})`)
if (!skipNpm) npmCi()
if (wantFfmpeg) installFfmpeg()
if (wantGrok) installGrok()
if (wantCursor) installCursor()
if (wantEngine) buildEngine()

console.log('')
console.log('Next:')
console.log('  npm run dev                 # Electron studio')
if (wantGrok || isFile(grokBin())) console.log('  grok login                  # Grok Build OAuth (brains + VO)')
else {
  console.log(
    isWin
      ? '  irm https://x.ai/cli/install.ps1 | iex     then  grok login'
      : '  curl -fsSL https://x.ai/cli/install.sh | bash     then  grok login'
  )
}
console.log('  cargo build -p slate-engine # optional film factory, then ◆ Agent → Connect')
console.log('  npm run package:desktop     # unpacked app in dist/ (all OS)')
if (!existsSync(join(root, 'node_modules', 'electron'))) {
  warn('electron is not installed — npm ci should have pulled it')
}
