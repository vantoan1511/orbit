import { computed, ref } from 'vue'
import { appearanceService } from '../services/appearanceService.ts'
import { CONFIG_KEYS, type Configuration } from '../types/settings.ts'
import {
  DEFAULT_APPEARANCE_PREFERENCES,
  type AccentColor,
  type AppearancePreferences,
  type ThemeMode,
  type UiStyle
} from '../types/appearance.ts'
import { getAccentPalette } from '../utils/appearanceConstants.ts'

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

export function applyAppearanceToDom(): void {
  if (typeof document === 'undefined') return
  const html = document.documentElement

  const currentDark = isDark.value
  const currentResolved = resolvedTheme.value
  const currentMode = preferences.value.themeMode
  const currentAccent = preferences.value.accentColor
  const currentStyle = preferences.value.uiStyle

  // Theme & dark mode class
  html.classList.toggle('my-app-dark', currentDark)
  html.setAttribute('data-theme', currentResolved)
  html.setAttribute('data-theme-mode', currentMode)
  html.setAttribute('data-accent', currentAccent)
  html.setAttribute('data-ui-style', currentStyle)

  // CSS variables for dynamic accents
  const palette = getAccentPalette(currentAccent, currentDark)
  html.style.setProperty('--accent', palette.accent)
  html.style.setProperty('--accent-hover', palette.hover)
  html.style.setProperty('--accent-active', palette.active)
  html.style.setProperty('--accent-soft', palette.soft)
  html.style.setProperty('--selection', palette.selection)
  html.style.setProperty('--input-focus', palette.accent)
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

  return {
    preferences,
    themeMode,
    accentColor,
    uiStyle,
    resolvedTheme,
    isDark,
    systemIsDark,
    setThemeMode,
    cycleThemeMode,
    setAccentColor,
    setUiStyle,
    initAppearance
  }
}
