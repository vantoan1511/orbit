import { updatePrimaryPalette } from '@primeuix/themes'
import type {
  AccentColor,
  SemanticColorsPreferences,
  ThemeMode,
  UiStyle
} from '../types/appearance.ts'
import {
  getAccentPalette,
  getPrimeVuePaletteForAccent,
  resolveSemanticPalette
} from './appearanceConstants.ts'

export interface AppearanceDomSyncOptions {
  themeMode: ThemeMode
  resolvedTheme: 'dark' | 'light'
  accentColor: AccentColor
  uiStyle: UiStyle
  semanticColors?: SemanticColorsPreferences
}

/**
 * Synchronizes HTML root attributes, CSS variables, and PrimeVue reactive primary palette.
 * @param options Appearance sync configuration
 * @param target Root element to apply to (defaults to document.documentElement when available)
 */
export function syncAppearanceToDom(
  options: AppearanceDomSyncOptions,
  target?: HTMLElement | null
): void {
  const root =
    target !== undefined
      ? target
      : typeof document !== 'undefined'
        ? document.documentElement
        : null

  if (!root) return

  const isDark = options.resolvedTheme === 'dark'

  // Theme & dark mode class
  root.classList.toggle('my-app-dark', isDark)
  root.setAttribute('data-theme', options.resolvedTheme)
  root.setAttribute('data-theme-mode', options.themeMode)
  root.setAttribute('data-accent', options.accentColor)
  root.setAttribute('data-ui-style', options.uiStyle)

  // Core accent CSS variables
  const palette = getAccentPalette(options.accentColor, isDark)
  root.style.setProperty('--accent', palette.accent)
  root.style.setProperty('--accent-hover', palette.hover)
  root.style.setProperty('--accent-active', palette.active)
  root.style.setProperty('--accent-soft', palette.soft)
  root.style.setProperty('--selection', palette.selection)
  root.style.setProperty('--input-focus', palette.accent)

  // Semantic status CSS variables
  const semanticPalette = resolveSemanticPalette(options.semanticColors, isDark)
  root.style.setProperty('--success', semanticPalette.success.solid)
  root.style.setProperty('--success-soft', semanticPalette.success.soft)
  root.style.setProperty('--warning', semanticPalette.warning.solid)
  root.style.setProperty('--warning-soft', semanticPalette.warning.soft)
  root.style.setProperty('--danger', semanticPalette.danger.solid)
  root.style.setProperty('--danger-soft', semanticPalette.danger.soft)
  root.style.setProperty('--info', semanticPalette.info.solid)
  root.style.setProperty('--info-soft', semanticPalette.info.soft)

  // PrimeVue primary palette sync & CSS variables
  const pvPalette = getPrimeVuePaletteForAccent(options.accentColor)
  const primaryColor = isDark ? pvPalette['400'] : pvPalette['500']
  const primaryHover = isDark ? pvPalette['300'] : pvPalette['600']
  const primaryActive = isDark ? pvPalette['200'] : pvPalette['700']
  const primaryContrast = isDark ? '#020617' : '#ffffff'

  root.style.setProperty('--p-primary-color', primaryColor)
  root.style.setProperty('--p-primary-hover-color', primaryHover)
  root.style.setProperty('--p-primary-active-color', primaryActive)
  root.style.setProperty('--p-primary-contrast-color', primaryContrast)
  root.style.setProperty('--p-focus-ring-color', palette.accent)

  // PrimeVue theme engine reactive update in browser
  try {
    if (typeof updatePrimaryPalette === 'function') {
      updatePrimaryPalette(pvPalette)
    }
  } catch {
    // Graceful fallback in non-browser or unit-test environments
  }
}
