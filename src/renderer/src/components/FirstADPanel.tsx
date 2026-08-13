// First AD — optional conversational operator. Talk through what you want;
// when intent is clear it sets up scenes, shots, specs and prompts itself.
// Fully opt-in: summoned from the titlebar, never replaces the manual tools.

import React, { useEffect, useRef, useState } from 'react'
import { useProject } from '../stores/project'
import { runFirstAD, applyAdActions, type AdAction, type AdReply } from '../lib/firstAD'
import type { ChatMsg } from '../../../shared/types'

export default function FirstADPanel({ onClose }: { onClose(): void }): React.JSX.Element {
  const store = useProject()
  const project = store.project!
  const [msg, setMsg] = useState('')
  const [busy, setBusy] = useState(false)
  const scrollRef = useRef<HTMLDivElement>(null)
  const msgs: ChatMsg[] = project.copilot ?? []

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: 999999 })
  }, [msgs.length, busy])

  const push = (m: ChatMsg): void =>
    store.mutate((p) => {
      p.copilot = [...(p.copilot ?? []), m]
    })

  const send = async (): Promise<void> => {
    const text = msg.trim()
    if (!text || busy) return
    setMsg('')
    const history = msgs.map((m) => ({ role: m.role, text: m.text }))
    push({ role: 'user', text })
    setBusy(true)
    const res = await runFirstAD(project, history, text)
    setBusy(false)

    if (!res.ok || !res.json) {
      push({ role: 'assistant', text: `⚠ ${res.error ?? "The brain didn't answer — try again."}` })
      return
    }
    const parsed = res.json as AdReply
    const actions: AdAction[] = Array.isArray(parsed.actions) ? parsed.actions : []

    let receipts: string[] = []
    let focus: { sceneId: string; shotId: string | null } | null = null
    if (actions.length) {
      store.mutate((p) => {
        const result = applyAdActions(p, actions)
        receipts = result.receipts
        focus = result.focus
        p.copilot = [...(p.copilot ?? []), { role: 'assistant', text: parsed.reply || 'Done.', receipts: result.receipts }]
      })
    } else {
      push({ role: 'assistant', text: parsed.reply || '…' })
    }
    if (focus) {
      const f = focus as { sceneId: string; shotId: string | null }
      if (f.shotId) store.selectShot(f.sceneId, f.shotId)
      else store.selectScene(f.sceneId)
    }
  }

  const clear = (): void =>
    store.mutate((p) => {
      p.copilot = []
    })

  return (
    <div className="ad-panel">
      <div className="ad-head">
        <div>
          <div className="ad-title">First AD</div>
          <div className="ad-sub">Studio planner — scenes, shots, prompts. No Comfy.</div>
        </div>
        <div style={{ display: 'flex', gap: 6 }}>
          {msgs.length > 0 && (
            <button className="btn btn-ghost btn-sm" onClick={clear} title="Clear this conversation">
              Clear
            </button>
          )}
          <button className="btn btn-ghost btn-sm" onClick={onClose} title="Close (the AD keeps the transcript)">
            ✕
          </button>
        </div>
      </div>

      <div className="ad-scroll" ref={scrollRef}>
        {msgs.length === 0 && (
          <div className="ad-hint">
            <p>
              Talk the film through. When it&apos;s clear, this AD writes the paper: scenes, shots,
              specs, prompts, cast, locations. Generates live in ◆ Agent — not here.
            </p>
            <p className="ad-examples">
              <i>“I need a 90-second car chase through a night market, Seedance, 10s chunks.”</i>
              <i>“Something feels flat about scene 2 — talk me through options.”</i>
              <i>“Cast the getaway driver and give me a location for the finale.”</i>
            </p>
          </div>
        )}
        {msgs.map((m, i) => (
          <div key={i} className={`ad-msg ${m.role}`}>
            <div className="ad-msg-text">{m.text}</div>
            {m.receipts && m.receipts.length > 0 && (
              <div className="ad-receipts">
                {m.receipts.map((r, j) => (
                  <div key={j} className={`ad-receipt ${r.startsWith('✗') ? 'bad' : ''}`}>
                    {r}
                  </div>
                ))}
              </div>
            )}
          </div>
        ))}
        {busy && (
          <span className="thinking" style={{ padding: '6px 16px' }}>
            <span className="dot" />
            <span className="dot" />
            <span className="dot" />
            &nbsp;on it
          </span>
        )}
      </div>

      <div className="ad-input">
        <textarea
          rows={2}
          placeholder="What are we shooting?"
          value={msg}
          onChange={(e) => setMsg(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault()
              void send()
            }
          }}
        />
        <button className="btn btn-key" disabled={!msg.trim() || busy} onClick={() => void send()}>
          Send
        </button>
      </div>
    </div>
  )
}
