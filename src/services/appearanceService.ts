import { storage } from './nativeService.ts'
import { appSettingsService } from './appSettingsService.ts'
import { THEME_STORAGE_KEY, themeService } from './themeService.ts'
import { CONFIG_KEYS } from '../types/settings.ts'
import { DEFAULT_STORAGE_TIMEOUT_MS, withTimeout } from '../utils/async.ts'
import {
  DEFAULT_APPEARANCE_PREFERENCES,
  DEFAULT_SEMANTIC_COLORS_PREFERENCES,
  type AccentColor,
  type AppearancePreferences,
  type SemanticColorsPreferences,
  type SemanticPresetId,
  type ThemeMode,
  type UiStyle
} from '../types/appearance.ts'
import { SEMANTIC_ROLE_OPTIONS } from '../utils/appearanceConstants.ts'

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
const VALID_SEMANTIC_PRESETS: readonly SemanticPresetId[] = [
  'default',
  'colorblind',
  'high-contrast',
  'custom'
]

function sanitizeSemanticColors(raw: unknown): SemanticColorsPreferences {
  if (!raw || typeof raw !== 'object') {
    return { ...DEFAULT_SEMANTIC_COLORS_PREFERENCES }
  }

  const obj = raw as Record<string, unknown>
  const preset: SemanticPresetId =
    typeof obj.preset === 'string' &&
    VALID_SEMANTIC_PRESETS.includes(obj.preset as SemanticPresetId)
      ? (obj.preset as SemanticPresetId)
      : DEFAULT_SEMANTIC_COLORS_PREFERENCES.preset

  const validSuccess = SEMANTIC_ROLE_OPTIONS.success.map((o) => o.id)
  const validWarning = SEMANTIC_ROLE_OPTIONS.warning.map((o) => o.id)
  const validDanger = SEMANTIC_ROLE_OPTIONS.danger.map((o) => o.id)
  const validInfo = SEMANTIC_ROLE_OPTIONS.info.map((o) => o.id)

  const success =
    typeof obj.success === 'string' && (validSuccess as readonly string[]).includes(obj.success)
      ? obj.success
      : DEFAULT_SEMANTIC_COLORS_PREFERENCES.success
  const warning =
    typeof obj.warning === 'string' && (validWarning as readonly string[]).includes(obj.warning)
      ? obj.warning
      : DEFAULT_SEMANTIC_COLORS_PREFERENCES.warning
  const danger =
    typeof obj.danger === 'string' && (validDanger as readonly string[]).includes(obj.danger)
      ? obj.danger
      : DEFAULT_SEMANTIC_COLORS_PREFERENCES.danger
  const info =
    typeof obj.info === 'string' && (validInfo as readonly string[]).includes(obj.info)
      ? obj.info
      : DEFAULT_SEMANTIC_COLORS_PREFERENCES.info

  return { preset, success, warning, danger, info }
}

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

  const semanticColors = sanitizeSemanticColors(obj.semanticColors)

  return { themeMode, accentColor, uiStyle, semanticColors }
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
        customValues.uiStyle !== undefined ||
        customValues.semanticColors !== undefined
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
            [CONFIG_KEYS.UI_STYLE]: sanitized.uiStyle,
            [CONFIG_KEYS.SEMANTIC_COLORS]: sanitized.semanticColors
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
        [CONFIG_KEYS.UI_STYLE]: sanitized.uiStyle,
        [CONFIG_KEYS.SEMANTIC_COLORS]: sanitized.semanticColors
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
        [CONFIG_KEYS.UI_STYLE]: DEFAULT_APPEARANCE_PREFERENCES.uiStyle,
        [CONFIG_KEYS.SEMANTIC_COLORS]: DEFAULT_APPEARANCE_PREFERENCES.semanticColors
      })
      await storage.removeData(APPEARANCE_STORAGE_KEY)
    } catch (e) {
      console.warn('Failed to remove appearance preferences from native storage:', e)
    }
  }
}
