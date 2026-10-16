import { storage } from './nativeService.ts'
import { THEME_STORAGE_KEY, themeService } from './themeService.ts'
import { DEFAULT_STORAGE_TIMEOUT_MS, withTimeout } from '../utils/async.ts'
import {
  DEFAULT_APPEARANCE_PREFERENCES,
  type AccentColor,
  type AppearancePreferences,
  type ThemeMode,
  type UiStyle
} from '../types/appearance.ts'

export const APPEARANCE_STORAGE_KEY = 'orbit_appearance_preferences'
export const STORAGE_TIMEOUT_MS = DEFAULT_STORAGE_TIMEOUT_MS

const VALID_THEME_MODES: readonly ThemeMode[] = ['dark', 'light', 'system']
const VALID_ACCENT_COLORS: readonly AccentColor[] = [
  'blue',
  'emerald',
  'violet',
  'cyan',
  'amber',
  'rose',
  'slate'
]
const VALID_UI_STYLES: readonly UiStyle[] = ['flat', 'glass', 'aero', 'liquidglass', 'ios']

function sanitizePreferences(raw: unknown): AppearancePreferences {
  if (!raw || typeof raw !== 'object') {
    return { ...DEFAULT_APPEARANCE_PREFERENCES }
  }

  const obj = raw as Record<string, unknown>

  const themeMode: ThemeMode =
    typeof obj.themeMode === 'string' &&
    (VALID_THEME_MODES as readonly string[]).includes(obj.themeMode)
      ? (obj.themeMode as ThemeMode)
      : DEFAULT_APPEARANCE_PREFERENCES.themeMode

  const accentColor: AccentColor =
    typeof obj.accentColor === 'string' &&
    (VALID_ACCENT_COLORS as readonly string[]).includes(obj.accentColor)
      ? (obj.accentColor as AccentColor)
      : DEFAULT_APPEARANCE_PREFERENCES.accentColor

  const uiStyle: UiStyle =
    typeof obj.uiStyle === 'string' && (VALID_UI_STYLES as readonly string[]).includes(obj.uiStyle)
      ? (obj.uiStyle as UiStyle)
      : DEFAULT_APPEARANCE_PREFERENCES.uiStyle

  return { themeMode, accentColor, uiStyle }
}

export const appearanceService = {
  /**
   * Load persisted appearance preferences from native desktop storage,
   * falling back to migrating legacy theme preferences if absent.
   */
  async loadAppearancePreferences(): Promise<AppearancePreferences> {
    try {
      let raw: string | null = null

      try {
        raw = await withTimeout(storage.getData(APPEARANCE_STORAGE_KEY), STORAGE_TIMEOUT_MS)
      } catch {
        // Not in appearance storage yet. Check legacy theme storage.
        try {
          const legacyTheme = await themeService.loadTheme()
          if (legacyTheme === 'dark' || legacyTheme === 'light') {
            const migrated: AppearancePreferences = {
              themeMode: legacyTheme,
              accentColor: 'blue',
              uiStyle: 'flat'
            }
            try {
              await storage.setData(APPEARANCE_STORAGE_KEY, JSON.stringify(migrated))
            } catch (saveErr) {
              console.warn('Failed to save migrated appearance preferences:', saveErr)
            }
            return migrated
          }
        } catch {
          // Fallback to default
        }
      }

      if (raw) {
        try {
          const parsed = JSON.parse(raw)
          return sanitizePreferences(parsed)
        } catch (parseErr) {
          console.warn('Corrupted appearance preferences in storage:', parseErr)
        }
      }
    } catch (e) {
      console.warn('Failed to load appearance preferences from native storage:', e)
    }

    return { ...DEFAULT_APPEARANCE_PREFERENCES }
  },

  /**
   * Persist appearance preferences to native desktop storage,
   * and synchronize themeMode to legacy theme storage for backward compatibility.
   */
  async saveAppearancePreferences(preferences: AppearancePreferences): Promise<void> {
    try {
      const sanitized = sanitizePreferences(preferences)
      await storage.setData(APPEARANCE_STORAGE_KEY, JSON.stringify(sanitized))

      // Keep legacy theme key synchronized if explicit dark/light mode
      if (sanitized.themeMode === 'dark' || sanitized.themeMode === 'light') {
        try {
          await storage.setData(THEME_STORAGE_KEY, sanitized.themeMode)
        } catch {
          // Best effort sync
        }
      }
    } catch (e) {
      console.warn('Failed to save appearance preferences to native storage:', e)
    }
  },

  /**
   * Remove persisted appearance preferences from native desktop storage.
   */
  async clearAppearancePreferences(): Promise<void> {
    try {
      await storage.removeData(APPEARANCE_STORAGE_KEY)
    } catch (e) {
      console.warn('Failed to remove appearance preferences from native storage:', e)
    }
  }
}
