import { storage } from './nativeService.ts'
import type {
  NotificationItem,
  NotificationSeverity,
  NotificationCategory
} from '../types/notification.ts'
import { DEFAULT_STORAGE_TIMEOUT_MS, withTimeout } from '../utils/async.ts'

export const NOTIFICATION_STORAGE_KEY = 'orbit_notifications'
export const MAX_NOTIFICATIONS = 100
export const STORAGE_TIMEOUT_MS = DEFAULT_STORAGE_TIMEOUT_MS

const VALID_SEVERITIES: Set<NotificationSeverity> = new Set(['info', 'success', 'warn', 'error'])
const VALID_CATEGORIES: Set<NotificationCategory> = new Set([
  'system',
  'kubernetes',
  'updater',
  'command'
])

function isValidNotificationItem(item: unknown): item is NotificationItem {
  if (!item || typeof item !== 'object') return false
  const candidate = item as Record<string, unknown>
  return (
    typeof candidate.id === 'string' &&
    typeof candidate.title === 'string' &&
    typeof candidate.message === 'string' &&
    VALID_SEVERITIES.has(candidate.severity as NotificationSeverity) &&
    VALID_CATEGORIES.has(candidate.category as NotificationCategory) &&
    typeof candidate.timestamp === 'number' &&
    typeof candidate.read === 'boolean'
  )
}

export function validateNotifications(parsed: unknown): NotificationItem[] {
  if (!Array.isArray(parsed)) {
    return []
  }

  const validItems: NotificationItem[] = []
  for (const item of parsed) {
    if (isValidNotificationItem(item)) {
      validItems.push(item)
      if (validItems.length >= MAX_NOTIFICATIONS) {
        break
      }
    }
  }

  return validItems
}

export const notificationStorageService = {
  /**
   * Load persisted notifications from native desktop storage,
   * falling back to a one-time migration from legacy localStorage if present.
   */
  async loadNotifications(): Promise<NotificationItem[]> {
    try {
      let raw: string | null = null

      try {
        if (typeof window !== 'undefined' && !(window as unknown as { NL_PORT?: number }).NL_PORT) {
          throw new Error('Neutralino runtime not available')
        }
        raw = await withTimeout(storage.getData(NOTIFICATION_STORAGE_KEY), STORAGE_TIMEOUT_MS)
      } catch {
        // Native storage key not found or uninitialized runtime.
        // Fall back to check for legacy localStorage data
        if (typeof localStorage !== 'undefined') {
          try {
            const legacy = localStorage.getItem(NOTIFICATION_STORAGE_KEY)
            if (legacy) {
              raw = legacy
              // Migrate to native desktop storage and clean up legacy localStorage key
              try {
                await storage.setData(NOTIFICATION_STORAGE_KEY, legacy)
                localStorage.removeItem(NOTIFICATION_STORAGE_KEY)
              } catch (migrationErr) {
                console.warn(
                  'Failed to write migrated notifications to native storage:',
                  migrationErr
                )
              }
            }
          } catch {
            // localStorage not accessible or restricted
          }
        }
      }

      if (raw) {
        const parsed = JSON.parse(raw)
        return validateNotifications(parsed)
      }
    } catch (e) {
      console.warn('Failed to load notifications from native storage:', e)
    }

    return []
  },

  /**
   * Persist notifications to native desktop storage.
   */
  async saveNotifications(notifications: NotificationItem[]): Promise<void> {
    try {
      const capped = notifications.slice(0, MAX_NOTIFICATIONS)
      await storage.setData(NOTIFICATION_STORAGE_KEY, JSON.stringify(capped))
    } catch (e) {
      console.warn('Failed to save notifications to native storage:', e)
    }
  },

  /**
   * Remove persisted notifications from native desktop storage.
   */
  async clearNotifications(): Promise<void> {
    try {
      await storage.removeData(NOTIFICATION_STORAGE_KEY)
    } catch (e) {
      console.warn('Failed to remove notifications from native storage:', e)
    }
  }
}
