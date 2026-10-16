import type {
  AccentColor,
  AccentColorDefinition,
  AccentColorPalette,
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
