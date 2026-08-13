// Hybrid agent dock — engine, film factory, music compile, First AD, quality review.

import React, { useCallback, useEffect, useRef, useState } from 'react'
import { useProject } from '../stores/project'
import type {
  EngineHealth,
  EngineJobStatus,
  QualityVerdictView
} from '../../../shared/types'

function scoreRow(label: string, v: number): React.JSX.Element {
  const pct = Math.round(Math.max(0, Math.min(1, v)) * 100)
  return (
    <div className="engine-score-row" key={label}>
      <span>{label}</span>
      <div className="engine-score-bar">
        <i style={{ width: `${pct}%` }} />
      </div>
      <span className="engine-score-n">{v.toFixed(2)}</span>
    </div>
  )
}

type PackRow = {
  id: string
  label: string
  modality: string
  ready: boolean
  note?: string
}

type CompiledMusicView = {
  cueId: string
  name: string
  target: string
  prompt: string
  lyrics?: string
  durationSec?: number
}

type FactoryResultView = {
  ok?: boolean
  projectId?: string
  sceneId?: string
  shots?: { id?: string; name?: string; error?: string | null }[]
  receipts?: string[]
  warnings?: string[]
  elapsedMs?: number
}

export default function EngineDock({ onClose }: { onClose(): void }): React.JSX.Element {
  const { project, refreshMetas, open } = useProject()
  const [busy, setBusy] = useState(false)
  const [err, setErr] = useState<string | null>(null)
  const [msg, setMsg] = useState<string | null>(null)
  const [health, setHealth] = useState<EngineHealth | null>(null)
  const [status, setStatus] = useState<EngineJobStatus | null>(null)
  const [verdict, setVerdict] = useState<QualityVerdictView | null>(null)
  const [adInput, setAdInput] = useState('')
  const [adReply, setAdReply] = useState<string | null>(null)
  const [adReceipts, setAdReceipts] = useState<string[]>([])
  const [packs, setPacks] = useState<PackRow[]>([])
  const [brief, setBrief] = useState('')
  const [factoryPack, setFactoryPack] = useState('default-still')
  const [shotCount, setShotCount] = useState(4)
  const [factoryResult, setFactoryResult] = useState<FactoryResultView | null>(null)
  const [musicTarget, setMusicTarget] = useState<'generic' | 'suno'>('generic')
  const [compiledMusic, setCompiledMusic] = useState<CompiledMusicView[]>([])
  const [factoryWatch, setFactoryWatch] = useState(false)
  const factorySawActive = useRef(false)

  const refresh = useCallback(async (): Promise<void> => {
    try {
      const h = await window.slate.engineHealth()
      setHealth(h)
      const s = await window.slate.engineStatus()
      setStatus(s)
      try {
        const listed = (await window.slate.engineInvoke('slate_list_packs', {})) as PackRow[]
        if (Array.isArray(listed)) setPacks(listed)
      } catch {
        /* packs optional */
      }
      setErr(null)
    } catch (e) {
      setHealth(null)
      setStatus(null)
      setErr(e instanceof Error ? e.message : String(e))
    }
  }, [])

  useEffect(() => {
    void refresh()
    const t = setInterval(() => void refresh(), 2500)
    return () => clearInterval(t)
  }, [refresh])

  useEffect(() => {
    if (!factoryWatch) {
      factorySawActive.current = false
      return
    }
    if (status?.active) {
      factorySawActive.current = true
      return
    }
    if (!factorySawActive.current) return
    setFactoryWatch(false)
    const pid = status?.projectId
    if (pid) {
      void (async () => {
        await refreshMetas()
        await open(pid)
        setMsg(`Factory done — opened ${pid}`)
      })()
    } else {
      setMsg(status?.message || 'Factory stopped')
    }
  }, [factoryWatch, status?.active, status?.projectId, refreshMetas, open])

  const ensure = async (): Promise<void> => {
    setBusy(true)
    setMsg(null)
    try {
      const r = await window.slate.engineEnsure()
      setMsg(r.message)
      if (!r.ok) setErr(r.message)
      else setErr(null)
      await refresh()
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
  }

  const latestTakePath = (): string | null => {
    if (!project) return null
    for (const sc of [...project.scenes].reverse()) {
      for (const sh of [...sc.shots].reverse()) {
        for (const t of [...sh.takes].reverse()) {
          if (t.mediaPath && /\.(png|jpe?g|webp|mp4|webm|mkv|gif)$/i.test(t.mediaPath)) {
            return t.mediaPath
          }
          const n = t.notes || ''
          // notes: "path | quality:..."
          const path = n.split('|')[0]?.trim()
          if (path && /\.(png|jpe?g|webp|mp4|webm|mkv|gif)$/i.test(path)) return path
          if (path && path.includes('takes')) return path.split(' ')[0]
        }
      }
    }
    return null
  }

  const latestShotPrompt = (): string => {
    if (!project) return ''
    for (const sc of [...project.scenes].reverse()) {
      for (const sh of [...sc.shots].reverse()) {
        if (sh.prompt) return sh.prompt
      }
    }
    return ''
  }

  const judgeLatest = async (): Promise<void> => {
    const mediaPath = latestTakePath()
    if (!mediaPath) {
      setErr('No take image path found on this project. Run a generate first.')
      return
    }
    setBusy(true)
    setErr(null)
    try {
      const raw = (await window.slate.engineInvoke('slate_judge_take', {
        mediaPath,
        prompt: latestShotPrompt(),
        continuity: project?.world || ''
      })) as {
        skipped?: boolean
        skipReason?: string
        verdict?: QualityVerdictView
      }
      if (raw.skipped) {
        setMsg(raw.skipReason || 'Judge skipped')
        setVerdict(raw.verdict ?? null)
      } else {
        setVerdict(raw.verdict ?? null)
        setMsg(raw.verdict?.accept ? 'Quality PASS' : 'Quality FAIL — consider retry')
      }
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
  }

  const approve = (): void => {
    setMsg('Human approved — take kept as-is.')
  }

  const retryShot = async (): Promise<void> => {
    if (!project) return
    let shotId: string | null = null
    for (const sc of [...project.scenes].reverse()) {
      for (const sh of [...sc.shots].reverse()) {
        shotId = sh.id
        break
      }
      if (shotId) break
    }
    if (!shotId) {
      setErr('No shot to regenerate.')
      return
    }
    setBusy(true)
    setErr(null)
    try {
      const out = (await window.slate.engineInvoke('slate_generate_shot', {
        projectId: project.id,
        shotId,
        pack_id: 'default-still'
      })) as { quality?: QualityVerdictView; takePath?: string; error?: string }
      if (out.error) setErr(out.error)
      else {
        setMsg('Regenerate finished')
        if (out.quality) setVerdict(out.quality)
        await refreshMetas()
      }
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
      await refresh()
    }
  }

  const sendFirstAd = async (): Promise<void> => {
    if (!project || !adInput.trim()) return
    setBusy(true)
    setErr(null)
    try {
      const out = (await window.slate.engineInvoke('slate_first_ad', {
        projectId: project.id,
        message: adInput.trim()
      })) as {
        reply?: string
        receipts?: string[]
        scenePlanSummary?: string
        continuity?: { summaryOneLine?: string }
      }
      setAdReply(out.reply || '…')
      setAdReceipts(out.receipts || [])
      setAdInput('')
      setMsg(out.scenePlanSummary || 'First AD turn complete')
      await refresh()
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
  }

  const assembleCut = async (): Promise<void> => {
    if (!project) return
    setBusy(true)
    setErr(null)
    try {
      const out = (await window.slate.engineInvoke('slate_assemble', {
        projectId: project.id,
        circledOnly: false
      })) as { ok?: boolean; path?: string; clipCount?: number; error?: string }
      if (out.path) {
        setMsg(`Cut assembled (${out.clipCount ?? '?'} clips): ${out.path}`)
      } else {
        setErr(out.error || 'Assemble failed')
      }
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
  }

  const compileMusic = async (): Promise<void> => {
    if (!project) return
    setBusy(true)
    setErr(null)
    try {
      const out = (await window.slate.engineInvoke('slate_compile_music', {
        projectId: project.id,
        target: musicTarget
      })) as CompiledMusicView[]
      const rows = Array.isArray(out) ? out : []
      setCompiledMusic(rows)
      setMsg(
        rows.length === 0
          ? 'No music cues on this project — add one in Sound or ask the First AD.'
          : `Compiled ${rows.length} cue${rows.length === 1 ? '' : 's'} (${musicTarget})`
      )
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
  }

  const copyPrompt = async (text: string): Promise<void> => {
    try {
      await window.slate.copyText(text)
      setMsg('Copied music prompt')
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e))
    }
  }

  const runBrief = async (): Promise<void> => {
    if (!brief.trim()) {
      setErr('Write a scene brief first.')
      return
    }
    setBusy(true)
    setErr(null)
    setFactoryResult(null)
    try {
      const out = (await window.slate.engineInvoke('slate_film_factory', {
        brief: brief.trim(),
        pack_id: factoryPack,
        shot_count: shotCount,
        background: true
      })) as FactoryResultView & { started?: boolean; message?: string }
      if (out.started || out.ok) {
        setFactoryWatch(true)
        setMsg(out.message || 'Factory running — watch Engine status. Cancel stops between shots and interrupts Comfy.')
      } else {
        setFactoryResult(out)
        setErr(out.warnings?.join('; ') || 'Factory did not start')
      }
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
      await refresh()
    }
  }

  const cancel = async (): Promise<void> => {
    try {
      await window.slate.engineInvoke('slate_cancel', {})
      setMsg('Cancel requested')
      await refresh()
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e))
    }
  }

  const visionReady = health?.vision?.ready
  const comfyOk = health?.comfy?.ok

  return (
    <div className="engine-dock">
      <div className="engine-dock-head">
        <div>
          <div className="engine-dock-title">Agent dock</div>
          <div className="engine-dock-sub">Engine · Factory · Music · First AD · Quality</div>
        </div>
        <button className="btn btn-ghost btn-sm" onClick={onClose}>
          Close
        </button>
      </div>

      <div className="engine-dock-body">
        <section className="engine-section">
          <div className="engine-section-title">Engine</div>
          <div className="engine-pills">
            <span className="engine-pill" data-ok={comfyOk ? '1' : '0'}>
              Comfy {comfyOk ? 'ok' : 'down'}
            </span>
            <span className="engine-pill" data-ok={visionReady ? '1' : '0'}>
              VL {visionReady ? health?.vision?.model || 'ready' : 'not ready'}
            </span>
            <span className="engine-pill" data-ok={status?.active ? '1' : '0'}>
              {status?.active ? `busy: ${status.step}` : 'idle'}
            </span>
          </div>
          <div className="engine-actions">
            <button className="btn btn-sm" disabled={busy} onClick={() => void ensure()}>
              Connect / start engine
            </button>
            <button className="btn btn-ghost btn-sm" disabled={busy} onClick={() => void refresh()}>
              Refresh
            </button>
            <button className="btn btn-ghost btn-sm" disabled={busy} onClick={() => void cancel()}>
              Cancel job
            </button>
          </div>
          {status?.scenePlan && (
            <div className="engine-plan">
              <b>Plan</b> {status.scenePlan}
            </div>
          )}
          {status?.continuitySummary && (
            <div className="engine-plan">
              <b>Continuity</b> {status.continuitySummary}
            </div>
          )}
          {status?.message && <div className="engine-msg">{status.message}</div>}
        </section>

        <section className="engine-section">
          <div className="engine-section-title">First AD (engine)</div>
          <textarea
            className="engine-ad-input"
            rows={3}
            placeholder="Tell the AD what to set up…"
            value={adInput}
            onChange={(e) => setAdInput(e.target.value)}
            disabled={busy || !project}
          />
          <button
            className="btn btn-sm btn-key"
            disabled={busy || !project || !adInput.trim()}
            onClick={() => void sendFirstAd()}
          >
            Run First AD turn
          </button>
          {adReply && <div className="engine-ad-reply">{adReply}</div>}
          {adReceipts.length > 0 && (
            <ul className="engine-receipts">
              {adReceipts.map((r, i) => (
                <li key={i}>{r}</li>
              ))}
            </ul>
          )}
        </section>

        <section className="engine-section">
          <div className="engine-section-title">Film factory</div>
          <textarea
            className="engine-ad-input"
            rows={3}
            placeholder="One-line scene brief — e.g. a courier waits on a rainy neon rooftop, then a door opens"
            value={brief}
            onChange={(e) => setBrief(e.target.value)}
            disabled={busy}
          />
          <div className="engine-row">
            <select
              className="engine-select"
              value={factoryPack}
              onChange={(e) => setFactoryPack(e.target.value)}
              disabled={busy}
            >
              {(packs.length > 0
                ? packs
                : [
                    { id: 'default-still', label: 'default-still', modality: 'image', ready: true },
                    { id: 'default-video', label: 'default-video', modality: 'video', ready: true }
                  ]
              ).map((p) => (
                <option key={p.id} value={p.id}>
                  {p.label || p.id}
                  {p.ready === false ? ' (not ready)' : ''}
                </option>
              ))}
            </select>
            <select
              className="engine-select"
              value={shotCount}
              onChange={(e) => setShotCount(Number(e.target.value))}
              disabled={busy}
            >
              {[4, 5, 6, 7, 8].map((n) => (
                <option key={n} value={n}>
                  {n} shots
                </option>
              ))}
            </select>
          </div>
          <button
            className="btn btn-sm btn-key"
            disabled={busy || !brief.trim() || factoryWatch || !!status?.active}
            onClick={() => void runBrief()}
          >
            Run brief
          </button>
          <button
            className="btn btn-ghost btn-sm"
            disabled={busy || !project}
            onClick={() => void assembleCut()}
          >
            Assemble cut
          </button>
          {(factoryWatch || status?.active) && (
            <div className="engine-plan">
              <b>{status?.step || 'starting'}</b>
              {status?.message ? ` — ${status.message}` : ''}
              {status?.scenePlan ? ` · ${status.scenePlan}` : ''}
            </div>
          )}
          {factoryResult && (
            <div className="engine-plan">
              <b>{factoryResult.ok ? 'Done' : 'Failed'}</b>
              {typeof factoryResult.elapsedMs === 'number'
                ? ` · ${Math.round(factoryResult.elapsedMs / 1000)}s`
                : ''}
              {factoryResult.shots
                ? ` · ${factoryResult.shots.filter((s) => !s.error).length}/${factoryResult.shots.length} shots`
                : ''}
              {factoryResult.warnings && factoryResult.warnings.length > 0 && (
                <ul className="engine-receipts">
                  {factoryResult.warnings.map((w, i) => (
                    <li key={i}>{w}</li>
                  ))}
                </ul>
              )}
            </div>
          )}
        </section>

        <section className="engine-section">
          <div className="engine-section-title">Music compile</div>
          <p className="engine-msg">
            Text prompts only — no audio render. {project?.music?.length ?? 0} cue
            {(project?.music?.length ?? 0) === 1 ? '' : 's'} on this project.
          </p>
          <div className="engine-row">
            <select
              className="engine-select"
              value={musicTarget}
              onChange={(e) => setMusicTarget(e.target.value as 'generic' | 'suno')}
              disabled={busy || !project}
            >
              <option value="generic">generic</option>
              <option value="suno">suno</option>
            </select>
            <button
              className="btn btn-sm"
              disabled={busy || !project}
              onClick={() => void compileMusic()}
            >
              Compile cues
            </button>
          </div>
          {compiledMusic.map((c) => (
            <div key={c.cueId} className="engine-music-card">
              <div className="engine-verdict-head">
                {c.name} · {c.target}
                {typeof c.durationSec === 'number' ? ` · ${c.durationSec}s` : ''}
              </div>
              <pre className="engine-music-prompt">{c.prompt}</pre>
              <button className="btn btn-ghost btn-sm" onClick={() => void copyPrompt(c.prompt)}>
                Copy prompt
              </button>
            </div>
          ))}
        </section>

        <section className="engine-section">
          <div className="engine-section-title">Quality review</div>
          <div className="engine-actions">
            <button className="btn btn-sm" disabled={busy || !project} onClick={() => void judgeLatest()}>
              Judge latest take
            </button>
            <button className="btn btn-sm" disabled={busy} onClick={approve}>
              Approve
            </button>
            <button className="btn btn-sm" disabled={busy || !project} onClick={() => void retryShot()}>
              Retry shot
            </button>
          </div>
          {verdict && (
            <div className="engine-verdict" data-accept={verdict.accept ? '1' : '0'}>
              <div className="engine-verdict-head">
                {verdict.accept ? 'PASS' : 'FAIL'} · overall {verdict.overall.toFixed(2)}
                {verdict.judgeModel ? ` · ${verdict.judgeModel}` : ''}
              </div>
              {scoreRow('Visual', verdict.scores?.visualQuality ?? 0)}
              {scoreRow('Continuity', verdict.scores?.continuity ?? 0)}
              {scoreRow('Artifacts', verdict.scores?.artifacts ?? 0)}
              {scoreRow('Fidelity', verdict.scores?.promptFidelity ?? 0)}
              {verdict.summary && <p className="engine-summary">{verdict.summary}</p>}
              {verdict.issues?.length > 0 && (
                <ul className="engine-receipts">
                  {verdict.issues.map((x, i) => (
                    <li key={i}>{x}</li>
                  ))}
                </ul>
              )}
              {verdict.retryHints?.length > 0 && (
                <div className="engine-hints">
                  <b>Retry hints</b>
                  <ul>
                    {verdict.retryHints.map((x, i) => (
                      <li key={i}>{x}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}
        </section>

        {msg && <div className="engine-toast ok">{msg}</div>}
        {err && <div className="engine-toast err">{err}</div>}
      </div>
    </div>
  )
}
