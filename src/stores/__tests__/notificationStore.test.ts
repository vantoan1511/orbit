import assert from 'node:assert/strict'
import test, { beforeEach } from 'node:test'
import { createPinia, setActivePinia } from 'pinia'
import { notificationStorageService } from '../../services/notificationStorageService.ts'
import { useNotificationStore } from '../notificationStore.ts'
import type { NotificationItem } from '@/types/notification'

beforeEach(() => {
  setActivePinia(createPinia())
})

test('notificationStore initializes with empty notifications and unreadCount 0', () => {
  const store = useNotificationStore()
  assert.equal(store.notifications.length, 0)
  assert.equal(store.unreadCount, 0)
  assert.equal(store.isInitialized, false)
})

test('notificationStore.addNotification adds notification with id and timestamp', () => {
  const store = useNotificationStore()

  store.addNotification({
    title: 'Hello',
    message: 'World',
    severity: 'info',
    category: 'system'
  })

  assert.equal(store.notifications.length, 1)
  assert.equal(store.notifications[0].title, 'Hello')
  assert.equal(store.notifications[0].read, false)
  assert.equal(typeof store.notifications[0].id, 'string')
  assert.equal(typeof store.notifications[0].timestamp, 'number')
  assert.equal(store.unreadCount, 1)
})

test('notificationStore.init loads notifications from notificationStorageService', async () => {
  const originalLoad = notificationStorageService.loadNotifications
  const mockItems: NotificationItem[] = [
    {
      id: 'stored-1',
      title: 'Stored Alert',
      message: 'From service',
      severity: 'warn',
      category: 'kubernetes',
      timestamp: 12345,
      read: false
    }
  ]

  notificationStorageService.loadNotifications = async () => mockItems

  try {
    const store = useNotificationStore()
    await store.init()

    assert.equal(store.isInitialized, true)
    assert.equal(store.notifications.length, 1)
    assert.equal(store.notifications[0].id, 'stored-1')
    assert.equal(store.unreadCount, 1)
  } finally {
    notificationStorageService.loadNotifications = originalLoad
  }
})

test('notificationStore markAsRead, markAllAsRead, removeNotification, and clearAll work correctly', () => {
  const store = useNotificationStore()

  store.addNotification({
    title: 'Note 1',
    message: 'M1',
    severity: 'info',
    category: 'system'
  })
  store.addNotification({
    title: 'Note 2',
    message: 'M2',
    severity: 'warn',
    category: 'kubernetes'
  })

  assert.equal(store.unreadCount, 2)
  const id1 = store.notifications[1].id
  const id2 = store.notifications[0].id

  store.markAsRead(id1)
  assert.equal(store.unreadCount, 1)

  store.markAllAsRead()
  assert.equal(store.unreadCount, 0)

  store.removeNotification(id2)
  assert.equal(store.notifications.length, 1)

  store.clearAll()
  assert.equal(store.notifications.length, 0)
})
