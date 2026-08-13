import { describe, it, expect } from 'vitest'
import {
  grokBuildCliModel,
  grokBuildHeadlessPrompt,
  grokBuildPromptFileName,
  isGrokBuildBrain,
  parseGrokBuildOutput
} from '../src/shared/grokBuild'
import { BRAIN_PICKER } from '../src/shared/types'

describe('Grok Build brain helpers', () => {
  it('treats grok-4.5 / grok-4.6 as Grok Build brains, not Composer', () => {
    expect(isGrokBuildBrain('grok-4.6')).toBe(true)
    expect(isGrokBuildBrain('grok-4.5')).toBe(true)
    expect(isGrokBuildBrain('cursor')).toBe(false)
    expect(grokBuildCliModel('grok-4.5')).toBe('grok-4.5')
    expect(grokBuildCliModel('grok-4.6')).toBe('grok-4.6')
  })

  it('labels the picker so Grok Build OAuth outranks Cursor', () => {
    const g46 = BRAIN_PICKER.find((o) => o.value === 'grok-4.6')
    const g45 = BRAIN_PICKER.find((o) => o.value === 'grok-4.5')
    const cursor = BRAIN_PICKER.find((o) => o.value === 'cursor')
    expect(g46?.label).toMatch(/Grok Build/i)
    expect(g45?.label).toMatch(/Grok Build/i)
    expect(g46?.label).toMatch(/else Cursor/i)
    expect(cursor?.label).toMatch(/Composer/)
  })

  it('parses grok --output-format json .text', () => {
    expect(
      parseGrokBuildOutput('{"text":"Scene one.","stopReason":"EndTurn","sessionId":"abc"}')
    ).toBe('Scene one.')
  })

  it('asks for grok login on auth failures, never Cursor', () => {
    expect(() => parseGrokBuildOutput('{"error":"Unauthorized. Please login."}')).toThrow(
      /grok login/
    )
    expect(() => parseGrokBuildOutput('Not authenticated. Run grok login.')).toThrow(/grok login/)
    try {
      parseGrokBuildOutput('{"error":"401 oauth revoked"}')
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      expect(msg).toMatch(/grok login/)
      expect(msg).not.toMatch(/cursor-agent/)
    }
  })

  it('points headless grok at a scratch prompt file', () => {
    expect(grokBuildPromptFileName()).toBe('slate-brain-prompt.txt')
    expect(grokBuildHeadlessPrompt()).toMatch(/slate-brain-prompt\.txt/)
  })
})
