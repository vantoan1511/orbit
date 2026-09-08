import test from 'node:test'
import assert from 'node:assert/strict'
import { THEME_STORAGE_KEY, LEGACY_THEME_STORAGE_KEY, themeService } from '../themeService.ts'
import { storage } from '../nativeService.ts'

test('themeService.loadTheme returns dark or light from native storage', async () => {
  const originalGetData = storage.getData

  storage.getData = async (key: string) => {
    if (key === THEME_STORAGE_KEY) {
      return 'dark'
    }
    throw new Error('Not found')
  }

  try {
    const theme = await themeService.loadTheme()
    assert.equal(theme, 'dark')
  } finally {
    storage.getData = originalGetData
  }
})

test('themeService.loadTheme returns null for unrecognized values', async () => {
  const originalGetData = storage.getData

  storage.getData = async (key: string) => {
    if (key === THEME_STORAGE_KEY) {
      return 'invalid-color'
    }
    throw new Error('Not found')
  }

  try {
    const theme = await themeService.loadTheme()
    assert.equal(theme, null)
  } finally {
    storage.getData = originalGetData
  }
})

test('themeService.loadTheme migrates legacy orbit-theme-preference from localStorage', async () => {
  const originalGetData = storage.getData
  const originalSetData = storage.setData

  let nativeStorageValue: string | null = null
  const removedKeys: string[] = []

  storage.getData = async () => {
    throw new Error('Not in native storage')
  }

  storage.setData = async (key: string, data?: string | null) => {
    if (key === THEME_STORAGE_KEY) {
      nativeStorageValue = data ?? null
    }
  }

  const originalLocalStorage = globalThis.localStorage
  const mockLocalStorage = {
    getItem(key: string) {
      if (key === LEGACY_THEME_STORAGE_KEY) {
        return 'light'
      }
      return null
    },
    removeItem(key: string) {
      removedKeys.push(key)
    },
    setItem() {}
  }

  Object.defineProperty(globalThis, 'localStorage', {
    value: mockLocalStorage,
    configurable: true,
    writable: true
  })

  try {
    const theme = await themeService.loadTheme()
    assert.equal(theme, 'light')
    assert.equal(nativeStorageValue, 'light')
    assert.ok(removedKeys.includes(LEGACY_THEME_STORAGE_KEY))
  } finally {
    storage.getData = originalGetData
    storage.setData = originalSetData
    Object.defineProperty(globalThis, 'localStorage', {
      value: originalLocalStorage,
      configurable: true,
      writable: true
    })
  }
})

test('themeService.saveTheme saves valid theme to native storage', async () => {
  const originalSetData = storage.setData
  let savedKey = ''
  let savedValue = ''

  storage.setData = async (key: string, data?: string | null) => {
    savedKey = key
    savedValue = data || ''
  }

  try {
    await themeService.saveTheme('dark')
    assert.equal(savedKey, THEME_STORAGE_KEY)
    assert.equal(savedValue, 'dark')
  } finally {
    storage.setData = originalSetData
  }
})

test('themeService.clearTheme removes key from native storage', async () => {
  const originalRemoveData = storage.removeData
  let removedKey = ''

  storage.removeData = async (key: string) => {
    removedKey = key
  }

  try {
    await themeService.clearTheme()
    assert.equal(removedKey, THEME_STORAGE_KEY)
  } finally {
    storage.removeData = originalRemoveData
  }
})
