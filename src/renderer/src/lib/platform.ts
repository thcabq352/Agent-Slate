/** Help / shortcut labels. Renderer has no reliable Node `process.platform` in the browser preview. */

import { hostOs } from '../../../shared/installHints'

export { hostOs } from '../../../shared/installHints'

export function isMac(): boolean {
  return hostOs() === 'mac'
}

export function isWin(): boolean {
  return hostOs() === 'win'
}

export function isLinux(): boolean {
  return hostOs() === 'linux'
}

export function helpShortcutLabel(): string {
  return isMac() ? '⌘/' : 'Ctrl+/'
}
