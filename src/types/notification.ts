export type NotificationSeverity = 'info' | 'success' | 'warn' | 'error'

export type NotificationCategory = 'system' | 'kubernetes' | 'updater' | 'command'

export interface NotificationItem {
  id: string
  title: string
  message: string
  severity: NotificationSeverity
  category: NotificationCategory
  timestamp: number
  read: boolean
}
