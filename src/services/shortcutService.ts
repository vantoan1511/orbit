import { storage } from './nativeService.ts'
import { appSettingsService } from './appSettingsService.ts'
import { CONFIG_KEYS } from '../types/settings.ts'
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
 * Sanitizes raw data from storage or backend configuration into a complete and valid ShortcutBindings object.
 */
export function sanitizeBindings(raw: unknown, isMac = isClientMac): ShortcutBindings {
  const defaults = getDefaultBindingsForPlatform(isMac)

  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) {
    if (typeof raw === 'string') {
      try {
        const parsed = JSON.parse(raw)
        return sanitizeBindings(parsed, isMac)
      } catch {
        return { ...defaults }
      }
    }
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
   * Synchronize bindings with new config values received via AppSettingsUpdated.
   */
  syncFromConfig(raw: unknown, isMac = isClientMac): ShortcutBindings {
    return sanitizeBindings(raw, isMac)
  },

  /**
   * Load persisted shortcut bindings from configuration system,
   * falling back to migrating legacy storage if present.
   */
  async loadBindings(isMac = isClientMac, customValue?: unknown): Promise<ShortcutBindings> {
    if (
      customValue !== undefined &&
      customValue !== null &&
      typeof customValue === 'object' &&
      Object.keys(customValue).length > 0
    ) {
      return sanitizeBindings(customValue, isMac)
    }

    // Check for legacy storage key to perform automatic migration
    try {
      const raw = await storage.getData(SHORTCUTS_STORAGE_KEY)
      if (raw) {
        try {
          const parsed = JSON.parse(raw)
          const sanitized = sanitizeBindings(parsed, isMac)

          // Migrate to core configuration system (~/.orbit/config.json)
          await appSettingsService.updateAppSettings({
            [CONFIG_KEYS.KEYBOARD_SHORTCUTS]: sanitized
          })

          return sanitized
        } catch (parseErr) {
          console.warn('Corrupted shortcut bindings in storage, restoring defaults:', parseErr)
        }
      }
    } catch {
      // Legacy storage key absent or inaccessible
    }

    return sanitizeBindings(null, isMac)
  },

  /**
   * Persist shortcut bindings to the unified configuration system (~/.orbit/config.json).
   */
  async saveBindings(bindings: ShortcutBindings, isMac = isClientMac): Promise<void> {
    const sanitized = sanitizeBindings(bindings, isMac)

    try {
      await appSettingsService.updateAppSettings({
        [CONFIG_KEYS.KEYBOARD_SHORTCUTS]: sanitized
      })
      await storage.setData(SHORTCUTS_STORAGE_KEY, JSON.stringify(sanitized))
    } catch (err) {
      console.warn('Failed to save shortcut bindings to configuration system:', err)
    }
  },

  /**
   * Reset shortcuts to system defaults in the configuration system.
   */
  async resetBindings(isMac = isClientMac): Promise<ShortcutBindings> {
    const defaults = getDefaultBindingsForPlatform(isMac)

    try {
      await appSettingsService.updateAppSettings({
        [CONFIG_KEYS.KEYBOARD_SHORTCUTS]: {}
      })
      await storage.removeData(SHORTCUTS_STORAGE_KEY)
    } catch (err) {
      console.warn('Failed to reset shortcut bindings in configuration system:', err)
    }

    return { ...defaults }
  }
}
