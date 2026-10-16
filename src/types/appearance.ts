export type ThemeMode = 'dark' | 'light' | 'system'

export type AccentColor = 'blue' | 'emerald' | 'violet' | 'cyan' | 'amber' | 'rose' | 'slate'

export type UiStyle = 'flat' | 'glass' | 'aero' | 'liquidglass' | 'ios'

export interface AccentColorPalette {
  accent: string
  hover: string
  active: string
  soft: string
  selection: string
}

export interface AccentColorDefinition {
  id: AccentColor
  name: string
  light: AccentColorPalette
  dark: AccentColorPalette
}

export interface UiStyleDefinition {
  id: UiStyle
  name: string
  description: string
  badge?: string
}

export interface AppearancePreferences {
  themeMode: ThemeMode
  accentColor: AccentColor
  uiStyle: UiStyle
}

export const DEFAULT_APPEARANCE_PREFERENCES: AppearancePreferences = {
  themeMode: 'system',
  accentColor: 'blue',
  uiStyle: 'flat'
}
