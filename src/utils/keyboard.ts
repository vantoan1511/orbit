export interface KeyboardEventLike {
  key: string
  ctrlKey?: boolean
  metaKey?: boolean
  shiftKey?: boolean
  altKey?: boolean
  preventDefault?: () => void
}

export interface WheelEventLike {
  ctrlKey?: boolean
  metaKey?: boolean
  preventDefault?: () => void
}

export interface BrowserShortcutOptions {
  /**
   * Whether to allow developer tools shortcuts (F12, Ctrl+Shift+I, etc.).
   * Defaults to false (dev tools shortcuts are blocked unless explicitly allowed).
   */
  allowDevTools?: boolean
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

/**
 * Checks if the given event represents a browser printing shortcut:
 * - Ctrl+P / Cmd+P (Print preview)
 * - Ctrl+Shift+P / Cmd+Shift+P (System printer dialog)
 */
export function isBrowserPrintShortcut(event: KeyboardEventLike): boolean {
  const isPrimaryModifier = Boolean(event.ctrlKey || event.metaKey)
  if (!isPrimaryModifier || event.altKey) {
    return false
  }

  return event.key === 'p' || event.key === 'P'
}

/**
 * Checks if the given event represents a browser save or open command:
 * - Ctrl+S / Cmd+S (Save webpage as HTML)
 * - Ctrl+O / Cmd+O (Open HTML file)
 */
export function isBrowserSaveOrOpenShortcut(event: KeyboardEventLike): boolean {
  const isPrimaryModifier = Boolean(event.ctrlKey || event.metaKey)
  if (!isPrimaryModifier || event.altKey || event.shiftKey) {
    return false
  }

  const k = event.key.toLowerCase()
  return k === 's' || k === 'o'
}

/**
 * Checks if the given event represents a browser view-source command:
 * - Ctrl+U (Windows / Linux)
 * - Cmd+Option+U (macOS)
 */
export function isBrowserViewSourceShortcut(event: KeyboardEventLike): boolean {
  const k = event.key.toLowerCase()
  if (k !== 'u') {
    return false
  }

  // Windows / Linux: Ctrl+U (without Alt/Shift)
  if (event.ctrlKey && !event.altKey && !event.shiftKey) {
    return true
  }

  // macOS: Cmd+Option+U
  if (event.metaKey && event.altKey && !event.shiftKey) {
    return true
  }

  return false
}

/**
 * Checks if the given event represents browser window, tab, history, or bookmark navigation:
 * - Ctrl+N / Cmd+N (New window)
 * - Ctrl+Shift+N / Cmd+Shift+N (New incognito window)
 * - Ctrl+T / Cmd+T (New browser tab)
 * - Ctrl+Shift+T / Cmd+Shift+T (Reopen closed browser tab)
 * - Ctrl+D / Cmd+D (Add bookmark)
 * - Ctrl+Shift+O (Bookmark manager on Windows/Linux)
 * - Cmd+Option+B (Bookmark manager on macOS)
 * - Ctrl+H (History on Windows/Linux)
 * - Cmd+Y (History on macOS)
 * - Ctrl+J (Downloads on Windows/Linux)
 * - Cmd+Option+L (Downloads on macOS)
 * - Ctrl+Shift+Delete / Cmd+Shift+Delete (Clear browsing data)
 * - Alt+ArrowLeft / Alt+ArrowRight / Cmd+[ / Cmd+] (Webview history back/forward)
 */
export function isBrowserNavigationShortcut(event: KeyboardEventLike): boolean {
  // Alt+Arrow navigation (History Back / Forward)
  if (event.altKey && !event.ctrlKey && !event.metaKey) {
    if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
      return true
    }
  }

  // macOS Cmd+[ / Cmd+] (History Back / Forward)
  if (event.metaKey && !event.ctrlKey && !event.altKey) {
    if (event.key === '[' || event.key === ']') {
      return true
    }
  }

  const isPrimaryModifier = Boolean(event.ctrlKey || event.metaKey)
  if (!isPrimaryModifier) {
    return false
  }

  const k = event.key.toLowerCase()

  // New Window: Ctrl+N, Ctrl+Shift+N, Cmd+N, Cmd+Shift+N
  if (k === 'n' && !event.altKey) {
    return true
  }

  // New Tab / Reopen Closed Tab: Ctrl+T, Ctrl+Shift+T, Cmd+T, Cmd+Shift+T
  if (k === 't' && !event.altKey) {
    return true
  }

  // Bookmarks: Ctrl+D / Cmd+D
  if (k === 'd' && !event.altKey && !event.shiftKey) {
    return true
  }

  // Bookmark Manager: Ctrl+Shift+O (Win/Linux) or Cmd+Option+B (macOS)
  if (event.ctrlKey && event.shiftKey && k === 'o' && !event.altKey) {
    return true
  }
  if (event.metaKey && event.altKey && k === 'b' && !event.shiftKey) {
    return true
  }

  // History: Ctrl+H (Win/Linux) or Cmd+Y (macOS)
  if (event.ctrlKey && k === 'h' && !event.altKey && !event.shiftKey) {
    return true
  }
  if (event.metaKey && k === 'y' && !event.altKey && !event.shiftKey) {
    return true
  }

  // Downloads: Ctrl+J (Win/Linux) or Cmd+Option+L (macOS)
  if (event.ctrlKey && k === 'j' && !event.altKey && !event.shiftKey) {
    return true
  }
  if (event.metaKey && event.altKey && k === 'l' && !event.shiftKey) {
    return true
  }

  // Clear Browsing Data: Ctrl+Shift+Delete / Cmd+Shift+Delete / Ctrl+Shift+Backspace
  if (event.shiftKey && (event.key === 'Delete' || event.key === 'Backspace') && !event.altKey) {
    return true
  }

  return false
}

/**
 * Checks if the given event represents an accessibility or help prompt:
 * - F7 (Caret browsing prompt in Chromium / Edge WebView2)
 * - F1 (Help page launch)
 */
export function isBrowserPromptShortcut(event: KeyboardEventLike): boolean {
  return event.key === 'F7' || event.key === 'F1'
}

/**
 * Checks if the given event represents a browser zoom keyboard command:
 * - Ctrl+= / Ctrl++ / Cmd+= / Cmd++ (Zoom In)
 * - Ctrl+- / Ctrl+_ / Cmd+- / Cmd+_ (Zoom Out)
 * - Ctrl+0 / Cmd+0 (Reset Zoom)
 */
export function isBrowserZoomShortcut(event: KeyboardEventLike): boolean {
  const isPrimaryModifier = Boolean(event.ctrlKey || event.metaKey)
  if (!isPrimaryModifier || event.altKey) {
    return false
  }

  const k = event.key
  return k === '+' || k === '=' || k === '-' || k === '_' || k === '0'
}

/**
 * Checks if the given event represents a developer tools shortcut:
 * - F12
 * - Ctrl+Shift+I / Ctrl+Shift+J / Ctrl+Shift+C (Windows / Linux)
 * - Cmd+Option+I / Cmd+Option+J / Cmd+Option+C (macOS)
 */
export function isBrowserDevToolsShortcut(event: KeyboardEventLike): boolean {
  if (event.key === 'F12') {
    return true
  }

  const k = event.key.toLowerCase()
  if (k !== 'i' && k !== 'j' && k !== 'c') {
    return false
  }

  // Windows / Linux: Ctrl+Shift+I / J / C
  if (event.ctrlKey && event.shiftKey && !event.altKey) {
    return true
  }

  // macOS: Cmd+Option+I / J / C
  if (event.metaKey && event.altKey && !event.shiftKey) {
    return true
  }

  return false
}

/**
 * Aggregates all blocked browser shortcuts according to configuration options.
 */
export function isBlockedBrowserShortcut(
  event: KeyboardEventLike,
  options?: BrowserShortcutOptions
): boolean {
  if (isBrowserRefreshShortcut(event)) {
    return true
  }

  if (isBrowserPrintShortcut(event)) {
    return true
  }

  if (isBrowserSaveOrOpenShortcut(event)) {
    return true
  }

  if (isBrowserViewSourceShortcut(event)) {
    return true
  }

  if (isBrowserNavigationShortcut(event)) {
    return true
  }

  if (isBrowserPromptShortcut(event)) {
    return true
  }

  if (isBrowserZoomShortcut(event)) {
    return true
  }

  if (!options?.allowDevTools && isBrowserDevToolsShortcut(event)) {
    return true
  }

  return false
}

/**
 * Prevents default browser behavior when a blocked shortcut is detected.
 * Returns true if intercepted and preventDefault was called, false otherwise.
 */
export function handlePreventBrowserShortcuts(
  event: KeyboardEventLike,
  options?: BrowserShortcutOptions
): boolean {
  if (isBlockedBrowserShortcut(event, options)) {
    if (typeof event.preventDefault === 'function') {
      event.preventDefault()
    }
    return true
  }
  return false
}

/**
 * Prevents browser zoom on Ctrl/Cmd + Mouse Wheel.
 * Returns true if intercepted and preventDefault was called, false otherwise.
 */
export function handlePreventBrowserZoomWheel(event: WheelEventLike): boolean {
  if (event.ctrlKey || event.metaKey) {
    if (typeof event.preventDefault === 'function') {
      event.preventDefault()
    }
    return true
  }
  return false
}
