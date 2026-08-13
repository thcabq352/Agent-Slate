// About Agent-Slate — brand art, credit, and support links. Opened from the Home
// screen button or the app menu's "About Agent-Slate" (via the about:open IPC event).

import React from 'react'
import brandArt from '../assets/brand.webp'
import pkg from '../../../../package.json'

const APP_VERSION = (pkg as { version: string }).version

export default function AboutModal({ onClose }: { onClose(): void }): React.JSX.Element {
  return (
    <div className="modal-scrim" onClick={onClose}>
      <div className="modal about-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-body" style={{ textAlign: 'center', padding: '22px 26px' }}>
          <img className="about-brand" src={brandArt} alt="Agent-Slate" />
          <p className="about-text">
            Agent-Slate is a maintained fork of Sam Wasserman&apos;s Slate, the prompt studio for
            AI filmmaking — plan shots, direct coverage, spot your score, cast your voices, and
            keep continuity across an entire film while an AI brain helps you craft every prompt.
            Compile for any generator, or run the optional local film factory (ComfyUI) so a brief
            becomes takes on disk.
          </p>
          <p className="about-meta">
            Version {APP_VERSION} · Apache-2.0
            <br />
            Created by Sam Wasserman · Maintained by thcabq352
            <br />
            Brain: Grok Build OAuth (grok login) first for Grok 4.5/4.6, else Cursor OAuth, or Codex, or a local model — no API keys.
            <br />
            <a href="https://wassermanproductions.com" target="_blank" rel="noreferrer">
              wassermanproductions.com
            </a>{' '}
            ·{' '}
            <a href="https://wasserman.ai" target="_blank" rel="noreferrer">
              wasserman.ai
            </a>{' '}
            ·{' '}
            <a href="https://github.com/thcabq352/Agent-Slate" target="_blank" rel="noreferrer">
              github.com/thcabq352/Agent-Slate
            </a>
          </p>
          <div style={{ display: 'flex', gap: 8, justifyContent: 'center' }}>
            <a
              className="btn btn-key"
              href="https://ko-fi.com/samwasserman"
              target="_blank"
              rel="noreferrer"
              style={{ textDecoration: 'none' }}
            >
              ♥ Support on Ko-fi
            </a>
            <button className="btn" onClick={onClose}>
              Close
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
