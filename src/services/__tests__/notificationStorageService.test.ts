import test from 'node:test'
import assert from 'node:assert/strict'
import {
  NOTIFICATION_STORAGE_KEY,
  MAX_NOTIFICATIONS,
  validateNotifications,
  notificationStorageService
} from '../notificationStorageService.ts'
import { storage } from '../nativeService.ts'
import type { NotificationItem } from '@/types/notification'

test('validateNotifications handles null, non-array, and invalid data gracefully', () => {
  assert.deepEqual(validateNotifications(null), [])
  assert.deepEqual(validateNotifications('not-an-array'), [])
  assert.deepEqual(validateNotifications({}), [])

  const mixedData = [
    {
      id: '1',
      title: 'Test 1',
      message: 'Msg 1',
      severity: 'info',
      category: 'system',
      timestamp: 100,
      read: false
    },
    { id: '2', title: 'Invalid' }, // missing message, severity, category, etc.
    null,
    'random-string',
    {
      id: '3',
      title: 'Test 3',
      message: 'Msg 3',
      severity: 'error',
      category: 'kubernetes',
      timestamp: 200,
      read: true
    }
  ]

  const valid = validateNotifications(mixedData)
  assert.equal(valid.length, 2)
  assert.equal(valid[0].id, '1')
  assert.equal(valid[1].id, '3')
})

test('validateNotifications caps items at MAX_NOTIFICATIONS', () => {
  const items: NotificationItem[] = []
  for (let i = 0; i < 150; i++) {
    items.push({
      id: `id-${i}`,
      title: `Title ${i}`,
      message: `Message ${i}`,
      severity: 'info',
      category: 'system',
      timestamp: Date.now() + i,
      read: false
    })
  }

  const result = validateNotifications(items)
  assert.equal(result.length, MAX_NOTIFICATIONS)
  assert.equal(result[0].id, 'id-0')
})

test('notificationStorageService.loadNotifications loads from native storage when present', async () => {
  const originalGetData = storage.getData

  const mockItem: NotificationItem = {
    id: 'n-1',
    title: 'Storage Test',
    message: 'Loaded from native',
    severity: 'success',
    category: 'system',
    timestamp: 123456789,
    read: false
  }

  storage.getData = async (key: string) => {
    if (key === NOTIFICATION_STORAGE_KEY) {
      return JSON.stringify([mockItem])
    }
    throw new Error('Key not found')
  }

  try {
    const loaded = await notificationStorageService.loadNotifications()
    assert.equal(loaded.length, 1)
    assert.deepEqual(loaded[0], mockItem)
  } finally {
    storage.getData = originalGetData
  }
})

test('notificationStorageService.loadNotifications performs legacy migration from localStorage', async () => {
  const originalGetData = storage.getData
  const originalSetData = storage.setData

  const legacyNotifications: NotificationItem[] = [
    {
      id: 'legacy-1',
      title: 'Legacy Alert',
      message: 'From localStorage',
      severity: 'warn',
      category: 'command',
      timestamp: 99999,
      read: true
    }
  ]

  let nativeStorageValue: string | null = null
  let removedKey: string | null = null

  storage.getData = async () => {
    throw new Error('Not found in native storage')
  }

  storage.setData = async (key: string, data?: string | null) => {
    if (key === NOTIFICATION_STORAGE_KEY) {
      nativeStorageValue = data ?? null
    }
  }

  const originalLocalStorage = globalThis.localStorage
  const mockLocalStorage = {
    getItem(key: string) {
      if (key === NOTIFICATION_STORAGE_KEY) {
        return JSON.stringify(legacyNotifications)
      }
      return null
    },
    removeItem(key: string) {
      removedKey = key
    },
    setItem() {}
  }

  Object.defineProperty(globalThis, 'localStorage', {
    value: mockLocalStorage,
    configurable: true,
    writable: true
  })

  try {
    const loaded = await notificationStorageService.loadNotifications()
    assert.equal(loaded.length, 1)
    assert.equal(loaded[0].id, 'legacy-1')
    assert.equal(removedKey, NOTIFICATION_STORAGE_KEY)
    assert.notEqual(nativeStorageValue, null)
    assert.deepEqual(JSON.parse(nativeStorageValue!), legacyNotifications)
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

test('notificationStorageService.saveNotifications writes capped list to native storage', async () => {
  const originalSetData = storage.setData
  let savedKey = ''
  let savedData = ''

  storage.setData = async (key: string, data?: string | null) => {
    savedKey = key
    savedData = data || ''
  }

  const items: NotificationItem[] = []
  for (let i = 0; i < 110; i++) {
    items.push({
      id: `id-${i}`,
      title: `Title ${i}`,
      message: `Message ${i}`,
      severity: 'info',
      category: 'system',
      timestamp: i,
      read: false
    })
  }

  try {
    await notificationStorageService.saveNotifications(items)
    assert.equal(savedKey, NOTIFICATION_STORAGE_KEY)
    const parsed = JSON.parse(savedData) as NotificationItem[]
    assert.equal(parsed.length, MAX_NOTIFICATIONS)
  } finally {
    storage.setData = originalSetData
  }
})

test('notificationStorageService.clearNotifications removes data from native storage', async () => {
  const originalRemoveData = storage.removeData
  let removedKey = ''

  storage.removeData = async (key: string) => {
    removedKey = key
  }

  try {
    await notificationStorageService.clearNotifications()
    assert.equal(removedKey, NOTIFICATION_STORAGE_KEY)
  } finally {
    storage.removeData = originalRemoveData
  }
})
