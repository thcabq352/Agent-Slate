import { describe, it, expect } from 'vitest'
import { anyBrainAvailable, homeBrainBanner } from '../src/renderer/src/lib/brainReady'
import { helpShortcutLabel, isMac } from '../src/renderer/src/lib/platform'
import type { BrainStatus } from '../src/shared/types'

const empty: BrainStatus = {
  cursor: { available: false, version: null },
  grok: { available: false, version: null },
  codex: { available: false, version: null },
  local: { available: false, version: null, endpoint: null }
}

describe('homeBrainBanner', () => {
  it('is silent until status arrives', () => {
    expect(homeBrainBanner(null)).toBeNull()
  })

  it('treats a local Ollama/LM Studio brain as success (no Cursor nag)', () => {
    const brain: BrainStatus = {
      ...empty,
      local: { available: true, version: '3 model(s) @ 127.0.0.1:11434', endpoint: 'http://127.0.0.1:11434/v1' }
    }
    expect(anyBrainAvailable(brain)).toBe(true)
    const banner = homeBrainBanner(brain)
    expect(banner?.kind).toBe('ok')
    expect(banner?.text).toMatch(/Local model ready/)
    expect(banner?.text).not.toMatch(/cursor-agent login/)
  })

  it('stays quiet when Cursor, Grok Build, or Codex is up', () => {
    expect(
      homeBrainBanner({
        ...empty,
        cursor: { available: true, version: '1.0' }
      })
    ).toBeNull()
    expect(
      homeBrainBanner({
        ...empty,
        grok: { available: true, version: 'grok 1.0' }
      })
    ).toBeNull()
  })

  it('warns only when nothing is available, and mentions local servers', () => {
    const banner = homeBrainBanner(empty)
    expect(banner?.kind).toBe('warn')
    expect(banner?.text).toMatch(/Ollama/)
    expect(banner?.text).toMatch(/grok login/)
    expect(banner?.text).toMatch(/install\.ps1|setup\.sh/)
  })
})

describe('helpShortcutLabel', () => {
  it('is Ctrl+/ off macOS (Windows/Linux)', () => {
    if (isMac()) expect(helpShortcutLabel()).toBe('⌘/')
    else expect(helpShortcutLabel()).toBe('Ctrl+/')
  })
})
