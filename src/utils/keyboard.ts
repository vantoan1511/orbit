export interface KeyboardEventLike {
  key: string
  ctrlKey?: boolean
  metaKey?: boolean
  shiftKey?: boolean
  altKey?: boolean
  preventDefault?: () => void
}

/**
 * Checks if the given keyboard event represents a browser refresh command:
 * - F5 (with or without Ctrl / Shift / Meta)
 * - Ctrl+R or Cmd+R (with or without Shift)
 */
export function isBrowserRefreshShortcut(event: KeyboardEventLike): boolean {
  if (event.key === 'F5') {
    return true
  }

  if ((event.ctrlKey || event.metaKey) && (event.key === 'r' || event.key === 'R')) {
    return true
  }

  return false
}

/**
 * Prevents default browser reload behavior when a refresh shortcut is detected.
 * Returns true if the shortcut was intercepted and prevented, false otherwise.
 */
export function handlePreventBrowserRefresh(event: KeyboardEventLike): boolean {
  if (isBrowserRefreshShortcut(event)) {
    if (typeof event.preventDefault === 'function') {
      event.preventDefault()
    }
    return true
  }
  return false
}
