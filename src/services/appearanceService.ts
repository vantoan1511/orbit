import { storage } from './nativeService.ts'
import { appSettingsService } from './appSettingsService.ts'
import { THEME_STORAGE_KEY, themeService } from './themeService.ts'
import { CONFIG_KEYS } from '../types/settings.ts'
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
   * Load persisted appearance preferences from configuration system or storage,
   * falling back to migrating legacy theme preferences if absent.
   */
  async loadAppearancePreferences(
    customValues?: Partial<AppearancePreferences>
  ): Promise<AppearancePreferences> {
    if (customValues && typeof customValues === 'object') {
      const sanitized = sanitizePreferences(customValues)
      if (
        customValues.themeMode !== undefined ||
        customValues.accentColor !== undefined ||
        customValues.uiStyle !== undefined
      ) {
        return sanitized
      }
    }

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
              await appSettingsService.updateAppSettings({
                [CONFIG_KEYS.THEME_MODE]: migrated.themeMode,
                [CONFIG_KEYS.ACCENT_COLOR]: migrated.accentColor,
                [CONFIG_KEYS.UI_STYLE]: migrated.uiStyle
              })
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
          const sanitized = sanitizePreferences(parsed)
          // Ensure synchronized with configuration system
          await appSettingsService.updateAppSettings({
            [CONFIG_KEYS.THEME_MODE]: sanitized.themeMode,
            [CONFIG_KEYS.ACCENT_COLOR]: sanitized.accentColor,
            [CONFIG_KEYS.UI_STYLE]: sanitized.uiStyle
          })
          return sanitized
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
   * Persist appearance preferences to unified configuration system (~/.orbit/config.json),
   * and synchronize themeMode to legacy theme storage for backward compatibility.
   */
  async saveAppearancePreferences(preferences: AppearancePreferences): Promise<void> {
    try {
      const sanitized = sanitizePreferences(preferences)

      // Dispatch to core configuration system (~/.orbit/config.json)
      await appSettingsService.updateAppSettings({
        [CONFIG_KEYS.THEME_MODE]: sanitized.themeMode,
        [CONFIG_KEYS.ACCENT_COLOR]: sanitized.accentColor,
        [CONFIG_KEYS.UI_STYLE]: sanitized.uiStyle
      })

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
      console.warn('Failed to save appearance preferences to configuration system:', e)
    }
  },

  /**
   * Reset appearance preferences to system defaults in configuration system.
   */
  async clearAppearancePreferences(): Promise<void> {
    try {
      await appSettingsService.updateAppSettings({
        [CONFIG_KEYS.THEME_MODE]: DEFAULT_APPEARANCE_PREFERENCES.themeMode,
        [CONFIG_KEYS.ACCENT_COLOR]: DEFAULT_APPEARANCE_PREFERENCES.accentColor,
        [CONFIG_KEYS.UI_STYLE]: DEFAULT_APPEARANCE_PREFERENCES.uiStyle
      })
      await storage.removeData(APPEARANCE_STORAGE_KEY)
    } catch (e) {
      console.warn('Failed to remove appearance preferences from native storage:', e)
    }
  }
}
