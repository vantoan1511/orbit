import { computed, ref } from 'vue'
import { appearanceService } from '../services/appearanceService.ts'
import { CONFIG_KEYS, type Configuration } from '../types/settings.ts'
import {
  DEFAULT_APPEARANCE_PREFERENCES,
  DEFAULT_SEMANTIC_COLORS_PREFERENCES,
  type AccentColor,
  type AppearancePreferences,
  type SemanticColorRole,
  type SemanticColorsPreferences,
  type SemanticPresetId,
  type ThemeMode,
  type UiStyle
} from '../types/appearance.ts'
import { SEMANTIC_COLOR_PRESETS } from '../utils/appearanceConstants.ts'
import { syncAppearanceToDom } from '../utils/appearanceTheme.ts'

const mediaQuery =
  typeof window !== 'undefined' && window.matchMedia
    ? window.matchMedia('(prefers-color-scheme: dark)')
    : null

const getSystemPrefersDark = (): boolean => {
  if (typeof window === 'undefined') return false
  return mediaQuery ? mediaQuery.matches : false
}

const systemIsDark = ref(getSystemPrefersDark())

// Shared global state across components
const preferences = ref<AppearancePreferences>({ ...DEFAULT_APPEARANCE_PREFERENCES })

const resolvedTheme = computed<'dark' | 'light'>(() => {
  if (preferences.value.themeMode === 'system') {
    return systemIsDark.value ? 'dark' : 'light'
  }
  return preferences.value.themeMode
})

const isDark = computed<boolean>(() => resolvedTheme.value === 'dark')
const themeMode = computed<ThemeMode>(() => preferences.value.themeMode)
const accentColor = computed<AccentColor>(() => preferences.value.accentColor)
const uiStyle = computed<UiStyle>(() => preferences.value.uiStyle)
const semanticColors = computed<SemanticColorsPreferences>(
  () => preferences.value.semanticColors ?? DEFAULT_SEMANTIC_COLORS_PREFERENCES
)

export function applyAppearanceToDom(): void {
  syncAppearanceToDom({
    themeMode: preferences.value.themeMode,
    resolvedTheme: resolvedTheme.value,
    accentColor: preferences.value.accentColor,
    uiStyle: preferences.value.uiStyle,
    semanticColors: preferences.value.semanticColors
  })
}

// Apply initial appearance upon module evaluation
applyAppearanceToDom()

// Listen for OS theme changes
if (mediaQuery) {
  mediaQuery.addEventListener('change', (e: MediaQueryListEvent) => {
    systemIsDark.value = e.matches
    if (preferences.value.themeMode === 'system') {
      applyAppearanceToDom()
    }
  })
}

/**
 * Initialize appearance preferences from native desktop storage.
 */
export async function initAppearance(): Promise<void> {
  try {
    const loaded = await appearanceService.loadAppearancePreferences()
    preferences.value = loaded
    applyAppearanceToDom()
  } catch (e) {
    console.warn('Failed to initialize appearance preferences:', e)
  }
}

/**
 * Synchronize appearance preferences from Configuration items received via backend IPC.
 */
export function syncAppearanceFromSettings(settings: Configuration[]): void {
  const themeModeConfig = settings.find((c) => c.key === CONFIG_KEYS.THEME_MODE)
  const accentColorConfig = settings.find((c) => c.key === CONFIG_KEYS.ACCENT_COLOR)
  const uiStyleConfig = settings.find((c) => c.key === CONFIG_KEYS.UI_STYLE)
  const semanticColorsConfig = settings.find((c) => c.key === CONFIG_KEYS.SEMANTIC_COLORS)

  let changed = false
  const updated: AppearancePreferences = { ...preferences.value }

  if (themeModeConfig && typeof themeModeConfig.value === 'string') {
    updated.themeMode = themeModeConfig.value as ThemeMode
    changed = true
  }
  if (accentColorConfig && typeof accentColorConfig.value === 'string') {
    updated.accentColor = accentColorConfig.value as AccentColor
    changed = true
  }
  if (uiStyleConfig && typeof uiStyleConfig.value === 'string') {
    updated.uiStyle = uiStyleConfig.value as UiStyle
    changed = true
  }
  if (semanticColorsConfig && semanticColorsConfig.value) {
    let parsedColors = semanticColorsConfig.value
    if (typeof parsedColors === 'string') {
      try {
        parsedColors = JSON.parse(parsedColors)
      } catch {
        // Ignore parse error
      }
    }
    if (typeof parsedColors === 'object' && parsedColors !== null) {
      updated.semanticColors = parsedColors as SemanticColorsPreferences
      changed = true
    }
  }

  if (changed) {
    preferences.value = updated
    applyAppearanceToDom()
  }
}

export function getThemeModeTooltip(mode: ThemeMode, resolved: 'dark' | 'light'): string {
  if (mode === 'dark') return 'Theme: Dark'
  if (mode === 'light') return 'Theme: Light'
  const resolvedCap = resolved === 'dark' ? 'Dark' : 'Light'
  return `Theme: System (Following OS: ${resolvedCap})`
}

export function getThemeModeIcon(mode: ThemeMode): 'Moon' | 'Sun' | 'Laptop' {
  if (mode === 'dark') return 'Moon'
  if (mode === 'light') return 'Sun'
  return 'Laptop'
}

export function useAppearance() {
  const setThemeMode = async (mode: ThemeMode): Promise<void> => {
    preferences.value = {
      ...preferences.value,
      themeMode: mode
    }
    applyAppearanceToDom()
    await appearanceService.saveAppearancePreferences(preferences.value)
  }

  const cycleThemeMode = async (): Promise<void> => {
    const current = preferences.value.themeMode
    const next: ThemeMode = current === 'dark' ? 'light' : current === 'light' ? 'system' : 'dark'
    await setThemeMode(next)
  }

  const setAccentColor = async (color: AccentColor): Promise<void> => {
    preferences.value = {
      ...preferences.value,
      accentColor: color
    }
    applyAppearanceToDom()
    await appearanceService.saveAppearancePreferences(preferences.value)
  }

  const setUiStyle = async (style: UiStyle): Promise<void> => {
    preferences.value = {
      ...preferences.value,
      uiStyle: style
    }
    applyAppearanceToDom()
    await appearanceService.saveAppearancePreferences(preferences.value)
  }

  const setSemanticColors = async (colors: SemanticColorsPreferences): Promise<void> => {
    preferences.value = {
      ...preferences.value,
      semanticColors: colors
    }
    applyAppearanceToDom()
    await appearanceService.saveAppearancePreferences(preferences.value)
  }

  const setSemanticPreset = async (preset: SemanticPresetId): Promise<void> => {
    if (preset !== 'custom' && preset in SEMANTIC_COLOR_PRESETS) {
      const presetConfig = SEMANTIC_COLOR_PRESETS[preset as Exclude<SemanticPresetId, 'custom'>]
      const nextColors: SemanticColorsPreferences = {
        preset,
        success: presetConfig.success,
        warning: presetConfig.warning,
        danger: presetConfig.danger,
        info: presetConfig.info
      }
      await setSemanticColors(nextColors)
    } else {
      const current = {
        ...(preferences.value.semanticColors ?? DEFAULT_SEMANTIC_COLORS_PREFERENCES)
      }
      current.preset = 'custom'
      await setSemanticColors(current)
    }
  }

  const setSemanticColorRole = async (role: SemanticColorRole, optionId: string): Promise<void> => {
    const current = { ...(preferences.value.semanticColors ?? DEFAULT_SEMANTIC_COLORS_PREFERENCES) }
    current[role] = optionId
    current.preset = 'custom'
    await setSemanticColors(current)
  }

  return {
    preferences,
    themeMode,
    accentColor,
    uiStyle,
    semanticColors,
    resolvedTheme,
    isDark,
    systemIsDark,
    setThemeMode,
    cycleThemeMode,
    setAccentColor,
    setUiStyle,
    setSemanticColors,
    setSemanticPreset,
    setSemanticColorRole,
    initAppearance
  }
}
