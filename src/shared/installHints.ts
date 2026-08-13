// OS-specific install / login commands. Used by Help, Home, setup scripts, and tests.
// Keep these as the one source of truth for Windows / macOS / Linux.

export type HostOs = 'win' | 'mac' | 'linux'

export function hostOsFromPlatform(platform: string | undefined): HostOs {
  if (platform === 'win32' || platform === 'win') return 'win'
  if (platform === 'darwin' || platform === 'mac') return 'mac'
  return 'linux'
}

/** Node `process.platform` first (tests / main), then renderer `navigator`. */
export function hostOs(): HostOs {
  if (typeof process !== 'undefined' && typeof process.platform === 'string') {
    if (process.platform === 'win32' || process.platform === 'darwin' || process.platform === 'linux') {
      return hostOsFromPlatform(process.platform)
    }
  }
  if (typeof navigator !== 'undefined') {
    const p = navigator.platform || ''
    const ua = navigator.userAgent || ''
    if (/Mac/i.test(p) || /Mac OS X/i.test(ua)) return 'mac'
    if (/Win/i.test(p) || /Windows/i.test(ua)) return 'win'
  }
  return 'linux'
}

export function setupCommand(os: HostOs): string {
  if (os === 'win') {
    return 'powershell -ExecutionPolicy Bypass -File .\\install.ps1 -Grok'
  }
  return './scripts/setup.sh --grok'
}

export function grokCliInstallCommand(os: HostOs): string {
  if (os === 'win') return 'irm https://x.ai/cli/install.ps1 | iex'
  return 'curl -fsSL https://x.ai/cli/install.sh | bash'
}

export function cursorCliInstallCommand(os: HostOs): string {
  if (os === 'win') return "irm 'https://cursor.com/install?win32=true' | iex"
  return 'curl https://cursor.com/install -fsS | bash'
}

export function ffmpegInstallCommand(os: HostOs): string {
  if (os === 'win') return 'winget install Gyan.FFmpeg'
  if (os === 'mac') return 'brew install ffmpeg'
  return 'sudo apt install ffmpeg'
}

export function grokLoginHint(os: HostOs): string {
  return `Install Grok Build (${grokCliInstallCommand(os)}), then grok login. If Grok Build is missing, Agent-Slate falls back to cursor-agent login.`
}

export function brainMissingHint(os: HostOs): string {
  return `No brain found. Run ${setupCommand(os)}, then grok login — or cursor-agent login, open the ChatGPT app for Codex, or start a local server (Ollama, LM Studio, vLLM).`
}

export function ffmpegInstallHintFor(os: HostOs): string {
  if (os === 'win') {
    return `Install ffmpeg (\`${ffmpegInstallCommand(os)}\`) or set SLATE_FFMPEG to ffmpeg.exe`
  }
  return `Install ffmpeg (\`${ffmpegInstallCommand(os)}\`) or set SLATE_FFMPEG`
}
