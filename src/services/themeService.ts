import { storage } from './nativeService.ts'
import { DEFAULT_STORAGE_TIMEOUT_MS, withTimeout } from '../utils/async.ts'

export const THEME_STORAGE_KEY = 'orbit_theme_preference'
export const LEGACY_THEME_STORAGE_KEY = 'orbit-theme-preference'
export const STORAGE_TIMEOUT_MS = DEFAULT_STORAGE_TIMEOUT_MS

export const themeService = {
  /**
   * Load persisted theme preference ('dark' | 'light' | null) from native desktop storage,
   * falling back to a one-time migration from legacy localStorage if present.
   */
  async loadTheme(): Promise<'dark' | 'light' | null> {
    try {
      let raw: string | null = null

      try {
        if (typeof window !== 'undefined' && !(window as unknown as { NL_PORT?: number }).NL_PORT) {
          throw new Error('Neutralino runtime not available')
        }
        raw = await withTimeout(storage.getData(THEME_STORAGE_KEY), STORAGE_TIMEOUT_MS)
      } catch {
        // Native storage key not found or uninitialized runtime.
        // Fall back to check for legacy localStorage data
        if (typeof localStorage !== 'undefined') {
          try {
            const legacy =
              localStorage.getItem(THEME_STORAGE_KEY) ||
              localStorage.getItem(LEGACY_THEME_STORAGE_KEY)

            if (legacy && (legacy === 'dark' || legacy === 'light')) {
              raw = legacy
              // Migrate to native desktop storage and clean up legacy localStorage keys
              try {
                await storage.setData(THEME_STORAGE_KEY, legacy)
                localStorage.removeItem(THEME_STORAGE_KEY)
                localStorage.removeItem(LEGACY_THEME_STORAGE_KEY)
              } catch (migrationErr) {
                console.warn('Failed to write migrated theme to native storage:', migrationErr)
              }
            }
          } catch {
            // localStorage not accessible or restricted
          }
        }
      }

      if (raw === 'dark' || raw === 'light') {
        return raw
      }
    } catch (e) {
      console.warn('Failed to load theme preference from native storage:', e)
    }

    return null
  },

  /**
   * Persist theme preference to native desktop storage.
   */
  async saveTheme(theme: 'dark' | 'light'): Promise<void> {
    try {
      await storage.setData(THEME_STORAGE_KEY, theme)
    } catch (e) {
      console.warn('Failed to save theme preference to native storage:', e)
    }
  },

  /**
   * Remove persisted theme preference from native desktop storage.
   */
  async clearTheme(): Promise<void> {
    try {
      await storage.removeData(THEME_STORAGE_KEY)
    } catch (e) {
      console.warn('Failed to remove theme preference from native storage:', e)
    }
  }
}
