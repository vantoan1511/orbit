import { storage } from './nativeService.ts'
import {
  DEFAULT_SHORTCUT_DEFINITIONS,
  getDefaultBindingsForPlatform
} from '../constants/shortcuts.ts'
import type { ShortcutActionId, ShortcutBindings } from '../types/shortcuts.ts'
import { normalizeComboString } from '../utils/shortcutHelpers.ts'

export const SHORTCUTS_STORAGE_KEY = 'orbit_keyboard_shortcuts'

const isClientMac =
  typeof navigator !== 'undefined' &&
  Boolean(navigator.platform && navigator.platform.toUpperCase().includes('MAC'))

/**
 * Sanitizes raw data from storage into a complete and valid ShortcutBindings object.
 */
export function sanitizeBindings(raw: unknown, isMac = isClientMac): ShortcutBindings {
  const defaults = getDefaultBindingsForPlatform(isMac)

  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) {
    return { ...defaults }
  }

  const result: ShortcutBindings = { ...defaults }
  const record = raw as Record<string, unknown>

  for (const def of DEFAULT_SHORTCUT_DEFINITIONS) {
    const val = record[def.id]
    if (typeof val === 'string' && val.trim().length > 0) {
      const fallback = isMac ? def.defaultKeyMac : def.defaultKey
      result[def.id as ShortcutActionId] = normalizeComboString(val, isMac) || fallback
    }
  }

  return result
}

export const shortcutService = {
  /**
   * Load persisted shortcut bindings from native storage.
   */
  async loadBindings(isMac = isClientMac): Promise<ShortcutBindings> {
    try {
      const raw = await storage.getData(SHORTCUTS_STORAGE_KEY)
      if (raw) {
        try {
          const parsed = JSON.parse(raw)
          return sanitizeBindings(parsed, isMac)
        } catch (parseErr) {
          console.warn('Corrupted shortcut bindings in storage, restoring defaults:', parseErr)
        }
      }
    } catch (err) {
      console.warn('Failed to load shortcut bindings from storage:', err)
    }

    return sanitizeBindings(null, isMac)
  },

  /**
   * Persist shortcut bindings to native storage.
   */
  async saveBindings(bindings: ShortcutBindings, isMac = isClientMac): Promise<void> {
    try {
      const sanitized = sanitizeBindings(bindings, isMac)
      await storage.setData(SHORTCUTS_STORAGE_KEY, JSON.stringify(sanitized))
    } catch (err) {
      console.warn('Failed to save shortcut bindings to storage:', err)
    }
  },

  /**
   * Reset shortcuts to system defaults in storage.
   */
  async resetBindings(isMac = isClientMac): Promise<ShortcutBindings> {
    const defaults = getDefaultBindingsForPlatform(isMac)
    try {
      await storage.removeData(SHORTCUTS_STORAGE_KEY)
    } catch (err) {
      console.warn('Failed to clear shortcut bindings from storage:', err)
    }
    return { ...defaults }
  }
}
