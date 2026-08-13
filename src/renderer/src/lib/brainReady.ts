import type { BrainStatus } from '../../../shared/types'
import { brainMissingHint, hostOs } from '../../../shared/installHints'

export function anyBrainAvailable(brain: BrainStatus | null | undefined): boolean {
  return Boolean(
    brain?.cursor.available || brain?.grok.available || brain?.codex.available || brain?.local?.available
  )
}

export type HomeBrainBanner = { kind: 'ok' | 'warn'; text: string }

/** Home copy: local Ollama/LM Studio counts as ready — never nag Cursor/Codex in that case. */
export function homeBrainBanner(brain: BrainStatus | null | undefined): HomeBrainBanner | null {
  if (!brain) return null
  if (brain.local?.available) {
    if (brain.cursor.available || brain.grok.available || brain.codex.available) return null
    const detail = brain.local.version ? ` — ${brain.local.version}` : ''
    return {
      kind: 'ok',
      text: `Local model ready${detail}. Pick “Local model” in a project’s Bible to use it.`
    }
  }
  if (brain.cursor.available || brain.grok.available || brain.codex.available) return null
  return {
    kind: 'warn',
    text: brainMissingHint(hostOs())
  }
}
