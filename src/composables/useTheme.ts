import { ref } from 'vue'

const THEME_STORAGE_KEY = 'orbit-theme-preference'

const mediaQuery =
  typeof window !== 'undefined' && window.matchMedia
    ? window.matchMedia('(prefers-color-scheme: dark)')
    : null

const getInitialTheme = (): boolean => {
  if (typeof window === 'undefined') return false
  const storedTheme = localStorage.getItem(THEME_STORAGE_KEY)
  if (storedTheme === 'dark') return true
  if (storedTheme === 'light') return false
  return mediaQuery ? mediaQuery.matches : false
}

// Shared state so it remains consistent if used in multiple components
const isDark = ref(getInitialTheme())

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
    if (!localStorage.getItem(THEME_STORAGE_KEY)) {
      isDark.value = e.matches
      applyTheme(isDark.value)
    }
  })
}

export function useTheme() {
  const toggleTheme = (): void => {
    isDark.value = !isDark.value
    localStorage.setItem(THEME_STORAGE_KEY, isDark.value ? 'dark' : 'light')
    applyTheme(isDark.value)
  }

  return {
    isDark,
    toggleTheme
  }
}
