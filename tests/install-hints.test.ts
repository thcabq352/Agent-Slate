import { describe, it, expect } from 'vitest'
import {
  brainMissingHint,
  cursorCliInstallCommand,
  ffmpegInstallCommand,
  ffmpegInstallHintFor,
  grokCliInstallCommand,
  hostOsFromPlatform,
  setupCommand
} from '../src/shared/installHints'

describe('install hints are explicit per OS', () => {
  it('maps process.platform to win/mac/linux', () => {
    expect(hostOsFromPlatform('win32')).toBe('win')
    expect(hostOsFromPlatform('darwin')).toBe('mac')
    expect(hostOsFromPlatform('linux')).toBe('linux')
  })

  it('Windows uses PowerShell installers, not curl | bash', () => {
    expect(setupCommand('win')).toMatch(/install\.ps1/)
    expect(setupCommand('win')).toMatch(/ExecutionPolicy Bypass/)
    expect(grokCliInstallCommand('win')).toMatch(/install\.ps1/)
    expect(cursorCliInstallCommand('win')).toMatch(/win32=true/)
    expect(ffmpegInstallCommand('win')).toMatch(/winget/)
    expect(ffmpegInstallHintFor('win')).toMatch(/ffmpeg\.exe/)
    expect(brainMissingHint('win')).toMatch(/install\.ps1/)
  })

  it('macOS and Linux share the POSIX setup script and curl installers', () => {
    for (const os of ['mac', 'linux'] as const) {
      expect(setupCommand(os)).toMatch(/setup\.sh/)
      expect(grokCliInstallCommand(os)).toMatch(/install\.sh/)
      expect(cursorCliInstallCommand(os)).toMatch(/cursor\.com\/install/)
      expect(brainMissingHint(os)).toMatch(/setup\.sh/)
    }
    expect(ffmpegInstallCommand('mac')).toMatch(/brew/)
    expect(ffmpegInstallCommand('linux')).toMatch(/apt/)
  })
})
