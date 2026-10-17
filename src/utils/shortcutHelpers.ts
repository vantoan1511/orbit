import type { KeyboardEventLike } from './keyboard.ts'
import type { KeyCombo, ShortcutActionId, ShortcutBindings } from '../types/shortcuts.ts'

export const MODIFIER_KEYS = new Set([
  'Control',
  'Shift',
  'Alt',
  'Meta',
  'AltGraph',
  'CapsLock',
  'Fn',
  'FnLock',
  'Hyper',
  'Super',
  'OS'
])

/**
 * Normalizes a primary key string into a standard presentation (e.g. 'p' -> 'P', 'escape' -> 'Escape')
 */
export function normalizeKey(key: string): string {
  if (!key) return ''
  const trimmed = key.trim()
  if (trimmed.length === 1) {
    return trimmed.toUpperCase()
  }

  // Canonical named keys
  const lower = trimmed.toLowerCase()
  if (lower === 'escape' || lower === 'esc') return 'Escape'
  if (lower === 'enter' || lower === 'return') return 'Enter'
  if (lower === 'tab') return 'Tab'
  if (lower === 'space' || lower === ' ') return 'Space'
  if (lower === 'backspace') return 'Backspace'
  if (lower === 'delete' || lower === 'del') return 'Delete'
  if (lower === 'arrowup' || lower === 'up') return 'Up'
  if (lower === 'arrowdown' || lower === 'down') return 'Down'
  if (lower === 'arrowleft' || lower === 'left') return 'Left'
  if (lower === 'arrowright' || lower === 'right') return 'Right'
  if (lower === 'home') return 'Home'
  if (lower === 'end') return 'End'
  if (lower === 'pageup') return 'PageUp'
  if (lower === 'pagedown') return 'PageDown'

  return trimmed
}

/**
 * Parses a key combination string (e.g. "Ctrl+Shift+P", "⌘P", "⌃Tab") into a KeyCombo object.
 */
export function parseKeyCombo(comboString: string): KeyCombo {
  const result: KeyCombo = {
    key: '',
    ctrlKey: false,
    metaKey: false,
    shiftKey: false,
    altKey: false
  }

  if (!comboString) return result

  let str = comboString.trim()

  // Detect Mac symbols
  if (str.includes('⌘')) {
    result.metaKey = true
    str = str.replace(/⌘/g, '')
  }
  if (str.includes('⌃')) {
    result.ctrlKey = true
    str = str.replace(/⌃/g, '')
  }
  if (str.includes('⌥')) {
    result.altKey = true
    str = str.replace(/⌥/g, '')
  }
  if (str.includes('⇧')) {
    result.shiftKey = true
    str = str.replace(/⇧/g, '')
  }

  // Handle plus-delimited strings (e.g. "Ctrl+Shift+P" or "Ctrl+,")
  const parts = str
    .split('+')
    .map((p) => p.trim())
    .filter(Boolean)

  for (const part of parts) {
    if (!part) continue
    const lower = part.toLowerCase()

    if (lower === 'ctrl' || lower === 'control') {
      result.ctrlKey = true
    } else if (lower === 'cmd' || lower === 'command' || lower === 'meta') {
      result.metaKey = true
    } else if (lower === 'alt' || lower === 'option') {
      result.altKey = true
    } else if (lower === 'shift') {
      result.shiftKey = true
    } else {
      result.key = normalizeKey(part)
    }
  }

  // Edge case: when the key itself was "+", split('+') might leave parts empty or missing key
  if (!result.key && comboString.endsWith('+') && comboString.length > 1) {
    result.key = '+'
  }

  return result
}

/**
 * Formats a KeyCombo into a readable presentation string, respecting the target OS.
 */
export function formatKeyCombo(combo: KeyCombo, isMac: boolean): string {
  if (!combo.key) return ''

  if (isMac) {
    let res = ''
    if (combo.metaKey) res += '⌘'
    if (combo.ctrlKey) res += '⌃'
    if (combo.altKey) res += '⌥'
    if (combo.shiftKey) res += '⇧'
    res += combo.key
    return res
  }

  const parts: string[] = []
  if (combo.ctrlKey || combo.metaKey) parts.push('Ctrl')
  if (combo.altKey) parts.push('Alt')
  if (combo.shiftKey) parts.push('Shift')
  parts.push(combo.key)

  return parts.join('+')
}

/**
 * Converts a DOM keyboard event to a normalized KeyCombo, or null if only modifier keys were pressed.
 */
export function eventToKeyCombo(event: KeyboardEventLike): KeyCombo | null {
  if (!event.key || MODIFIER_KEYS.has(event.key)) {
    return null
  }

  return {
    key: normalizeKey(event.key),
    ctrlKey: Boolean(event.ctrlKey),
    metaKey: Boolean(event.metaKey),
    shiftKey: Boolean(event.shiftKey),
    altKey: Boolean(event.altKey)
  }
}

/**
 * Compares a KeyboardEvent against a stored shortcut string.
 * On macOS, 'Ctrl+' in standard strings is mapped to Meta unless explicitly ⌃ was defined.
 */
export function isKeyComboMatch(
  event: KeyboardEventLike,
  comboString: string,
  isMac: boolean
): boolean {
  if (!comboString) return false

  const target = parseKeyCombo(comboString)
  if (!target.key) return false

  const currentKey = normalizeKey(event.key)
  if (currentKey !== target.key) return false

  const eventShift = Boolean(event.shiftKey)
  const eventAlt = Boolean(event.altKey)

  if (eventShift !== target.shiftKey) return false
  if (eventAlt !== target.altKey) return false

  if (isMac) {
    // If target specifically has ⌃ (ctrlKey) and not ⌘ (metaKey)
    if (target.ctrlKey && !target.metaKey && !comboString.includes('Ctrl+')) {
      return Boolean(event.ctrlKey) && !event.metaKey
    }
    // On Mac, standard 'Ctrl+' or '⌘' shortcuts check event.metaKey
    const targetPrimary = target.metaKey || target.ctrlKey
    const eventPrimary = Boolean(event.metaKey)
    return targetPrimary === eventPrimary
  }

  // Windows / Linux: check event.ctrlKey
  const eventCtrl = Boolean(event.ctrlKey)
  const targetCtrl = target.ctrlKey || target.metaKey
  return eventCtrl === targetCtrl
}

/**
 * Checks whether assigning a given key combo to an action conflicts with another action's binding.
 */
export function findShortcutConflict(
  actionId: ShortcutActionId,
  newComboString: string,
  currentBindings: ShortcutBindings,
  isMac: boolean
): ShortcutActionId | null {
  const normalizedNew = normalizeComboString(newComboString, isMac)
  if (!normalizedNew) return null

  for (const [id, existingBinding] of Object.entries(currentBindings)) {
    if (id === actionId) continue
    const normalizedExisting = normalizeComboString(existingBinding, isMac)
    if (normalizedExisting === normalizedNew) {
      return id as ShortcutActionId
    }
  }

  return null
}

/**
 * Normalizes a combo string into canonical representation.
 */
export function normalizeComboString(comboStr: string, isMac: boolean): string {
  const combo = parseKeyCombo(comboStr)
  return formatKeyCombo(combo, isMac)
}
