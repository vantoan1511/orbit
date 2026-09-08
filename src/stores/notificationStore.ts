import {
  MAX_NOTIFICATIONS,
  NOTIFICATION_STORAGE_KEY,
  notificationStorageService
} from '../services/notificationStorageService.ts'
import type { NotificationItem } from '../types/notification.ts'
import { defineStore } from 'pinia'
import { computed, nextTick, ref, watch } from 'vue'

export { MAX_NOTIFICATIONS, NOTIFICATION_STORAGE_KEY }

let saveTimeout: ReturnType<typeof setTimeout> | null = null
let currentNotificationsRef: NotificationItem[] | null = null
let lastPersistedJson = ''

async function flushSave() {
  if (currentNotificationsRef) {
    if (saveTimeout) {
      clearTimeout(saveTimeout)
      saveTimeout = null
    }
    const json = JSON.stringify(currentNotificationsRef)
    if (json !== lastPersistedJson) {
      lastPersistedJson = json
      await notificationStorageService.saveNotifications(currentNotificationsRef)
    }
  }
}

if (typeof window !== 'undefined') {
  window.addEventListener('beforeunload', () => {
    void flushSave()
  })
}

function scheduleSave(items: NotificationItem[], immediate = false) {
  currentNotificationsRef = items
  if (saveTimeout) {
    clearTimeout(saveTimeout)
    saveTimeout = null
  }
  const json = JSON.stringify(items)
  if (json === lastPersistedJson) {
    return
  }
  if (immediate) {
    lastPersistedJson = json
    void notificationStorageService.saveNotifications(items)
    return
  }
  saveTimeout = setTimeout(async () => {
    saveTimeout = null
    if (json !== lastPersistedJson) {
      lastPersistedJson = json
      await notificationStorageService.saveNotifications(items)
    }
  }, 250)
}

export const useNotificationStore = defineStore('notification', () => {
  const notifications = ref<NotificationItem[]>([])
  const isDrawerOpen = ref(false)
  const isInitialized = ref(false)

  // Watch for changes to persist once initialized
  watch(
    notifications,
    (val) => {
      if (isInitialized.value) {
        scheduleSave(val)
      }
    },
    { deep: true }
  )

  async function init(): Promise<void> {
    try {
      const loaded = await notificationStorageService.loadNotifications()
      lastPersistedJson = JSON.stringify(loaded)
      if (notifications.value.length === 0) {
        notifications.value = loaded
      } else {
        // Merge items if some were added before init resolved
        const existingIds = new Set(notifications.value.map((n) => n.id))
        const merged = [...notifications.value]
        let hasNew = false
        for (const item of loaded) {
          if (!existingIds.has(item.id)) {
            merged.push(item)
            existingIds.add(item.id)
            hasNew = true
          }
        }
        notifications.value = merged.slice(0, MAX_NOTIFICATIONS)
        if (hasNew) {
          scheduleSave(notifications.value)
        }
      }
    } catch (e) {
      console.warn('Failed to initialize notifications from native storage:', e)
    } finally {
      await nextTick()
      isInitialized.value = true
    }
  }

  const unreadCount = computed(() => notifications.value.filter((n) => !n.read).length)

  const sortedNotifications = computed(() => {
    return [...notifications.value].sort((a, b) => b.timestamp - a.timestamp)
  })

  function addNotification(item: Omit<NotificationItem, 'id' | 'timestamp' | 'read'>) {
    const id =
      typeof crypto !== 'undefined' && crypto.randomUUID
        ? crypto.randomUUID()
        : `${Date.now()}-${Math.random().toString(36).substring(2, 9)}`

    const newNotification: NotificationItem = {
      ...item,
      id,
      timestamp: Date.now(),
      read: false
    }

    notifications.value = [newNotification, ...notifications.value].slice(0, MAX_NOTIFICATIONS)
    if (isInitialized.value) {
      scheduleSave(notifications.value)
    }
  }

  function markAsRead(id: string) {
    const target = notifications.value.find((n) => n.id === id)
    if (target) {
      target.read = true
    }
  }

  function markAllAsRead() {
    notifications.value.forEach((n) => {
      n.read = true
    })
  }

  function removeNotification(id: string) {
    notifications.value = notifications.value.filter((n) => n.id !== id)
  }

  function clearAll() {
    notifications.value = []
    if (isInitialized.value) {
      scheduleSave([], true)
    }
  }

  function toggleDrawer() {
    isDrawerOpen.value = !isDrawerOpen.value
  }

  function openDrawer() {
    isDrawerOpen.value = true
  }

  function closeDrawer() {
    isDrawerOpen.value = false
  }

  return {
    notifications,
    isDrawerOpen,
    isInitialized,
    unreadCount,
    sortedNotifications,
    init,
    flushSave,
    addNotification,
    markAsRead,
    markAllAsRead,
    removeNotification,
    clearAll,
    toggleDrawer,
    openDrawer,
    closeDrawer
  }
})
