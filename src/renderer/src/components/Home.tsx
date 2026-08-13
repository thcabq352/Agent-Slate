import React, { useState } from 'react'
import { useProject } from '../stores/project'
import brandArt from '../assets/brand.webp'
import AboutModal from './AboutModal'
import { homeBrainBanner } from '../lib/brainReady'
import { helpShortcutLabel } from '../lib/platform'

export default function Home({ onOpenHelp }: { onOpenHelp?: () => void }): React.JSX.Element {
  const { metas, create, open, remove, brain } = useProject()
  const [name, setName] = useState('')
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null)
  const [showAbout, setShowAbout] = useState(false)

  const banner = homeBrainBanner(brain)

  const submit = (): void => {
    const n = name.trim()
    if (n) {
      void create(n)
      setName('')
    }
  }

  return (
    <div className="home">
      <div className="home-inner">
        <div className="home-hero">
          <img className="home-brand" src={brandArt} alt="Agent-Slate — prompt studio for AI filmmaking" />
          <p>
            Plan shots, direct coverage, spot your score, cast your voices, keep continuity — and
            compile production-ready prompts for any generator.
          </p>
          <div className="home-paths">
            <div className="home-path">
              <b>1. Studio</b>
              <span>Create a project, fill the Bible, write or generate shot prompts, compile, copy.</span>
            </div>
            <div className="home-path">
              <b>2. Brain</b>
              <span>
                grok login for Grok 4.5/4.6 (falls back to Cursor), cursor-agent login for Composer,
                or a local model. Test from the titlebar pill.
              </span>
            </div>
            <div className="home-path">
              <b>3. Factory</b>
              <span>Optional. ◆ Agent + ComfyUI for local takes. Not required to write prompts.</span>
            </div>
          </div>
          {banner && (
            <div className={banner.kind === 'ok' ? 'brain-ok' : 'brain-warning'}>{banner.text}</div>
          )}
        </div>

        <div className="home-new">
          <input
            placeholder="New project title — e.g. Night Market"
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && submit()}
          />
          <button className="btn btn-key" onClick={submit} disabled={!name.trim()}>
            Create Project
          </button>
        </div>

        {metas.length > 0 && (
          <div className="home-list">
            <div className="panel-title" style={{ margin: '18px 2px 8px' }}>
              Projects
            </div>
            {metas.map((m) => (
              <div
                key={m.id}
                className="home-project"
                role="button"
                tabIndex={0}
                onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && void open(m.id)}
                onClick={() => void open(m.id)}
              >
                <div className="home-project-main">
                  <div className="home-project-name">{m.name}</div>
                  {m.logline && <div className="home-project-log">{m.logline}</div>}
                </div>
                <div className="row-meta">
                  {m.sceneCount} scenes · {m.shotCount} shots
                </div>
                {confirmDelete === m.id ? (
                  <span onClick={(e) => e.stopPropagation()}>
                    <button
                      className="btn btn-sm btn-danger"
                      onClick={() => {
                        void remove(m.id)
                        setConfirmDelete(null)
                      }}
                    >
                      Delete forever
                    </button>
                    <button className="btn btn-sm btn-ghost" onClick={() => setConfirmDelete(null)}>
                      Keep
                    </button>
                  </span>
                ) : (
                  <button
                    className="btn btn-sm btn-ghost"
                    aria-label="Delete project"
                    title="Delete project"
                    onClick={(e) => {
                      e.stopPropagation()
                      setConfirmDelete(m.id)
                    }}
                  >
                    ✕
                  </button>
                )}
              </div>
            ))}
          </div>
        )}

        <div className="home-foot">
          {onOpenHelp && (
            <button className="btn btn-ghost btn-sm" onClick={onOpenHelp}>
              Help ({helpShortcutLabel()})
            </button>
          )}
          <button className="btn btn-ghost btn-sm" onClick={() => setShowAbout(true)}>
            About Agent-Slate
          </button>
        </div>
      </div>

      {showAbout && <AboutModal onClose={() => setShowAbout(false)} />}
    </div>
  )
}
