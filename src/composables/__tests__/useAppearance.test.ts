import test from 'node:test'
import assert from 'node:assert/strict'
import { appearanceService } from '../../services/appearanceService.ts'
import {
  useAppearance,
  initAppearance,
  getThemeModeTooltip,
  getThemeModeIcon
} from '../useAppearance.ts'
import { useTheme } from '../useTheme.ts'
import type { AppearancePreferences } from '../../types/appearance.ts'

test('useAppearance initializes and updates themeMode', async () => {
  const originalSave = appearanceService.saveAppearancePreferences
  let savedPrefs: AppearancePreferences | null = null

  appearanceService.saveAppearancePreferences = async (p) => {
    savedPrefs = p
  }

  try {
    const { themeMode, setThemeMode, resolvedTheme } = useAppearance()
    await setThemeMode('dark')
    assert.equal(themeMode.value, 'dark')
    assert.equal(resolvedTheme.value, 'dark')
    assert.equal(savedPrefs?.themeMode, 'dark')

    await setThemeMode('light')
    assert.equal(themeMode.value, 'light')
    assert.equal(resolvedTheme.value, 'light')
    assert.equal(savedPrefs?.themeMode, 'light')
  } finally {
    appearanceService.saveAppearancePreferences = originalSave
  }
})

test('cycleThemeMode cycles dark -> light -> system -> dark', async () => {
  const originalSave = appearanceService.saveAppearancePreferences
  appearanceService.saveAppearancePreferences = async () => {}

  try {
    const { themeMode, setThemeMode, cycleThemeMode } = useAppearance()

    await setThemeMode('dark')
    assert.equal(themeMode.value, 'dark')

    await cycleThemeMode()
    assert.equal(themeMode.value, 'light')

    await cycleThemeMode()
    assert.equal(themeMode.value, 'system')

    await cycleThemeMode()
    assert.equal(themeMode.value, 'dark')
  } finally {
    appearanceService.saveAppearancePreferences = originalSave
  }
})

test('setAccentColor updates accent color and persists', async () => {
  const originalSave = appearanceService.saveAppearancePreferences
  let savedColor = ''

  appearanceService.saveAppearancePreferences = async (p) => {
    savedColor = p.accentColor
  }

  try {
    const { accentColor, setAccentColor } = useAppearance()
    await setAccentColor('emerald')
    assert.equal(accentColor.value, 'emerald')
    assert.equal(savedColor, 'emerald')
  } finally {
    appearanceService.saveAppearancePreferences = originalSave
  }
})

test('setUiStyle updates UI style and persists', async () => {
  const originalSave = appearanceService.saveAppearancePreferences
  let savedStyle = ''

  appearanceService.saveAppearancePreferences = async (p) => {
    savedStyle = p.uiStyle
  }

  try {
    const { uiStyle, setUiStyle } = useAppearance()
    await setUiStyle('glass')
    assert.equal(uiStyle.value, 'glass')
    assert.equal(savedStyle, 'glass')
  } finally {
    appearanceService.saveAppearancePreferences = originalSave
  }
})

test('getThemeModeTooltip and getThemeModeIcon return descriptive details', () => {
  assert.equal(getThemeModeTooltip('dark', 'dark'), 'Theme: Dark')
  assert.equal(getThemeModeTooltip('light', 'light'), 'Theme: Light')
  assert.equal(getThemeModeTooltip('system', 'dark'), 'Theme: System (Following OS: Dark)')

  assert.equal(getThemeModeIcon('dark'), 'Moon')
  assert.equal(getThemeModeIcon('light'), 'Sun')
  assert.equal(getThemeModeIcon('system'), 'Laptop')
})

test('initAppearance loads preferences from appearanceService', async () => {
  const originalLoad = appearanceService.loadAppearancePreferences
  appearanceService.loadAppearancePreferences = async () => ({
    themeMode: 'light',
    accentColor: 'rose',
    uiStyle: 'ios'
  })

  try {
    await initAppearance()
    const { themeMode, accentColor, uiStyle } = useAppearance()
    assert.equal(themeMode.value, 'light')
    assert.equal(accentColor.value, 'rose')
    assert.equal(uiStyle.value, 'ios')
  } finally {
    appearanceService.loadAppearancePreferences = originalLoad
  }
})

test('useTheme backward compatibility delegates to useAppearance', async () => {
  const { isDark, toggleTheme } = useTheme()
  const initial = isDark.value
  await toggleTheme()
  assert.equal(isDark.value, !initial)
})

test('syncAppearanceFromSettings updates appearance from backend Configuration array', async () => {
  const { syncAppearanceFromSettings, useAppearance } = await import('../useAppearance.ts')
  const { themeMode, accentColor, uiStyle } = useAppearance()

  syncAppearanceFromSettings([
    {
      key: 'themeMode',
      name: 'Theme Mode',
      description: '',
      datatype: 'string',
      value: 'light',
      isConfidential: false,
      cardinality: '1..1',
      enable: true
    },
    {
      key: 'accentColor',
      name: 'Accent Color',
      description: '',
      datatype: 'string',
      value: 'cyan',
      isConfidential: false,
      cardinality: '1..1',
      enable: true
    },
    {
      key: 'uiStyle',
      name: 'UI Style',
      description: '',
      datatype: 'string',
      value: 'aero',
      isConfidential: false,
      cardinality: '1..1',
      enable: true
    }
  ])

  assert.equal(themeMode.value, 'light')
  assert.equal(accentColor.value, 'cyan')
  assert.equal(uiStyle.value, 'aero')
})

test('useAppearance provides semanticColors and updates preset and individual roles', async () => {
  const originalSave = appearanceService.saveAppearancePreferences
  let savedColors: unknown = null

  appearanceService.saveAppearancePreferences = async (p) => {
    savedColors = p.semanticColors
  }

  try {
    const { semanticColors, setSemanticPreset, setSemanticColorRole } = useAppearance()

    assert.ok(semanticColors.value)
    assert.equal(semanticColors.value.preset, 'default')

    await setSemanticPreset('colorblind')
    assert.equal(semanticColors.value.preset, 'colorblind')
    assert.equal(semanticColors.value.success, 'teal')
    assert.equal(semanticColors.value.danger, 'crimson')
    assert.ok(savedColors)

    await setSemanticColorRole('success', 'mint')
    assert.equal(semanticColors.value.preset, 'custom')
    assert.equal(semanticColors.value.success, 'mint')
  } finally {
    appearanceService.saveAppearancePreferences = originalSave
  }
})

test('syncAppearanceFromSettings updates semanticColors from backend Configuration', async () => {
  const { syncAppearanceFromSettings, useAppearance } = await import('../useAppearance.ts')
  const { semanticColors } = useAppearance()

  syncAppearanceFromSettings([
    {
      key: 'semanticColors',
      name: 'Semantic Colors',
      description: '',
      datatype: 'string',
      value: {
        preset: 'high-contrast',
        success: 'forest',
        warning: 'orange',
        danger: 'ruby',
        info: 'indigo'
      },
      isConfidential: false,
      cardinality: '0..1',
      enable: true
    }
  ])

  assert.equal(semanticColors.value.preset, 'high-contrast')
  assert.equal(semanticColors.value.success, 'forest')
  assert.equal(semanticColors.value.danger, 'ruby')
})
