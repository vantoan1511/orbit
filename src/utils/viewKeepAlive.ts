import { defineComponent, h, type Component, type VNode } from 'vue'
import type { AppTab } from '@/types/tabs.ts'
import { isEditRoutePath } from './editKeepAlive.ts'

export function viewCacheKey(path: string): string {
  const withoutQuery = path.split('?')[0] ?? ''
  return withoutQuery.split('#')[0] ?? ''
}

export function viewTabCacheName(key: string): string {
  const safeKey = key.replace(/[^a-zA-Z0-9_]/g, '_')
  return `ViewTab_${safeKey}`
}

export function getViewCacheNames(tabs: AppTab[]): string[] {
  const uniqueKeys = new Set<string>()
  for (const tab of tabs) {
    if (!isEditRoutePath(tab.path)) {
      const key = viewCacheKey(tab.path)
      if (key) {
        uniqueKeys.add(key)
      }
    }
  }
  return Array.from(uniqueKeys).map((k) => viewTabCacheName(k))
}

export function resolveViewKey(path: string, fullPath: string): string {
  return isEditRoutePath(path) ? fullPath : path
}

export interface ViewHostRegistry {
  getOrCreateHost: (key: string, baseComponent: Component) => Component
  prune: (openKeys: string[]) => void
  hasHost: (key: string) => boolean
  clearAll: () => void
}

export function createViewHostRegistry(): ViewHostRegistry {
  const hostMap = new Map<string, Component>()

  function getOrCreateHost(key: string, baseComponent: Component): Component {
    const existing = hostMap.get(key)
    if (existing) {
      return existing
    }

    const hostName = viewTabCacheName(key)
    const host = defineComponent({
      name: hostName,
      setup(_props, { attrs, slots }) {
        return (): VNode => h(baseComponent, attrs, slots)
      }
    })

    hostMap.set(key, host)
    return host
  }

  function prune(openKeys: string[]): void {
    const openSet = new Set(openKeys.map((k) => viewCacheKey(k)))
    for (const key of hostMap.keys()) {
      if (!openSet.has(key)) {
        hostMap.delete(key)
      }
    }
  }

  function hasHost(key: string): boolean {
    return hostMap.has(key)
  }

  function clearAll(): void {
    hostMap.clear()
  }

  return {
    getOrCreateHost,
    prune,
    hasHost,
    clearAll
  }
}

export const defaultViewHostRegistry = createViewHostRegistry()
