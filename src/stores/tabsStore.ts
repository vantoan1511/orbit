import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import type { AppTab, TabsStorageState } from '@/types/tabs.ts'
import { tabStorageService } from '../services/tabStorageService.ts'
import { getTabMetadataForRoute } from '../utils/tabIcons.ts'

function parseRoute(routeStr: string): { path: string; query?: Record<string, string> } {
  const [pathPart, queryPart] = routeStr.split('?')
  const path = pathPart || '/'
  if (!queryPart) {
    return { path }
  }
  const query: Record<string, string> = {}
  const searchParams = new URLSearchParams(queryPart)
  searchParams.forEach((value, key) => {
    query[key] = value
  })
  return { path, query }
}

export const useTabsStore = defineStore('tabs', () => {
  const tabs = ref<AppTab[]>([])
  const activeTabId = ref<string | null>(null)
  const currentClusterId = ref<string | undefined>(undefined)
  const isHydrated = ref<boolean>(false)

  const activeTab = computed<AppTab | null>(() => {
    if (!activeTabId.value) return null
    return tabs.value.find((t) => t.id === activeTabId.value) ?? null
  })

  const activeTabIndex = computed<number>(() => {
    if (!activeTabId.value) return -1
    return tabs.value.findIndex((t) => t.id === activeTabId.value)
  })

  let saveTimer: ReturnType<typeof setTimeout> | null = null

  function scheduleSave() {
    if (saveTimer) {
      clearTimeout(saveTimer)
    }
    saveTimer = setTimeout(() => {
      const state: TabsStorageState = {
        tabs: tabs.value,
        activeTabId: activeTabId.value
      }
      void tabStorageService.saveTabsState(state, currentClusterId.value)
    }, 150)
  }

  function openTab(target: {
    route: string
    title?: string
    iconName?: string
    category?: string
    closable?: boolean
  }): AppTab {
    const { path, query } = parseRoute(target.route)
    const metadata = getTabMetadataForRoute(path, query)

    const resolvedTitle =
      target.title && target.title.trim() && target.title !== 'Tab' ? target.title : metadata.title
    const resolvedIconName = target.iconName || metadata.iconName
    const resolvedCategory = target.category || metadata.category

    const existing = tabs.value.find((t) => t.route === target.route || t.id === target.route)
    if (existing) {
      activeTabId.value = existing.id
      if (
        existing.title === 'Tab' ||
        !existing.title ||
        (existing.path === '/logs' && existing.title === 'Logs' && resolvedTitle !== 'Logs')
      ) {
        existing.title = resolvedTitle
      }
      if (!existing.iconName) {
        existing.iconName = resolvedIconName
      }
      if (!existing.category) {
        existing.category = resolvedCategory
      }
      scheduleSave()
      return existing
    }

    const newTab: AppTab = {
      id: target.route,
      title: resolvedTitle,
      route: target.route,
      path,
      query,
      iconName: resolvedIconName,
      category: resolvedCategory,
      closable: target.closable ?? true
    }

    tabs.value.push(newTab)
    activeTabId.value = newTab.id
    scheduleSave()
    return newTab
  }

  function setActiveTab(tabId: string): void {
    if (tabs.value.some((t) => t.id === tabId)) {
      activeTabId.value = tabId
      scheduleSave()
    }
  }

  function closeTab(tabId: string): void {
    const index = tabs.value.findIndex((t) => t.id === tabId)
    if (index === -1) return

    const isClosingActive = activeTabId.value === tabId

    if (isClosingActive) {
      if (tabs.value.length > 1) {
        if (index < tabs.value.length - 1) {
          activeTabId.value = tabs.value[index + 1]!.id
        } else {
          activeTabId.value = tabs.value[index - 1]!.id
        }
      } else {
        activeTabId.value = null
      }
    }

    tabs.value.splice(index, 1)
    scheduleSave()
  }

  function closeOtherTabs(tabId: string): void {
    const target = tabs.value.find((t) => t.id === tabId)
    if (!target) return

    tabs.value = [target]
    activeTabId.value = target.id
    scheduleSave()
  }

  function closeTabsToTheRight(tabId: string): void {
    const index = tabs.value.findIndex((t) => t.id === tabId)
    if (index === -1) return

    const remaining = tabs.value.slice(0, index + 1)
    tabs.value = remaining

    if (activeTabId.value && !remaining.some((t) => t.id === activeTabId.value)) {
      activeTabId.value = tabId
    }
    scheduleSave()
  }

  function closeAllTabs(): void {
    tabs.value = []
    activeTabId.value = null
    scheduleSave()
  }

  function reorderTabs(fromIndex: number, toIndex: number): void {
    if (
      fromIndex < 0 ||
      fromIndex >= tabs.value.length ||
      toIndex < 0 ||
      toIndex >= tabs.value.length
    ) {
      return
    }
    const [moved] = tabs.value.splice(fromIndex, 1)
    if (moved) {
      tabs.value.splice(toIndex, 0, moved)
      scheduleSave()
    }
  }

  function moveTabLeft(tabId: string): void {
    const index = tabs.value.findIndex((t) => t.id === tabId)
    if (index > 0) {
      reorderTabs(index, index - 1)
    }
  }

  function moveTabRight(tabId: string): void {
    const index = tabs.value.findIndex((t) => t.id === tabId)
    if (index >= 0 && index < tabs.value.length - 1) {
      reorderTabs(index, index + 1)
    }
  }

  function syncWithRoute(
    routePath: string,
    routeQuery?: Record<string, string>,
    title?: string,
    iconName?: string,
    category?: string
  ): AppTab | null {
    // Skip empty or transient routes
    if (!routePath || routePath === '/welcome') {
      return null
    }

    const queryParts = routeQuery
      ? Object.entries(routeQuery)
          .map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`)
          .join('&')
      : ''
    const fullRoute = queryParts ? `${routePath}?${queryParts}` : routePath

    const meta = getTabMetadataForRoute(routePath, routeQuery)
    const resolvedTitle = title && title.trim() && title !== 'Tab' ? title : meta.title
    const resolvedIconName = iconName || meta.iconName
    const resolvedCategory = category || meta.category

    return openTab({
      route: fullRoute,
      title: resolvedTitle,
      iconName: resolvedIconName,
      category: resolvedCategory
    })
  }

  async function init(clusterId?: string): Promise<void> {
    currentClusterId.value = clusterId
    try {
      const state = await tabStorageService.loadTabsState(clusterId)
      if (state) {
        tabs.value = state.tabs.map((t) => {
          const isLegacyTab = !t.title || t.title === 'Tab'
          const isUnresolvedLogTab =
            t.path === '/logs' && t.title === 'Logs' && Boolean(t.query?.workload || t.query?.pod)
          if (isLegacyTab || isUnresolvedLogTab) {
            const meta = getTabMetadataForRoute(t.path, t.query)
            return {
              ...t,
              title: meta.title,
              iconName: t.iconName || meta.iconName,
              category: t.category || meta.category
            }
          }
          return t
        })
        activeTabId.value = state.activeTabId
      }
    } finally {
      isHydrated.value = true
    }
  }

  return {
    tabs,
    activeTabId,
    activeTab,
    activeTabIndex,
    isHydrated,
    openTab,
    setActiveTab,
    closeTab,
    closeOtherTabs,
    closeTabsToTheRight,
    closeAllTabs,
    reorderTabs,
    moveTabLeft,
    moveTabRight,
    syncWithRoute,
    init
  }
})
