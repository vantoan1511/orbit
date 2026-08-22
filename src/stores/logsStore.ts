import { storage } from '@/services/nativeService'
import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface RecentLogInfo {
  namespace: string
  workloadKind: string
  workloadName: string
  pod?: string
  container?: string
  timestamp: number
}

const STORAGE_KEY = 'orbit_logs_recent_logs'
const MAX_RECENT_LOGS = 15

export const useLogsStore = defineStore('logs', () => {
  const recentLogs = ref<RecentLogInfo[]>([])

  async function loadRecentLogs() {
    try {
      const data = await storage.getData(STORAGE_KEY)
      if (data) {
        const parsed = JSON.parse(data)
        if (Array.isArray(parsed)) {
          recentLogs.value = parsed
        }
      }
    } catch (e) {
      console.warn('Failed to load recent logs from native storage:', e)
      recentLogs.value = []
    }
  }

  async function addRecentLog(log: Omit<RecentLogInfo, 'timestamp'>) {
    // Filter out existing duplicate entry for namespace + workloadKind + workloadName
    const filtered = recentLogs.value.filter(
      (item) =>
        !(
          item.namespace === log.namespace &&
          item.workloadKind === log.workloadKind &&
          item.workloadName === log.workloadName
        )
    )

    const newLog: RecentLogInfo = {
      ...log,
      timestamp: Date.now()
    }

    recentLogs.value = [newLog, ...filtered].slice(0, MAX_RECENT_LOGS)

    try {
      await storage.setData(STORAGE_KEY, JSON.stringify(recentLogs.value))
    } catch (e) {
      console.warn('Failed to save recent logs to native storage:', e)
    }
  }

  async function clearRecentLogs() {
    recentLogs.value = []
    try {
      await storage.setData(STORAGE_KEY, JSON.stringify([]))
    } catch (e) {
      console.warn('Failed to clear recent logs from native storage:', e)
    }
  }

  return {
    recentLogs,
    loadRecentLogs,
    addRecentLog,
    clearRecentLogs
  }
})
