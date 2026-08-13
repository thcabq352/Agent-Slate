// Reference ingestion — extract representative frames from video with the system ffmpeg.
// Frames are small JPEGs in the project cache; source media is linked, never copied.

import { execFile } from 'child_process'
import { promises as fs } from 'fs'
import { join, extname } from 'path'
import { createHash } from 'crypto'
import { cacheDir } from './projects'
import { ffmpegBin } from './ffmpeg'

const IMAGE_EXT = new Set(['.png', '.jpg', '.jpeg', '.webp', '.gif', '.tif', '.tiff', '.bmp', '.heic'])
const VIDEO_EXT = new Set(['.mp4', '.mov', '.m4v', '.webm', '.mkv', '.avi', '.mxf'])

const MAX_FRAMES = 16

function run(cmd: string, args: string[]): Promise<string> {
  return new Promise((resolve, reject) => {
    execFile(cmd, args, { timeout: 120000, maxBuffer: 32 * 1024 * 1024, windowsHide: true }, (err, stdout, stderr) => {
      if (err) reject(new Error(stderr.toString().slice(-400) || err.message))
      else resolve(stdout.toString())
    })
  })
}

export async function ffmpegAvailable(): Promise<boolean> {
  try {
    await run(ffmpegBin(), ['-version'])
    return true
  } catch {
    return false
  }
}

export function mediaKind(path: string): 'image' | 'video' | null {
  const ext = extname(path).toLowerCase()
  if (IMAGE_EXT.has(ext)) return 'image'
  if (VIDEO_EXT.has(ext)) return 'video'
  return null
}

/**
 * Extract up to MAX_FRAMES representative frames: scene-change detection first,
 * topped up with evenly spaced samples for clips with few cuts.
 */
export async function extractFrames(projectId: string, videoPath: string): Promise<string[]> {
  const hash = createHash('sha1').update(videoPath).digest('hex').slice(0, 12)
  const outDir = join(cacheDir(projectId), 'frames', hash)
  await fs.mkdir(outDir, { recursive: true })

  const existing = (await fs.readdir(outDir)).filter((f) => f.endsWith('.jpg'))
  if (existing.length > 0) return existing.map((f) => join(outDir, f)).sort()

  // Pass 1 — scene changes.
  await run(ffmpegBin(), [
    '-i', videoPath,
    '-vf', `select='gt(scene,0.28)',scale=768:-2`,
    '-vsync', 'vfr',
    '-frames:v', String(MAX_FRAMES),
    '-q:v', '4',
    join(outDir, 'cut-%03d.jpg')
  ]).catch(() => undefined)

  let frames = (await fs.readdir(outDir)).filter((f) => f.endsWith('.jpg'))

  // Pass 2 — if the clip has few hard cuts, sample evenly (one frame every 2s).
  if (frames.length < 4) {
    await run(ffmpegBin(), [
      '-i', videoPath,
      '-vf', 'fps=1/2,scale=768:-2',
      '-frames:v', String(MAX_FRAMES - frames.length),
      '-q:v', '4',
      join(outDir, 'sample-%03d.jpg')
    ]).catch(() => undefined)
    frames = (await fs.readdir(outDir)).filter((f) => f.endsWith('.jpg'))
  }

  return frames.map((f) => join(outDir, f)).sort()
}
