import test from 'node:test'
import assert from 'node:assert/strict'
import { APPEARANCE_STORAGE_KEY, appearanceService } from '../appearanceService.ts'
import { THEME_STORAGE_KEY } from '../themeService.ts'
import { storage } from '../nativeService.ts'
import {
  DEFAULT_APPEARANCE_PREFERENCES,
  type AppearancePreferences
} from '../../types/appearance.ts'

test('appearanceService.loadAppearancePreferences returns defaults when storage is empty', async () => {
  const originalGetData = storage.getData

  storage.getData = async () => {
    throw new Error('Not found')
  }

  try {
    const prefs = await appearanceService.loadAppearancePreferences()
    assert.deepEqual(prefs, DEFAULT_APPEARANCE_PREFERENCES)
  } finally {
    storage.getData = originalGetData
  }
})

test('appearanceService.loadAppearancePreferences loads saved preferences', async () => {
  const originalGetData = storage.getData

  const customPrefs: AppearancePreferences = {
    themeMode: 'dark',
    accentColor: 'emerald',
    uiStyle: 'glass'
  }

  storage.getData = async (key: string) => {
    if (key === APPEARANCE_STORAGE_KEY) {
      return JSON.stringify(customPrefs)
    }
    throw new Error('Not found')
  }

  try {
    const prefs = await appearanceService.loadAppearancePreferences()
    assert.deepEqual(prefs, customPrefs)
  } finally {
    storage.getData = originalGetData
  }
})

test('appearanceService.loadAppearancePreferences migrates legacy orbit_theme_preference', async () => {
  const originalGetData = storage.getData
  const originalSetData = storage.setData
  let savedAppearanceRaw: string | null = null

  storage.getData = async (key: string) => {
    if (key === APPEARANCE_STORAGE_KEY) {
      throw new Error('Appearance preferences not yet stored')
    }
    if (key === THEME_STORAGE_KEY) {
      return 'dark'
    }
    throw new Error('Not found')
  }

  storage.setData = async (key: string, data?: string | null) => {
    if (key === APPEARANCE_STORAGE_KEY) {
      savedAppearanceRaw = data ?? null
    }
  }

  try {
    const prefs = await appearanceService.loadAppearancePreferences()
    assert.equal(prefs.themeMode, 'dark')
    assert.equal(prefs.accentColor, 'blue')
    assert.equal(prefs.uiStyle, 'flat')
    assert.ok(savedAppearanceRaw !== null)
    assert.deepEqual(JSON.parse(savedAppearanceRaw), {
      themeMode: 'dark',
      accentColor: 'blue',
      uiStyle: 'flat'
    })
  } finally {
    storage.getData = originalGetData
    storage.setData = originalSetData
  }
})

test('appearanceService.saveAppearancePreferences persists preferences and syncs themeService key', async () => {
  const originalSetData = storage.setData
  const savedData = new Map<string, string>()

  storage.setData = async (key: string, data?: string | null) => {
    if (data !== undefined && data !== null) {
      savedData.set(key, data)
    }
  }

  const prefsToSave: AppearancePreferences = {
    themeMode: 'light',
    accentColor: 'violet',
    uiStyle: 'aero'
  }

  try {
    await appearanceService.saveAppearancePreferences(prefsToSave)
    assert.ok(savedData.has(APPEARANCE_STORAGE_KEY))
    assert.deepEqual(JSON.parse(savedData.get(APPEARANCE_STORAGE_KEY)!), prefsToSave)
    // Synchronized active theme mode
    assert.equal(savedData.get(THEME_STORAGE_KEY), 'light')
  } finally {
    storage.setData = originalSetData
  }
})

test('appearanceService.clearAppearancePreferences removes key from native storage', async () => {
  const originalRemoveData = storage.removeData
  const removedKeys: string[] = []

  storage.removeData = async (key: string) => {
    removedKeys.push(key)
  }

  try {
    await appearanceService.clearAppearancePreferences()
    assert.ok(removedKeys.includes(APPEARANCE_STORAGE_KEY))
  } finally {
    storage.removeData = originalRemoveData
  }
})

test('appearanceService.saveAppearancePreferences dispatches updateAppSettings to configuration system', async () => {
  const { coreEngine } = await import('../nativeService.ts')
  const originalDispatch = coreEngine.dispatch
  let dispatchedEvent = ''
  let dispatchedPayload: Record<string, unknown> | null = null

  coreEngine.dispatch = async (event: string, data?: unknown) => {
    dispatchedEvent = event
    dispatchedPayload = data as Record<string, unknown>
  }

  const prefsToSave: AppearancePreferences = {
    themeMode: 'light',
    accentColor: 'violet',
    uiStyle: 'aero'
  }

  try {
    await appearanceService.saveAppearancePreferences(prefsToSave)
    assert.equal(dispatchedEvent, 'updateAppSettings')
    assert.ok(dispatchedPayload)
    assert.equal(dispatchedPayload.themeMode, 'light')
    assert.equal(dispatchedPayload.accentColor, 'violet')
    assert.equal(dispatchedPayload.uiStyle, 'aero')
  } finally {
    coreEngine.dispatch = originalDispatch
  }
})
