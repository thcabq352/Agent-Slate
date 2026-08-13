#!/usr/bin/env node
// Zip skills/slate-film-factory → share/slate-film-factory.zip (store method, no extra deps).
import { mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'fs'
import { dirname, join, relative, resolve } from 'path'
import { fileURLToPath } from 'url'
import { crc32 } from 'zlib'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const srcDir = join(root, 'skills', 'slate-film-factory')
const outDir = join(root, 'share')
const outFile = join(outDir, 'slate-film-factory.zip')

function collect(dir, base = dir) {
  const files = []
  for (const name of readdirSync(dir)) {
    if (name.startsWith('.')) continue
    const p = join(dir, name)
    const st = statSync(p)
    if (st.isDirectory()) files.push(...collect(p, base))
    else files.push({ name: relative(base, p).replaceAll('\\', '/'), data: readFileSync(p) })
  }
  return files.sort((a, b) => a.name.localeCompare(b.name))
}

function u16(n) {
  const b = Buffer.alloc(2)
  b.writeUInt16LE(n)
  return b
}

function u32(n) {
  const b = Buffer.alloc(4)
  b.writeUInt32LE(n)
  return b
}

const files = collect(srcDir).map((f) => ({
  ...f,
  name: `slate-film-factory/${f.name}`
}))
if (files.length === 0) {
  console.error(`No files in ${srcDir}`)
  process.exit(1)
}

mkdirSync(outDir, { recursive: true })
const chunks = []
const central = []
let offset = 0
for (const f of files) {
  const name = Buffer.from(f.name, 'utf8')
  const crc = crc32(f.data)
  const local = Buffer.concat([
    Buffer.from('PK\x03\x04'),
    u16(20),
    u16(0),
    u16(0),
    u16(0),
    u16(0),
    u32(crc),
    u32(f.data.length),
    u32(f.data.length),
    u16(name.length),
    u16(0),
    name,
    f.data
  ])
  chunks.push(local)
  central.push(
    Buffer.concat([
      Buffer.from('PK\x01\x02'),
      u16(20),
      u16(20),
      u16(0),
      u16(0),
      u16(0),
      u16(0),
      u32(crc),
      u32(f.data.length),
      u32(f.data.length),
      u16(name.length),
      u16(0),
      u16(0),
      u16(0),
      u16(0),
      u32(0),
      u32(offset),
      name
    ])
  )
  offset += local.length
}
const centralBuf = Buffer.concat(central)
const end = Buffer.concat([
  Buffer.from('PK\x05\x06'),
  u16(0),
  u16(0),
  u16(files.length),
  u16(files.length),
  u32(centralBuf.length),
  u32(offset),
  u16(0)
])
writeFileSync(outFile, Buffer.concat([...chunks, centralBuf, end]))
console.log(`→ ${relative(root, outFile).replaceAll('\\', '/')} (${files.length} files)`)
