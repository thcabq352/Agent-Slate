import React, { useEffect, useState } from 'react'
import { useProject } from './stores/project'
import { brainDisplayName } from '../../shared/types'
import Navigator from './components/Navigator'
import ShotWorkspace from './components/ShotWorkspace'
import RightRail from './components/RightRail'
import FirstADPanel from './components/FirstADPanel'
import EngineDock from './components/EngineDock'
import HelpModal from './components/HelpModal'
import AboutModal from './components/AboutModal'
import Home from './components/Home'
import { helpShortcutLabel } from './lib/platform'
import { anyBrainAvailable } from './lib/brainReady'
import './styles/app.css'

export default function App(): React.JSX.Element {
  const { project, refreshMetas, refreshBrain, brain, dirty, close } = useProject()
  const [railOpen, setRailOpen] = useState(true)
  const [adOpen, setAdOpen] = useState(false)
  const [engineOpen, setEngineOpen] = useState(false)
  const [helpOpen, setHelpOpen] = useState(false)
  const [aboutOpen, setAboutOpen] = useState(false)
  const [testState, setTestState] = useState<'idle' | 'busy' | 'ok' | 'fail'>('idle')
  const [testMsg, setTestMsg] = useState<string | null>(null)

  const runBrainTest = async (): Promise<void> => {
    if (!project || testState === 'busy') return
    setTestState('busy')
    setTestMsg(null)
    const res = await window.slate.brainTest(project.defaults.brain, {
      endpoint: project.defaults.localEndpoint,
      model: project.defaults.localModel
    })
    if (res.ok && /ready/i.test(res.text)) {
      setTestState('ok')
      setTestMsg(`Brain online — replied in ${(res.elapsedMs / 1000).toFixed(1)}s`)
      setTimeout(() => {
        setTestState('idle')
        setTestMsg(null)
      }, 6000)
    } else {
      setTestState('fail')
      setTestMsg(res.error ?? `Unexpected reply: ${res.text.slice(0, 80)}`)
    }
  }

  useEffect(() => {
    void refreshMetas()
    void refreshBrain()
    const off = window.slate.onProjectsChanged(() => {
      void refreshMetas()
    })
    const offHelp = window.slate.onHelpOpen(() => setHelpOpen(true))
    const offAbout = window.slate.onAboutOpen(() => setAboutOpen(true))
    const onKey = (e: KeyboardEvent): void => {
      if (e.key === '/' && (e.ctrlKey || e.metaKey) && !e.altKey) {
        e.preventDefault()
        setHelpOpen(true)
      }
    }
    window.addEventListener('keydown', onKey)
    return () => {
      off()
      offHelp()
      offAbout()
      window.removeEventListener('keydown', onKey)
    }
  }, [refreshMetas, refreshBrain])

  const brainReady = anyBrainAvailable(brain)

  return (
    <div className="shell">
      <div className="titlebar">
        <div className="wordmark">
          <b>◆</b>&nbsp;&nbsp;Agent-Slate
        </div>
        <div className="titlebar-side">
          {dirty && <span className="save-dot" title="Saving…" />}
          <button
            className="btn btn-ghost btn-sm"
            title={`Agent-Slate Help (${helpShortcutLabel()})`}
            aria-label={`Agent-Slate Help (${helpShortcutLabel()})`}
            onClick={() => setHelpOpen(true)}
          >
            ?
          </button>
          {project && (
            <>
              <button
                className="brain-pill"
                data-ok={testState === 'fail' ? '0' : brainReady ? '1' : '0'}
                onClick={() => void runBrainTest()}
                title="Click to test the brain with a tiny live call"
              >
                {testState === 'busy'
                  ? 'Testing…'
                  : testState === 'ok'
                    ? '✓ Brain online'
                    : brainReady
                      ? `Brain: ${brainDisplayName(project.defaults.brain)} — test`
                      : 'Brain offline'}
              </button>
              <button
                className={`btn btn-sm ad-toggle ${adOpen ? 'on' : ''}`}
                onClick={() => setAdOpen((v) => !v)}
                title="First AD — studio planner. Scenes, shots, prompts, bible. Does not generate."
              >
                ✦ First AD
              </button>
              <button
                className={`btn btn-sm ad-toggle ${engineOpen ? 'on' : ''}`}
                onClick={() => setEngineOpen((v) => !v)}
                title="Agent dock — engine, Factory AD, quality, assemble"
              >
                ◆ Agent
              </button>
              <button className="btn btn-ghost btn-sm" onClick={close}>
                Close Project
              </button>
            </>
          )}
        </div>
      </div>

      {helpOpen && <HelpModal onClose={() => setHelpOpen(false)} />}
      {aboutOpen && <AboutModal onClose={() => setAboutOpen(false)} />}
      {testMsg && testState === 'fail' && (
        <div className="brain-toast" onClick={() => setTestMsg(null)}>
          ⚠ {testMsg}
        </div>
      )}
      {!project ? (
        <Home onOpenHelp={() => setHelpOpen(true)} />
      ) : (
        <div className="workspace" style={{ gridTemplateColumns: railOpen ? '240px 1fr 320px' : '240px 1fr 0px' }}>
          <div className="pane pane-left">
            <Navigator />
          </div>
          <div className="pane pane-center">
            <ShotWorkspace railOpen={railOpen} onToggleRail={() => setRailOpen((v) => !v)} />
          </div>
          <div className="pane pane-right" style={{ display: railOpen ? 'flex' : 'none' }}>
            <RightRail />
          </div>
          {adOpen && <FirstADPanel onClose={() => setAdOpen(false)} />}
          {engineOpen && <EngineDock onClose={() => setEngineOpen(false)} />}
        </div>
      )}
    </div>
  )
}
