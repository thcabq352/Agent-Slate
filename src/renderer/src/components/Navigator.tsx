import React, { useEffect, useState } from 'react'
import { useProject } from '../stores/project'
import { BRAIN_PICKER, type BrainBackend } from '../../../shared/types'

export default function Navigator(): React.JSX.Element {
  const store = useProject()
  const { project, sceneId, shotId } = store
  const [newScene, setNewScene] = useState(false)
  const [sceneName, setSceneName] = useState('')

  if (!project) return <></>

  const addScene = (): void => {
    const n = sceneName.trim()
    if (n) store.addScene(n)
    setSceneName('')
    setNewScene(false)
  }

  return (
    <>
      <div className="panel-head">
        <span className="panel-title">{project.name}</span>
        <button className="btn btn-ghost btn-sm" title="New scene" onClick={() => setNewScene(true)}>
          + Scene
        </button>
      </div>
      <div className="scroll">
        {newScene && (
          <div style={{ padding: '0 10px 8px' }}>
            <input
              autoFocus
              placeholder="Scene name…"
              value={sceneName}
              style={{ width: '100%' }}
              onChange={(e) => setSceneName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') addScene()
                if (e.key === 'Escape') setNewScene(false)
              }}
              onBlur={() => (sceneName.trim() ? addScene() : setNewScene(false))}
            />
          </div>
        )}
        {project.scenes.length === 0 && !newScene && (
          <div className="empty" style={{ height: 'auto', padding: '40px 20px' }}>
            <div className="glyph">🎬</div>
            <p>No scenes yet. Add a scene, then build shots inside it — or ask for coverage and let the brain build them for you.</p>
          </div>
        )}
        {project.scenes.map((scene) => (
          <div key={scene.id} className="nav-scene">
            <div
              className={`row ${sceneId === scene.id && !shotId ? 'active' : ''}`}
              role="button"
              tabIndex={0}
              onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && store.selectScene(scene.id)}
              onClick={() => store.selectScene(scene.id)}
            >
              <span className="nav-scene-glyph">▸</span>
              <span className="row-label" style={{ fontWeight: 600 }}>
                {scene.name}
              </span>
              <span className="row-meta">{scene.shots.length}</span>
            </div>
            {sceneId === scene.id && (
              <div className="nav-shots">
                {scene.shots.map((shot) => (
                  <div
                    key={shot.id}
                    className={`row ${shotId === shot.id ? 'active' : ''}`}
                    role="button"
                    tabIndex={0}
                    onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && store.selectShot(scene.id, shot.id)}
                    onClick={() => store.selectShot(scene.id, shot.id)}
                  >
                    <span className="row-label">{shot.name}</span>
                    <span className="row-meta">
                      {shot.spec.durationSec ? `${shot.spec.durationSec}s` : ''}
                    </span>
                  </div>
                ))}
                <div
                  className="row nav-add"
                  role="button"
                  tabIndex={0}
                  onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && store.addShot(scene.id)}
                  onClick={() => store.addShot(scene.id)}
                >
                  <span className="row-label" style={{ color: 'var(--ink-3)' }}>
                    + Add shot
                  </span>
                </div>
              </div>
            )}
          </div>
        ))}
      </div>
      <ProjectMetaEditor />
    </>
  )
}

function ProjectMetaEditor(): React.JSX.Element {
  const store = useProject()
  const p = store.project!
  const [open, setOpen] = useState(false)

  return (
    <div className="nav-foot">
      <button className="btn btn-ghost btn-sm" style={{ width: '100%' }} onClick={() => setOpen(!open)}>
        {open ? '▾ Project Bible' : '▸ Project Bible'}
      </button>
      {open && (
        <div style={{ padding: '8px 10px 12px' }}>
          <div className="field">
            <label>Logline</label>
            <textarea
              rows={2}
              value={p.logline}
              onChange={(e) => store.mutate((proj) => void (proj.logline = e.target.value))}
              placeholder="One sentence — what is this film?"
            />
          </div>
          <div className="field">
            <label>World &amp; Tone</label>
            <textarea
              rows={3}
              value={p.world}
              onChange={(e) => store.mutate((proj) => void (proj.world = e.target.value))}
              placeholder="Era, place, rules, texture, overall look…"
            />
          </div>
          <div className="grid-2">
            <div className="field">
              <label>Default AR</label>
              <select
                value={p.defaults.aspectRatio}
                onChange={(e) => store.mutate((proj) => void (proj.defaults.aspectRatio = e.target.value))}
              >
                {['16:9', '2.39:1', '1.85:1', '9:16', '4:5', '1:1', '4:3'].map((r) => (
                  <option key={r}>{r}</option>
                ))}
              </select>
            </div>
            <div className="field">
              <label>Default Length</label>
              <input
                type="number"
                min={1}
                max={600}
                value={p.defaults.durationSec}
                onChange={(e) =>
                  store.mutate((proj) => void (proj.defaults.durationSec = Number(e.target.value) || 8))
                }
              />
            </div>
          </div>
          <div className="field">
            <label>Brain</label>
            <select
              value={p.defaults.brain}
              onChange={(e) =>
                store.mutate((proj) => void (proj.defaults.brain = e.target.value as BrainBackend))
              }
            >
              {BRAIN_PICKER.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
          {p.defaults.brain === 'local' && <LocalBrainFields />}
        </div>
      )}
    </div>
  )
}

function LocalBrainFields(): React.JSX.Element {
  const store = useProject()
  const p = store.project!
  const [detected, setDetected] = useState<string | null>(null)
  const [models, setModels] = useState<string[]>([])

  useEffect(() => {
    let live = true
    void window.slate.localModels(p.defaults.localEndpoint || undefined).then((res) => {
      if (!live) return
      setDetected(res.endpoint)
      setModels(res.models.map((m) => m.id))
    })
    return () => {
      live = false
    }
  }, [p.defaults.localEndpoint])

  return (
    <>
      <div className="field">
        <label>Local server</label>
        <input
          value={p.defaults.localEndpoint ?? ''}
          placeholder={detected ? `auto — found ${detected.replace(/^https?:\/\//, '')}` : 'auto — Ollama · LM Studio · vLLM · llama.cpp'}
          onChange={(e) =>
            store.mutate((proj) => void (proj.defaults.localEndpoint = e.target.value.trim()))
          }
        />
      </div>
      <div className="field">
        <label>Local model</label>
        {models.length > 0 ? (
          <select
            value={p.defaults.localModel || models[0]}
            onChange={(e) => store.mutate((proj) => void (proj.defaults.localModel = e.target.value))}
          >
            {models.map((m) => (
              <option key={m} value={m}>
                {m}
              </option>
            ))}
          </select>
        ) : (
          <input
            value={p.defaults.localModel ?? ''}
            placeholder="no server found — model id"
            onChange={(e) =>
              store.mutate((proj) => void (proj.defaults.localModel = e.target.value.trim()))
            }
          />
        )}
      </div>
    </>
  )
}
