import test from 'node:test'
import assert from 'node:assert/strict'
import { themeService } from '../../services/themeService.ts'
import { appearanceService } from '../../services/appearanceService.ts'
import { useTheme, initTheme } from '../useTheme.ts'

test('useTheme toggles theme and calls themeService.saveTheme', async () => {
  const originalSave = themeService.saveTheme
  let savedTheme = ''

  themeService.saveTheme = async (t: 'dark' | 'light') => {
    savedTheme = t
  }

  try {
    const { isDark, toggleTheme } = useTheme()
    const initial = isDark.value
    toggleTheme()
    assert.equal(isDark.value, !initial)
    assert.equal(savedTheme, isDark.value ? 'dark' : 'light')
  } finally {
    themeService.saveTheme = originalSave
  }
})

test('initTheme applies loaded theme from themeService', async () => {
  await appearanceService.clearAppearancePreferences()
  const originalLoad = themeService.loadTheme
  themeService.loadTheme = async () => 'dark'

  try {
    await initTheme()
    const { isDark } = useTheme()
    assert.equal(isDark.value, true)
  } finally {
    themeService.loadTheme = originalLoad
    await appearanceService.clearAppearancePreferences()
  }
})
