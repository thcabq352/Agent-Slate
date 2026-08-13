// References — bring in stills and clips, break them down into element sheets.
// Also home of the Stills Library: harvest continuity stills from circled
// takes (Circle Take dailies) or any clip, and file them onto cast/location/
// look sheets so the brain keeps writing from what your film actually looks like.

import React, { useState } from 'react'
import { useProject, uid } from '../stores/project'
import { analyzeReference, elementSheetToSetups } from '../lib/brainTasks'
import type { CircledTake, ElementSheet, Reference, SectionId } from '../../../shared/types'

export default function ReferencesPanel(): React.JSX.Element {
  const store = useProject()
  const project = store.project!
  const [busy, setBusy] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  const addRefs = async (): Promise<void> => {
    const paths = await window.slate.pickMedia()
    for (const path of paths) {
      setBusy(path)
      setError(null)
      try {
        const { kind, frames } = await window.slate.ingestMedia(project.id, path)
        store.addReference({
          id: uid('ref'),
          path,
          kind,
          label: path.split('/').pop() ?? path,
          frames,
          elements: null,
          addedAt: new Date().toISOString()
        })
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e))
      }
    }
    setBusy(null)
  }

  const analyze = async (ref: Reference): Promise<void> => {
    setBusy(ref.id)
    setError(null)
    const res = await analyzeReference(project, ref.frames, ref.kind)
    setBusy(null)
    if (res.ok && res.json) {
      store.mutate((p) => {
        const r = p.references.find((x) => x.id === ref.id)
        if (r) r.elements = res.json as ElementSheet
      })
    } else setError(res.error ?? 'Analysis failed — is the brain signed in?')
  }

  const saveElements = (ref: Reference): void => {
    if (!ref.elements) return
    for (const s of elementSheetToSetups(ref.elements)) {
      store.upsertSetup({
        id: uid('setup'),
        label: `${s.label} (${ref.label.slice(0, 18)})`,
        snippet: s.snippet,
        section: s.section as SectionId,
        tags: ['reference'],
        favorite: false
      })
    }
  }

  return (
    <div className="scroll">
      <div style={{ padding: 10 }}>
        <button className="btn btn-sm" style={{ width: '100%' }} onClick={() => void addRefs()} disabled={!!busy}>
          + Add images or clips
        </button>
        <div style={{ fontSize: 11.5, color: 'var(--ink-3)', marginTop: 6 }}>
          Clips are broken into key frames locally (ffmpeg). Media stays where it lives — Agent-Slate links
          it, never copies it.
        </div>
        {error && <div style={{ color: 'var(--danger)', fontSize: 12, marginTop: 6 }}>{error}</div>}
      </div>

      {project.references.length === 0 && (
        <div className="empty" style={{ height: 'auto', padding: '30px 20px' }}>
          <p>Drop in a frame from a film you love or a take you generated — the brain breaks down its lensing, light, palette and movement into elements you can prompt with.</p>
        </div>
      )}

      <StillsLibrary />

      {project.references.map((ref) => (
        <div key={ref.id} className="card">
          <div style={{ display: 'flex', alignItems: 'baseline', gap: 8 }}>
            <b style={{ color: 'var(--ink-0)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: 160 }}>
              {ref.label}
            </b>
            <span className="row-meta">
              {ref.kind}
              {ref.kind === 'video' ? ` · ${ref.frames.length} frames` : ''}
            </span>
          </div>

          {ref.frames.length > 0 && (
            <div className="ref-frames">
              {ref.frames.slice(0, 6).map((f) => (
                <img key={f} src={`file://${f}`} alt="" />
              ))}
            </div>
          )}

          {ref.elements ? (
            <details style={{ fontSize: 12, color: 'var(--ink-2)', margin: '6px 0' }} open>
              <summary style={{ cursor: 'pointer', color: 'var(--key)' }}>Element sheet</summary>
              {(['lensing', 'lighting', 'palette', 'composition', 'movement', 'texture', 'mood', 'notes'] as const).map((k) => (
                <p key={k} style={{ margin: '5px 0' }}>
                  <b style={{ color: 'var(--ink-1)', textTransform: 'capitalize' }}>{k}:</b> {ref.elements![k]}
                </p>
              ))}
            </details>
          ) : null}

          <div style={{ display: 'flex', gap: 6, marginTop: 6 }}>
            {!ref.elements && (
              <button className="btn btn-sm btn-key" disabled={busy === ref.id} onClick={() => void analyze(ref)}>
                {busy === ref.id ? 'Breaking down…' : 'Break Down'}
              </button>
            )}
            {ref.elements && (
              <button className="btn btn-sm" onClick={() => saveElements(ref)} title="Save each element as a My Setup for one-click reuse">
                Save Elements as Setups
              </button>
            )}
            <button className="btn btn-sm btn-ghost btn-danger" onClick={() => store.removeReference(ref.id)}>
              ✕
            </button>
          </div>
        </div>
      ))}
    </div>
  )
}

// ---------------- Stills Library ----------------

interface StillSource {
  key: string
  label: string
  sub: string
  mediaPath: string
  inSec: number | null
  outSec: number | null
}

function StillsLibrary(): React.JSX.Element {
  const store = useProject()
  const project = store.project!
  const [sources, setSources] = useState<StillSource[]>([])
  const [scanned, setScanned] = useState(false)
  const [frames, setFrames] = useState<Record<string, string[]>>({})
  const [busy, setBusy] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  const scanDailies = async (): Promise<void> => {
    setBusy('scan')
    setError(null)
    try {
      const takes: CircledTake[] = await window.slate.stillsDiscover()
      const rows = takes.map((t) => ({
        key: `${t.mediaPath}:${t.inSec ?? ''}`,
        label: t.shot ? `${t.shot} — ${t.fileName}` : t.fileName,
        sub: `${t.project}${t.rating ? ` · ${'●'.repeat(t.rating)}` : ''}${t.inSec != null ? ` · select ${t.inSec}–${t.outSec ?? '…'}s` : ''}`,
        mediaPath: t.mediaPath,
        inSec: t.inSec,
        outSec: t.outSec
      }))
      setSources((prev) => {
        const have = new Set(prev.map((s) => s.key))
        return [...prev, ...rows.filter((r) => !have.has(r.key))]
      })
      setScanned(true)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    }
    setBusy(null)
  }

  const addClips = async (): Promise<void> => {
    const paths = await window.slate.pickMedia()
    setSources((prev) => {
      const have = new Set(prev.map((s) => s.key))
      const rows = paths
        .filter((p) => !have.has(p))
        .map((p) => ({
          key: p,
          label: p.split('/').pop() ?? p,
          sub: 'clip',
          mediaPath: p,
          inSec: null as number | null,
          outSec: null as number | null
        }))
      return [...prev, ...rows]
    })
    setScanned(true)
  }

  const extract = async (s: StillSource): Promise<void> => {
    setBusy(s.key)
    setError(null)
    try {
      const out = await window.slate.stillsExtract(project.id, s.mediaPath, s.inSec, s.outSec)
      setFrames((f) => ({ ...f, [s.key]: out }))
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    }
    setBusy(null)
  }

  return (
    <div className="card" style={{ marginTop: 2 }}>
      <div style={{ display: 'flex', alignItems: 'baseline', gap: 8 }}>
        <b style={{ color: 'var(--ink-0)' }}>Stills Library</b>
        <span className="row-meta">continuity stills from your takes</span>
      </div>
      <div style={{ fontSize: 11.5, color: 'var(--ink-3)', margin: '4px 0 8px' }}>
        Pull frames from circled takes in your dailies, or from any clip, and pin them to a
        character, location, or look. Sheets with stills stay visually consistent — the brain
        sees them whenever it writes that subject.
      </div>
      <div style={{ display: 'flex', gap: 6 }}>
        <button className="btn btn-sm btn-key" disabled={busy === 'scan'} onClick={() => void scanDailies()}>
          {busy === 'scan' ? 'Scanning…' : '◉ Scan dailies'}
        </button>
        <button className="btn btn-sm" onClick={() => void addClips()}>
          + From clips…
        </button>
      </div>
      {error && <div style={{ color: 'var(--danger)', fontSize: 12, marginTop: 6 }}>{error}</div>}
      {scanned && sources.length === 0 && (
        <div style={{ fontSize: 12, color: 'var(--ink-3)', marginTop: 8 }}>
          No circled takes found — log a keeper with ⭕ Circle in Deliver, use Circle Take dailies
          if installed, or add clips directly.
        </div>
      )}

      {sources.map((s) => (
        <div key={s.key} style={{ marginTop: 10 }}>
          <div style={{ display: 'flex', alignItems: 'baseline', gap: 8 }}>
            <b style={{ fontSize: 12.5, color: 'var(--ink-1)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: 170 }}>
              {s.label}
            </b>
            <span className="row-meta">{s.sub}</span>
            {!frames[s.key] && (
              <button className="btn btn-sm" style={{ marginLeft: 'auto' }} disabled={busy === s.key} onClick={() => void extract(s)}>
                {busy === s.key ? 'Extracting…' : 'Extract stills'}
              </button>
            )}
          </div>
          {frames[s.key] && <StillGrid paths={frames[s.key]} />}
        </div>
      ))}
    </div>
  )
}

function StillGrid({ paths }: { paths: string[] }): React.JSX.Element {
  const store = useProject()
  const project = store.project!
  const [saved, setSaved] = useState<string | null>(null)

  const targets = [
    ...project.characters.map((c) => ({ id: `char:${c.id}`, label: `Cast · ${c.name}` })),
    ...project.locations.map((l) => ({ id: `loc:${l.id}`, label: `Location · ${l.name}` })),
    ...project.lookbook.map((l) => ({ id: `look:${l.id}`, label: `Look · ${l.source}` })),
    { id: 'refs', label: 'References' }
  ]

  const assign = (path: string, target: string): void => {
    if (target === 'refs') {
      store.addReference({
        id: uid('ref'),
        path,
        kind: 'image',
        label: `still · ${path.split('/').pop()}`,
        frames: [path],
        elements: null,
        addedAt: new Date().toISOString()
      })
    } else {
      const [kind, id] = target.split(':')
      store.mutate((p) => {
        const sheet =
          kind === 'char'
            ? p.characters.find((x) => x.id === id)
            : kind === 'loc'
              ? p.locations.find((x) => x.id === id)
              : p.lookbook.find((x) => x.id === id)
        if (sheet) {
          sheet.images = sheet.images ?? []
          if (!sheet.images.includes(path)) sheet.images.push(path)
        }
      })
    }
    setSaved(path + target)
    setTimeout(() => setSaved(null), 1600)
  }

  return (
    <div className="still-grid">
      {paths.map((f) => (
        <div key={f} className="still-cell">
          <img src={`file://${f}`} alt="" />
          <select
            defaultValue=""
            onChange={(e) => {
              if (e.target.value) assign(f, e.target.value)
              e.target.value = ''
            }}
          >
            <option value="" disabled>
              {saved?.startsWith(f) ? '✓ pinned' : 'pin to…'}
            </option>
            {targets.map((t) => (
              <option key={t.id} value={t.id}>
                {t.label}
              </option>
            ))}
          </select>
        </div>
      ))}
    </div>
  )
}
