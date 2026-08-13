import { describe, it, expect, afterEach } from 'vitest'
import { mkdtempSync, writeFileSync } from 'fs'
import { join } from 'path'
import { tmpdir } from 'os'
import { ffmpegBin, ffmpegCandidateDirs, ffmpegInstallHint, resolveFfmpegBin } from '../src/main/ffmpeg'

const tmp = mkdtempSync(join(tmpdir(), 'slate-ffmpeg-'))

afterEach(() => {
  delete process.env.SLATE_FFMPEG
})

describe('ffmpeg resolve', () => {
  it('honors SLATE_FFMPEG when the file exists', () => {
    const fake = join(tmp, process.platform === 'win32' ? 'ffmpeg.exe' : 'ffmpeg')
    writeFileSync(fake, '')
    process.env.SLATE_FFMPEG = fake
    expect(resolveFfmpegBin()).toBe(fake)
    expect(ffmpegBin()).toBe(fake)
  })

  it('includes Windows-style candidate dirs', () => {
    const dirs = ffmpegCandidateDirs().join('|').toLowerCase()
    expect(dirs).toMatch(/ffmpeg/)
  })

  it('install hint is platform-specific', () => {
    const hint = ffmpegInstallHint()
    if (process.platform === 'win32') expect(hint).toMatch(/winget/i)
    else if (process.platform === 'darwin') expect(hint).toMatch(/brew/i)
    else expect(hint).toMatch(/apt|SLATE_FFMPEG/)
  })
})
