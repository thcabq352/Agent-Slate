// Stills Library — circled-take discovery against a fixture .ctake tree, and
// real ffmpeg extraction (window-aware) from a generated test clip.
import { describe, it, expect, beforeAll } from 'vitest'
import { mkdtempSync, mkdirSync, writeFileSync } from 'fs'
import { execFileSync } from 'child_process'
import { join } from 'path'
import { tmpdir } from 'os'
import { discoverCircledTakes, extractStills, resolveCircleTakeRecents } from '../src/main/stills'

let root: string
let clip: string

beforeAll(() => {
  root = mkdtempSync(join(tmpdir(), 'slate-stills-'))

  // A real 6s test clip with hard cuts every 2s (three colored scenes).
  clip = join(root, 'take.mp4')
  execFileSync('ffmpeg', [
    '-f', 'lavfi', '-i', 'color=red:s=320x180:d=2',
    '-f', 'lavfi', '-i', 'color=green:s=320x180:d=2',
    '-f', 'lavfi', '-i', 'color=blue:s=320x180:d=2',
    '-filter_complex', '[0][1][2]concat=n=3:v=1:a=0',
    '-y', clip
  ], { timeout: 60000 })

  // Fixture .ctake project with one circled and one uncircled take.
  const proj = join(root, 'Night Market.ctake')
  mkdirSync(proj, { recursive: true })
  writeFileSync(
    join(proj, 'project.json'),
    JSON.stringify({
      name: 'Night Market',
      shots: [{ id: 'shot-1', name: 'Rooftop Chase' }],
      takes: [
        { id: 't1', shotId: 'shot-1', mediaPath: clip, fileName: 'take.mp4', circled: true, rating: 3, inSec: 2, outSec: 6 },
        { id: 't2', shotId: 'shot-1', mediaPath: clip, fileName: 'take2.mp4', circled: false },
        { id: 't3', shotId: 'shot-1', mediaPath: join(root, 'gone.mp4'), fileName: 'gone.mp4', circled: true }
      ]
    })
  )
  writeFileSync(
    join(root, 'recents.json'),
    JSON.stringify([
      { path: proj, openedAt: '2026-08-01T00:00:00Z' },
      { path: join(root, 'vanished.ctake'), openedAt: '2026-07-01T00:00:00Z' }
    ])
  )
})

describe('discoverCircledTakes', () => {
  it('returns circled takes with shot names and select windows; drops missing media and vanished projects', async () => {
    const takes = await discoverCircledTakes({ recentsFile: join(root, 'recents.json') })
    expect(takes).toHaveLength(1)
    expect(takes[0]).toMatchObject({
      project: 'Night Market',
      shot: 'Rooftop Chase',
      fileName: 'take.mp4',
      rating: 3,
      inSec: 2,
      outSec: 6
    })
  })

  it('returns [] when recents.json is absent', async () => {
    const takes = await discoverCircledTakes({ recentsFile: join(root, 'nope.json') })
    expect(takes).toEqual([])
  })

  it('picks the first existing recents file among candidates', () => {
    const missing = join(root, 'nope.json')
    const present = join(root, 'recents.json')
    expect(resolveCircleTakeRecents([missing, present])).toBe(present)
  })
})

describe('extractStills', () => {
  it('extracts frames from a real clip', async () => {
    const frames = await extractStills(join(root, 'cache'), clip)
    expect(frames.length).toBeGreaterThanOrEqual(2)
    expect(frames[0]).toMatch(/\.jpg$/)
  })

  it('caches: second call returns the same set instantly', async () => {
    const a = await extractStills(join(root, 'cache'), clip)
    const b = await extractStills(join(root, 'cache'), clip)
    expect(b).toEqual(a)
  })

  it('honors the select window (different cache bucket)', async () => {
    const windowed = await extractStills(join(root, 'cache'), clip, 2, 6)
    const full = await extractStills(join(root, 'cache'), clip)
    expect(windowed.length).toBeGreaterThanOrEqual(1)
    expect(windowed[0]).not.toBe(full[0])
  })

  it('fails with guidance on a non-media file', async () => {
    const bad = join(root, 'recents.json')
    await expect(extractStills(join(root, 'cache'), bad)).rejects.toThrow()
  })
})
