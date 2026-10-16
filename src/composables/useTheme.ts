import { themeService } from '../services/themeService.ts'
import { initAppearance, useAppearance } from './useAppearance.ts'

export async function initTheme(): Promise<void> {
  await initAppearance()
}

export function useTheme() {
  const { isDark, setThemeMode } = useAppearance()

  const toggleTheme = (): void => {
    const nextDark = !isDark.value
    void setThemeMode(nextDark ? 'dark' : 'light')
    void themeService.saveTheme(nextDark ? 'dark' : 'light')
  }

  return {
    isDark,
    toggleTheme,
    initTheme
  }
}
