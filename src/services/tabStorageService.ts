import type { AppTab, TabsStorageState } from '@/types/tabs.ts'
import { storage } from './nativeService.ts'
import { DEFAULT_STORAGE_TIMEOUT_MS, withTimeout } from '../utils/async.ts'

export const BASE_TABS_STORAGE_KEY = 'orbit_app_tabs'
export const STORAGE_TIMEOUT_MS = DEFAULT_STORAGE_TIMEOUT_MS

export function getTabsStorageKey(clusterId?: string): string {
  if (clusterId && clusterId.trim()) {
    return `${BASE_TABS_STORAGE_KEY}_${clusterId.trim()}`
  }
  return BASE_TABS_STORAGE_KEY
}

export function createDefaultTabsState(): TabsStorageState {
  return {
    tabs: [],
    activeTabId: null
  }
}

function isValidTab(item: unknown): item is AppTab {
  if (!item || typeof item !== 'object') return false
  const candidate = item as Partial<AppTab>
  return (
    typeof candidate.id === 'string' &&
    typeof candidate.title === 'string' &&
    typeof candidate.route === 'string' &&
    typeof candidate.path === 'string'
  )
}

export function validateTabsState(parsed: unknown): TabsStorageState {
  if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) {
    const candidate = parsed as Partial<TabsStorageState>
    const rawTabs = Array.isArray(candidate.tabs) ? candidate.tabs : []
    const validTabs: AppTab[] = rawTabs.filter(isValidTab).map((t) => {
      const tab: AppTab = {
        id: t.id,
        title: t.title,
        route: t.route,
        path: t.path,
        closable: typeof t.closable === 'boolean' ? t.closable : true
      }
      if (t.query && typeof t.query === 'object') tab.query = t.query
      if (typeof t.iconName === 'string') tab.iconName = t.iconName
      if (typeof t.category === 'string') tab.category = t.category
      return tab
    })

    const activeTabId =
      typeof candidate.activeTabId === 'string' &&
      validTabs.some((t) => t.id === candidate.activeTabId)
        ? candidate.activeTabId
        : (validTabs[0]?.id ?? null)

    return {
      tabs: validTabs,
      activeTabId
    }
  }

  return createDefaultTabsState()
}

export const tabStorageService = {
  /**
   * Load persisted tabs state from native desktop storage.
   */
  async loadTabsState(clusterId?: string): Promise<TabsStorageState | null> {
    const key = getTabsStorageKey(clusterId)
    try {
      if (typeof window !== 'undefined' && !(window as unknown as { NL_PORT?: number }).NL_PORT) {
        throw new Error('Neutralino runtime not available')
      }
      const raw = await withTimeout(storage.getData(key), STORAGE_TIMEOUT_MS)
      if (raw) {
        const parsed = JSON.parse(raw)
        return validateTabsState(parsed)
      }
    } catch {
      // Native storage key not found, timeout, or runtime absent
    }

    return null
  },

  /**
   * Persist tabs state to native desktop storage.
   */
  async saveTabsState(state: TabsStorageState, clusterId?: string): Promise<void> {
    const key = getTabsStorageKey(clusterId)
    try {
      await storage.setData(key, JSON.stringify(state))
    } catch (e) {
      console.warn('Failed to save tabs state to native storage:', e)
    }
  },

  /**
   * Remove persisted tabs state from native desktop storage.
   */
  async clearTabsState(clusterId?: string): Promise<void> {
    const key = getTabsStorageKey(clusterId)
    try {
      await storage.removeData(key)
    } catch (e) {
      console.warn('Failed to remove tabs state from native storage:', e)
    }
  }
}
