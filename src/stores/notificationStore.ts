import type { NotificationItem } from '@/types/notification'
import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'

const STORAGE_KEY = 'orbit_notifications'
const MAX_NOTIFICATIONS = 100

export const useNotificationStore = defineStore('notification', () => {
  const notifications = ref<NotificationItem[]>([])
  const isDrawerOpen = ref(false)

  // Initialize from localStorage safely
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored) {
      notifications.value = JSON.parse(stored)
    }
  } catch (e) {
    console.error('Failed to load notifications from localStorage:', e)
    notifications.value = []
  }

  // Watch for changes to persist
  watch(
    notifications,
    (val) => {
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(val))
      } catch (e) {
        console.error('Failed to save notifications to localStorage:', e)
      }
    },
    { deep: true }
  )

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
    unreadCount,
    sortedNotifications,
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
