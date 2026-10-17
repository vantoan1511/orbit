import Nora from '@primeuix/themes/nora'
import type {
  AccentColor,
  AccentColorDefinition,
  AccentColorPalette,
  SemanticColorOption,
  SemanticColorRole,
  SemanticColorShade,
  SemanticColorsPreferences,
  SemanticPresetId,
  ThemeMode,
  UiStyle,
  UiStyleDefinition
} from '../types/appearance.ts'

export const ACCENT_COLOR_DEFINITIONS: readonly AccentColorDefinition[] = [
  {
    id: 'blue',
    name: 'Orbit Blue',
    light: {
      accent: '#4f8cff',
      hover: '#6ca2ff',
      active: '#3c78e8',
      soft: 'rgba(79, 140, 255, 0.12)',
      selection: 'rgba(79, 140, 255, 0.15)'
    },
    dark: {
      accent: '#6aa8ff',
      hover: '#84b8ff',
      active: '#4e95ff',
      soft: 'rgba(106, 168, 255, 0.14)',
      selection: 'rgba(106, 168, 255, 0.18)'
    }
  },
  {
    id: 'emerald',
    name: 'Emerald',
    light: {
      accent: '#10b981',
      hover: '#34d399',
      active: '#059669',
      soft: 'rgba(16, 185, 129, 0.12)',
      selection: 'rgba(16, 185, 129, 0.15)'
    },
    dark: {
      accent: '#34d399',
      hover: '#6ee7b7',
      active: '#10b981',
      soft: 'rgba(52, 211, 153, 0.14)',
      selection: 'rgba(52, 211, 153, 0.18)'
    }
  },
  {
    id: 'violet',
    name: 'Violet',
    light: {
      accent: '#8b5cf6',
      hover: '#a78bfa',
      active: '#7c3aed',
      soft: 'rgba(139, 92, 246, 0.12)',
      selection: 'rgba(139, 92, 246, 0.15)'
    },
    dark: {
      accent: '#a78bfa',
      hover: '#c4b5fd',
      active: '#8b5cf6',
      soft: 'rgba(167, 139, 250, 0.14)',
      selection: 'rgba(167, 139, 250, 0.18)'
    }
  },
  {
    id: 'cyan',
    name: 'Cyan',
    light: {
      accent: '#06b6d4',
      hover: '#22d3ee',
      active: '#0891b2',
      soft: 'rgba(6, 182, 212, 0.12)',
      selection: 'rgba(6, 182, 212, 0.15)'
    },
    dark: {
      accent: '#22d3ee',
      hover: '#67e8f9',
      active: '#06b6d4',
      soft: 'rgba(34, 211, 238, 0.14)',
      selection: 'rgba(34, 211, 238, 0.18)'
    }
  },
  {
    id: 'amber',
    name: 'Amber',
    light: {
      accent: '#f59e0b',
      hover: '#fbbf24',
      active: '#d97706',
      soft: 'rgba(245, 158, 11, 0.12)',
      selection: 'rgba(245, 158, 11, 0.15)'
    },
    dark: {
      accent: '#fbbf24',
      hover: '#fcd34d',
      active: '#f59e0b',
      soft: 'rgba(251, 191, 36, 0.14)',
      selection: 'rgba(251, 191, 36, 0.18)'
    }
  },
  {
    id: 'rose',
    name: 'Rose',
    light: {
      accent: '#f43f5e',
      hover: '#fb7185',
      active: '#e11d48',
      soft: 'rgba(244, 63, 94, 0.12)',
      selection: 'rgba(244, 63, 94, 0.15)'
    },
    dark: {
      accent: '#fb7185',
      hover: '#fda4af',
      active: '#f43f5e',
      soft: 'rgba(251, 113, 133, 0.14)',
      selection: 'rgba(251, 113, 133, 0.18)'
    }
  },
  {
    id: 'slate',
    name: 'Slate Noir',
    light: {
      accent: '#475569',
      hover: '#64748b',
      active: '#334155',
      soft: 'rgba(71, 85, 105, 0.12)',
      selection: 'rgba(71, 85, 105, 0.15)'
    },
    dark: {
      accent: '#94a3b8',
      hover: '#cbd5e1',
      active: '#64748b',
      soft: 'rgba(148, 163, 184, 0.14)',
      selection: 'rgba(148, 163, 184, 0.18)'
    }
  }
] as const

export const UI_STYLE_DEFINITIONS: readonly UiStyleDefinition[] = [
  {
    id: 'flat',
    name: 'Flat (Default)',
    description:
      'Nora minimalist, borderless grouping, high information density, pure technical IDE.',
    badge: 'Standard'
  },
  {
    id: 'glass',
    name: 'Modern Glass',
    description: 'Translucent panels with backdrop-filter blur and subtle glass highlight borders.'
  },
  {
    id: 'aero',
    name: 'Frutiger Aero',
    description: 'Subtle top-edge gloss highlights, sheen surfaces, and translucent headers.'
  },
  {
    id: 'liquidglass',
    name: 'Liquid Glass',
    description: 'High-index refraction blur with inner specular edge borders and deep contrast.'
  },
  {
    id: 'ios',
    name: 'iOS Cupertino',
    description: 'Frosted acrylic materials, smooth rounded cards, and vibrant system aesthetics.'
  }
] as const

export function getAccentPalette(color: AccentColor, isDark: boolean): AccentColorPalette {
  const match = ACCENT_COLOR_DEFINITIONS.find((c) => c.id === color)
  const def = match ?? ACCENT_COLOR_DEFINITIONS[0]!
  return isDark ? def.dark : def.light
}

export function formatThemeModeLabel(mode: ThemeMode): string {
  switch (mode) {
    case 'dark':
      return 'Dark'
    case 'light':
      return 'Light'
    case 'system':
      return 'System'
  }
}

export function formatUiStyleLabel(style: UiStyle): { name: string; description: string } {
  const def = UI_STYLE_DEFINITIONS.find((s) => s.id === style)
  if (!def) {
    return { name: style, description: '' }
  }
  return { name: def.name, description: def.description }
}

export function isTranslucentStyle(style: UiStyle): boolean {
  return style !== 'flat'
}

export interface PrimeVueColorPalette {
  50: string
  100: string
  200: string
  300: string
  400: string
  500: string
  600: string
  700: string
  800: string
  900: string
  950: string
}

/**
 * Maps each AccentColor to its corresponding PrimeVue Nora primitive palette.
 */
export function getPrimeVuePaletteForAccent(accent: AccentColor): PrimeVueColorPalette {
  const primitive = (Nora as { primitive?: Record<string, unknown> }).primitive
  if (primitive && typeof primitive === 'object') {
    const palette = primitive[accent] as PrimeVueColorPalette | undefined
    if (palette && typeof palette === 'object' && palette['500']) {
      return palette
    }
    const defaultPalette = primitive.blue as PrimeVueColorPalette | undefined
    if (defaultPalette && typeof defaultPalette === 'object' && defaultPalette['500']) {
      return defaultPalette
    }
  }

  // Fallback palette in case primitive is unavailable in test environment
  return {
    50: '#eff6ff',
    100: '#dbeafe',
    200: '#bfdbfe',
    300: '#93c5fd',
    400: '#60a5fa',
    500: '#3b82f6',
    600: '#2563eb',
    700: '#1d4ed8',
    800: '#1e40af',
    900: '#1e3a8a',
    950: '#172554'
  }
}

export interface UiStyleTokens {
  radiusCard: string
  radiusControl: string
  cardSheen: string
  cardShadow: string
  cardBorder: string
  backdropBlur: string
}

export function getUiStyleTokens(style: UiStyle, isDark: boolean): UiStyleTokens {
  switch (style) {
    case 'glass':
      return {
        radiusCard: '0.75rem',
        radiusControl: '0.375rem',
        cardSheen: 'none',
        cardShadow: isDark ? '0 8px 32px rgba(0, 0, 0, 0.4)' : '0 8px 32px rgba(0, 0, 0, 0.08)',
        cardBorder: isDark ? 'rgba(255, 255, 255, 0.12)' : 'rgba(0, 0, 0, 0.08)',
        backdropBlur: 'blur(14px)'
      }
    case 'aero':
      return {
        radiusCard: '0.75rem',
        radiusControl: '0.5rem',
        cardSheen:
          'linear-gradient(180deg, rgba(255, 255, 255, 0.08) 0%, rgba(255, 255, 255, 0.01) 100%)',
        cardShadow: isDark
          ? 'inset 0 1px 0 rgba(255, 255, 255, 0.22), 0 2px 8px rgba(0, 0, 0, 0.35)'
          : 'inset 0 1px 0 rgba(255, 255, 255, 0.8), 0 2px 6px rgba(0, 0, 0, 0.06)',
        cardBorder: isDark ? 'rgba(255, 255, 255, 0.16)' : 'rgba(200, 215, 235, 0.8)',
        backdropBlur: 'blur(14px)'
      }
    case 'liquidglass':
      return {
        radiusCard: '0.75rem',
        radiusControl: '0.5rem',
        cardSheen:
          'linear-gradient(135deg, rgba(255, 255, 255, 0.1) 0%, rgba(255, 255, 255, 0.02) 100%)',
        cardShadow: isDark
          ? 'inset 0 1px 1px 0 rgba(255, 255, 255, 0.35), inset 0 -1px 1px 0 rgba(0, 0, 0, 0.25), 0 8px 32px 0 rgba(0, 0, 0, 0.45)'
          : 'inset 0 1px 1px 0 rgba(255, 255, 255, 0.8), inset 0 -1px 1px 0 rgba(0, 0, 0, 0.06), 0 8px 32px 0 rgba(31, 38, 135, 0.12)',
        cardBorder: isDark ? 'rgba(255, 255, 255, 0.2)' : 'rgba(255, 255, 255, 0.5)',
        backdropBlur: 'blur(20px) saturate(180%)'
      }
    case 'ios':
      return {
        radiusCard: '1rem',
        radiusControl: '0.625rem',
        cardSheen: 'none',
        cardShadow: isDark
          ? '0 10px 25px -5px rgba(0, 0, 0, 0.5), 0 8px 10px -6px rgba(0, 0, 0, 0.5)'
          : '0 10px 25px -5px rgba(0, 0, 0, 0.08), 0 8px 10px -6px rgba(0, 0, 0, 0.04)',
        cardBorder: isDark ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.08)',
        backdropBlur: 'blur(24px) saturate(190%)'
      }
    case 'flat':
    default:
      return {
        radiusCard: '0.5rem',
        radiusControl: '0.375rem',
        cardSheen: 'none',
        cardShadow: 'none',
        cardBorder: isDark ? '#212427' : '#d5d9e1',
        backdropBlur: 'none'
      }
  }
}

export const SEMANTIC_ROLE_OPTIONS: Record<SemanticColorRole, readonly SemanticColorOption[]> = {
  success: [
    {
      id: 'emerald',
      name: 'Emerald (Standard)',
      light: { solid: '#28a745', soft: 'rgba(40, 167, 69, 0.12)' },
      dark: { solid: '#46d16e', soft: 'rgba(70, 209, 110, 0.14)' }
    },
    {
      id: 'forest',
      name: 'Forest Green',
      light: { solid: '#15803d', soft: 'rgba(21, 128, 61, 0.12)' },
      dark: { solid: '#22c55e', soft: 'rgba(34, 197, 94, 0.14)' }
    },
    {
      id: 'mint',
      name: 'Fresh Mint',
      light: { solid: '#059669', soft: 'rgba(5, 150, 105, 0.12)' },
      dark: { solid: '#10b981', soft: 'rgba(16, 185, 129, 0.14)' }
    },
    {
      id: 'teal',
      name: 'Deep Teal',
      light: { solid: '#0f766e', soft: 'rgba(15, 118, 110, 0.12)' },
      dark: { solid: '#14b8a6', soft: 'rgba(20, 184, 166, 0.14)' }
    }
  ],
  warning: [
    {
      id: 'amber',
      name: 'Amber (Standard)',
      light: { solid: '#f4a100', soft: 'rgba(244, 161, 0, 0.12)' },
      dark: { solid: '#ffc54d', soft: 'rgba(255, 197, 77, 0.14)' }
    },
    {
      id: 'orange',
      name: 'Vivid Orange',
      light: { solid: '#ea580c', soft: 'rgba(234, 88, 12, 0.12)' },
      dark: { solid: '#fb923c', soft: 'rgba(251, 146, 60, 0.14)' }
    },
    {
      id: 'yellow',
      name: 'Bright Yellow',
      light: { solid: '#ca8a04', soft: 'rgba(202, 138, 4, 0.12)' },
      dark: { solid: '#facc15', soft: 'rgba(250, 204, 21, 0.14)' }
    },
    {
      id: 'bronze',
      name: 'Warm Bronze',
      light: { solid: '#b45309', soft: 'rgba(180, 83, 9, 0.12)' },
      dark: { solid: '#f59e0b', soft: 'rgba(245, 158, 11, 0.14)' }
    }
  ],
  danger: [
    {
      id: 'rose',
      name: 'Rose (Standard)',
      light: { solid: '#d64545', soft: 'rgba(214, 69, 69, 0.12)' },
      dark: { solid: '#ff6b6b', soft: 'rgba(255, 107, 107, 0.14)' }
    },
    {
      id: 'crimson',
      name: 'Crimson Red',
      light: { solid: '#dc2626', soft: 'rgba(220, 38, 38, 0.12)' },
      dark: { solid: '#f87171', soft: 'rgba(248, 113, 113, 0.14)' }
    },
    {
      id: 'ruby',
      name: 'Vibrant Ruby',
      light: { solid: '#e11d48', soft: 'rgba(225, 29, 72, 0.12)' },
      dark: { solid: '#fb7185', soft: 'rgba(251, 113, 133, 0.14)' }
    },
    {
      id: 'coral',
      name: 'Deep Coral',
      light: { solid: '#be123c', soft: 'rgba(190, 18, 60, 0.12)' },
      dark: { solid: '#f43f5e', soft: 'rgba(244, 63, 94, 0.14)' }
    }
  ],
  info: [
    {
      id: 'blue',
      name: 'Orbit Blue (Standard)',
      light: { solid: '#3d8bff', soft: 'rgba(61, 139, 255, 0.12)' },
      dark: { solid: '#6aa8ff', soft: 'rgba(106, 168, 255, 0.14)' }
    },
    {
      id: 'cyan',
      name: 'Electric Cyan',
      light: { solid: '#0891b2', soft: 'rgba(8, 145, 178, 0.12)' },
      dark: { solid: '#22d3ee', soft: 'rgba(34, 211, 238, 0.14)' }
    },
    {
      id: 'sky',
      name: 'Sky Blue',
      light: { solid: '#0284c7', soft: 'rgba(2, 132, 199, 0.12)' },
      dark: { solid: '#38bdf8', soft: 'rgba(56, 189, 248, 0.14)' }
    },
    {
      id: 'indigo',
      name: 'Deep Indigo',
      light: { solid: '#4f46e5', soft: 'rgba(79, 70, 229, 0.12)' },
      dark: { solid: '#818cf8', soft: 'rgba(129, 140, 248, 0.14)' }
    }
  ]
} as const

export const SEMANTIC_COLOR_PRESETS: Record<
  Exclude<SemanticPresetId, 'custom'>,
  Omit<SemanticColorsPreferences, 'preset'>
> = {
  default: {
    success: 'emerald',
    warning: 'amber',
    danger: 'rose',
    info: 'blue'
  },
  colorblind: {
    success: 'teal',
    warning: 'amber',
    danger: 'crimson',
    info: 'sky'
  },
  'high-contrast': {
    success: 'forest',
    warning: 'orange',
    danger: 'ruby',
    info: 'indigo'
  }
} as const

export interface ResolvedSemanticPalette {
  success: SemanticColorShade
  warning: SemanticColorShade
  danger: SemanticColorShade
  info: SemanticColorShade
}

function resolveRoleShade(
  role: SemanticColorRole,
  optionId: string | undefined,
  isDark: boolean
): SemanticColorShade {
  const options = SEMANTIC_ROLE_OPTIONS[role]
  const matched = options.find((opt) => opt.id === optionId) ?? options[0]!
  return isDark ? matched.dark : matched.light
}

export function resolveSemanticPalette(
  preferences?: Partial<SemanticColorsPreferences>,
  isDark: boolean = false
): ResolvedSemanticPalette {
  const presetId = preferences?.preset ?? 'default'
  const presetConfig =
    presetId !== 'custom' && presetId in SEMANTIC_COLOR_PRESETS
      ? SEMANTIC_COLOR_PRESETS[presetId as Exclude<SemanticPresetId, 'custom'>]
      : SEMANTIC_COLOR_PRESETS.default

  const successId = preferences?.success ?? presetConfig.success
  const warningId = preferences?.warning ?? presetConfig.warning
  const dangerId = preferences?.danger ?? presetConfig.danger
  const infoId = preferences?.info ?? presetConfig.info

  return {
    success: resolveRoleShade('success', successId, isDark),
    warning: resolveRoleShade('warning', warningId, isDark),
    danger: resolveRoleShade('danger', dangerId, isDark),
    info: resolveRoleShade('info', infoId, isDark)
  }
}
