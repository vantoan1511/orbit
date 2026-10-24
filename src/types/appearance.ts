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

export type SemanticColorRole = 'success' | 'warning' | 'danger' | 'info'

export type SemanticPresetId = 'default' | 'colorblind' | 'high-contrast' | 'custom'

export interface SemanticColorShade {
  solid: string
  soft: string
}

export interface SemanticColorOption {
  id: string
  name: string
  light: SemanticColorShade
  dark: SemanticColorShade
}

export interface SemanticColorsPreferences {
  preset: SemanticPresetId
  success: string
  warning: string
  danger: string
  info: string
}

export const DEFAULT_SEMANTIC_COLORS_PREFERENCES: SemanticColorsPreferences = {
  preset: 'default',
  success: 'emerald',
  warning: 'amber',
  danger: 'rose',
  info: 'blue'
}

export interface AppearancePreferences {
  themeMode: ThemeMode
  accentColor: AccentColor
  uiStyle: UiStyle
  semanticColors?: SemanticColorsPreferences
}

export const DEFAULT_APPEARANCE_PREFERENCES: AppearancePreferences = {
  themeMode: 'dark',
  accentColor: 'blue',
  uiStyle: 'flat',
  semanticColors: { ...DEFAULT_SEMANTIC_COLORS_PREFERENCES }
}
