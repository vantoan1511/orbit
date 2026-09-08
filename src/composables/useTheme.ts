import { ref } from 'vue'
import { themeService } from '../services/themeService.ts'

const mediaQuery =
  typeof window !== 'undefined' && window.matchMedia
    ? window.matchMedia('(prefers-color-scheme: dark)')
    : null

const getInitialTheme = (): boolean => {
  if (typeof window === 'undefined') return false
  return mediaQuery ? mediaQuery.matches : false
}

// Shared state so it remains consistent if used in multiple components
const isDark = ref(getInitialTheme())
const hasExplicitPreference = ref(false)

const applyTheme = (dark: boolean): void => {
  if (typeof document === 'undefined') return
  const html = document.documentElement
  html.classList.toggle('my-app-dark', dark)
  html.setAttribute('data-theme', dark ? 'dark' : 'light')
}

// Apply initial theme immediately upon module evaluation
applyTheme(isDark.value)

// Listen for system theme changes if user hasn't set an explicit preference
if (mediaQuery) {
  mediaQuery.addEventListener('change', (e: MediaQueryListEvent) => {
    if (!hasExplicitPreference.value) {
      isDark.value = e.matches
      applyTheme(isDark.value)
    }
  })
}

/**
 * Initialize theme from native storage.
 * Called before mounting or on application startup.
 */
export async function initTheme(): Promise<void> {
  try {
    const stored = await themeService.loadTheme()
    if (stored === 'dark') {
      hasExplicitPreference.value = true
      isDark.value = true
      applyTheme(true)
    } else if (stored === 'light') {
      hasExplicitPreference.value = true
      isDark.value = false
      applyTheme(false)
    } else if (mediaQuery) {
      hasExplicitPreference.value = false
      isDark.value = mediaQuery.matches
      applyTheme(isDark.value)
    }
  } catch (e) {
    console.warn('Failed to initialize theme from native storage:', e)
  }
}

export function useTheme() {
  const toggleTheme = (): void => {
    isDark.value = !isDark.value
    hasExplicitPreference.value = true
    applyTheme(isDark.value)
    void themeService.saveTheme(isDark.value ? 'dark' : 'light')
  }

  return {
    isDark,
    toggleTheme,
    initTheme
  }
}
