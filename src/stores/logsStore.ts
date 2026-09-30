import { storage } from '../services/nativeService.ts'
import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface RecentLogInfo {
  namespace: string
  workloadKind: string
  workloadName: string
  pod?: string
  container?: string
  timestamp: number
  clusterId?: string
}

export const BASE_LOGS_STORAGE_KEY = 'orbit_logs_recent_logs'
export const MAX_RECENT_LOGS = 15

export function getLogsStorageKey(clusterId?: string | null): string {
  if (clusterId && clusterId.trim()) {
    return `${BASE_LOGS_STORAGE_KEY}_${clusterId.trim()}`
  }
  return BASE_LOGS_STORAGE_KEY
}

export const useLogsStore = defineStore('logs', () => {
  const recentLogs = ref<RecentLogInfo[]>([])
  const activeClusterId = ref<string | null>(null)

  async function loadRecentLogs(clusterId?: string | null) {
    const key = getLogsStorageKey(clusterId !== undefined ? clusterId : activeClusterId.value)
    try {
      const data = await storage.getData(key)
      if (data) {
        const parsed = JSON.parse(data)
        if (Array.isArray(parsed)) {
          recentLogs.value = parsed
          return
        }
      }
      recentLogs.value = []
    } catch (e) {
      console.warn('Failed to load recent logs from native storage:', e)
      recentLogs.value = []
    }
  }

  async function setClusterId(clusterId: string | null) {
    if (activeClusterId.value === clusterId && recentLogs.value.length > 0) return
    activeClusterId.value = clusterId
    recentLogs.value = []
    if (clusterId && clusterId.trim()) {
      await loadRecentLogs(clusterId)
    }
  }

  async function addRecentLog(log: Omit<RecentLogInfo, 'timestamp'>) {
    const targetClusterId = log.clusterId ?? activeClusterId.value

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
      timestamp: Date.now(),
      ...(targetClusterId ? { clusterId: targetClusterId } : {})
    }

    recentLogs.value = [newLog, ...filtered].slice(0, MAX_RECENT_LOGS)

    try {
      const key = getLogsStorageKey(targetClusterId)
      await storage.setData(key, JSON.stringify(recentLogs.value))
    } catch (e) {
      console.warn('Failed to save recent logs to native storage:', e)
    }
  }

  async function clearRecentLogs() {
    recentLogs.value = []
    try {
      const key = getLogsStorageKey(activeClusterId.value)
      await storage.setData(key, JSON.stringify([]))
    } catch (e) {
      console.warn('Failed to clear recent logs from native storage:', e)
    }
  }

  return {
    recentLogs,
    activeClusterId,
    setClusterId,
    loadRecentLogs,
    addRecentLog,
    clearRecentLogs
  }
})
