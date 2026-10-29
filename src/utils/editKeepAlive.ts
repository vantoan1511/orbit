import { defineComponent, h, type Component, type VNode } from 'vue'
import type { AppTab } from '@/types/tabs.ts'

const EDIT_ROUTE_REGEX = /^\/workloads\/[^/]+\/[^/]+\/[^/]+\/edit\/?$/

export function isEditRoutePath(path: string): boolean {
  return EDIT_ROUTE_REGEX.test(path)
}

export function editTabCacheName(tabId: string): string {
  const safeId = tabId.replace(/[^a-zA-Z0-9_]/g, '_')
  return `EditTab_${safeId}`
}

export function getEditCacheNames(tabs: AppTab[]): string[] {
  return tabs.filter((tab) => isEditRoutePath(tab.path)).map((tab) => editTabCacheName(tab.id))
}

export interface EditHostRegistry {
  getOrCreateEditHost: (tabId: string, baseComponent: Component) => Component
  pruneEditHosts: (openTabIds: string[]) => void
  hasEditHost: (tabId: string) => boolean
  clearAll: () => void
}

export function createEditHostRegistry(): EditHostRegistry {
  const hostMap = new Map<string, Component>()

  function getOrCreateEditHost(tabId: string, baseComponent: Component): Component {
    const existing = hostMap.get(tabId)
    if (existing) {
      return existing
    }

    const hostName = editTabCacheName(tabId)
    const host = defineComponent({
      name: hostName,
      props: ['kind', 'namespace', 'name'],
      setup(props, { attrs, slots }) {
        return (): VNode => h(baseComponent, { ...props, ...attrs }, slots)
      }
    })

    hostMap.set(tabId, host)
    return host
  }

  function pruneEditHosts(openTabIds: string[]): void {
    const openSet = new Set(openTabIds)
    for (const id of hostMap.keys()) {
      if (!openSet.has(id)) {
        hostMap.delete(id)
      }
    }
  }

  function hasEditHost(tabId: string): boolean {
    return hostMap.has(tabId)
  }

  function clearAll(): void {
    hostMap.clear()
  }

  return {
    getOrCreateEditHost,
    pruneEditHosts,
    hasEditHost,
    clearAll
  }
}

export const defaultEditHostRegistry = createEditHostRegistry()
