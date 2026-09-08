import test from 'node:test'
import assert from 'node:assert/strict'
import { storage } from '../../services/nativeService.ts'
import { useResizable } from '../useResizable.ts'

test('useResizable initializes with defaultWidth', () => {
  const { width, isResizing } = useResizable({ defaultWidth: 300 })
  assert.equal(width.value, 300)
  assert.equal(isResizing.value, false)
})

test('useResizable.loadStoredWidth loads width from native storage', async () => {
  const originalGetData = storage.getData
  storage.getData = async (key: string) => {
    if (key === 'test_panel_width') {
      return '350'
    }
    throw new Error('Not found')
  }

  try {
    const { width, loadStoredWidth } = useResizable({
      storageKey: 'test_panel_width',
      defaultWidth: 260,
      minWidth: 180,
      maxWidth: 600
    })

    await loadStoredWidth()
    assert.equal(width.value, 350)
  } finally {
    storage.getData = originalGetData
  }
})

test('useResizable.loadStoredWidth migrates legacy localStorage width', async () => {
  const originalGetData = storage.getData
  const originalSetData = storage.setData

  let savedNativeKey = ''
  let savedNativeVal = ''
  let removedLocalKey = ''

  storage.getData = async () => {
    throw new Error('Not in native storage')
  }
  storage.setData = async (key: string, val?: string | null) => {
    savedNativeKey = key
    savedNativeVal = val || ''
  }

  const originalLocalStorage = globalThis.localStorage
  const mockLocalStorage = {
    getItem(key: string) {
      if (key === 'test_legacy_width') {
        return '420'
      }
      return null
    },
    removeItem(key: string) {
      removedLocalKey = key
    },
    setItem() {}
  }

  Object.defineProperty(globalThis, 'localStorage', {
    value: mockLocalStorage,
    configurable: true,
    writable: true
  })

  try {
    const { width, loadStoredWidth } = useResizable({
      storageKey: 'test_legacy_width',
      defaultWidth: 260,
      minWidth: 180,
      maxWidth: 600
    })

    await loadStoredWidth()
    assert.equal(width.value, 420)
    assert.equal(savedNativeKey, 'test_legacy_width')
    assert.equal(savedNativeVal, '420')
    assert.equal(removedLocalKey, 'test_legacy_width')
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

test('useResizable ignores stored width outside min/maxWidth bounds', async () => {
  const originalGetData = storage.getData
  storage.getData = async () => '9999'

  try {
    const { width, loadStoredWidth } = useResizable({
      storageKey: 'test_overflow_width',
      defaultWidth: 260,
      minWidth: 180,
      maxWidth: 600
    })

    await loadStoredWidth()
    assert.equal(width.value, 260)
  } finally {
    storage.getData = originalGetData
  }
})

test('useResizable preserves legacy localStorage when native storage write fails', async () => {
  const originalGetData = storage.getData
  const originalSetData = storage.setData

  let removedLocalKey = ''

  storage.getData = async () => {
    throw new Error('Not in native storage')
  }
  storage.setData = async () => {
    throw new Error('Native disk write failed')
  }

  const originalLocalStorage = globalThis.localStorage
  const mockLocalStorage = {
    getItem(key: string) {
      if (key === 'test_failed_migration') {
        return '380'
      }
      return null
    },
    removeItem(key: string) {
      removedLocalKey = key
    },
    setItem() {}
  }

  Object.defineProperty(globalThis, 'localStorage', {
    value: mockLocalStorage,
    configurable: true,
    writable: true
  })

  try {
    const { width, loadStoredWidth } = useResizable({
      storageKey: 'test_failed_migration',
      defaultWidth: 260,
      minWidth: 180,
      maxWidth: 600
    })

    await loadStoredWidth()
    // The width is still used for current session
    assert.equal(width.value, 380)
    // But localStorage key was NOT deleted because native write failed
    assert.equal(removedLocalKey, '')
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
