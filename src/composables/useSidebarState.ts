import { ref, type ComputedRef, type Ref } from 'vue'
import { storage } from '../services/nativeService.ts'
import { DEFAULT_STORAGE_TIMEOUT_MS, withTimeout } from '../utils/async.ts'
import { RESOURCE_BREADCRUMB_MAP } from '../utils/breadcrumb.ts'
import type { CategoryId, SidebarCategory } from '../components/layout/sidebar/navigation.ts'

export interface UseSidebarStateOptions {
  storageKey?: string
  currentPath?: Ref<string> | ComputedRef<string>
  hasActiveCluster?: Ref<boolean> | ComputedRef<boolean>
  onNavigate?: (path: string) => void
}

export function resolveCategoryForPath(
  path: string,
  routeName?: string,
  routeKind?: string
): CategoryId | null {
  if (routeName === 'edit-workload' && routeKind && RESOURCE_BREADCRUMB_MAP[routeKind]) {
    return RESOURCE_BREADCRUMB_MAP[routeKind].categoryId
  }
  if (path === '/logs') return 'logs'
  if (path === '/' || path === '/nodes' || path === '/namespaces' || path === '/events')
    return 'core'
  if (path.startsWith('/workloads') || path === '/pods') return 'workloads'
  if (path.startsWith('/network')) return 'network'
  if (path.startsWith('/storage')) return 'storage'
  if (path.startsWith('/config')) return 'config'
  if (path.startsWith('/policies')) return 'security'
  return null
}

export function useSidebarState(options: UseSidebarStateOptions = {}) {
  const storageKey = options.storageKey ?? 'orbit_sidebar_collapsed'
  const isCollapsed = ref<boolean>(false)
  const activeTab = ref<CategoryId | null>('clusters')

  const hasCluster = options.hasActiveCluster ?? ref(false)

  const persistCollapsedState = async (collapsed: boolean): Promise<void> => {
    if (!storageKey) return
    try {
      await storage.setData(storageKey, collapsed ? 'true' : 'false')
    } catch (e) {
      console.warn(`Failed to persist sidebar collapsed state for ${storageKey}:`, e)
    }
  }

  const loadStoredState = async (): Promise<void> => {
    if (!storageKey) return
    try {
      let raw: string | null = null
      try {
        raw = await withTimeout(storage.getData(storageKey), DEFAULT_STORAGE_TIMEOUT_MS)
      } catch {
        // Fallback check for legacy localStorage
        if (typeof localStorage !== 'undefined') {
          try {
            const legacy = localStorage.getItem(storageKey)
            if (legacy) {
              raw = legacy
              try {
                await storage.setData(storageKey, legacy)
                localStorage.removeItem(storageKey)
              } catch (migrationErr) {
                console.warn(
                  `Failed to write migrated sidebar state for ${storageKey} to native storage:`,
                  migrationErr
                )
              }
            }
          } catch {
            // localStorage not accessible
          }
        }
      }

      if (raw === 'true') {
        isCollapsed.value = true
        activeTab.value = null
      } else if (raw === 'false') {
        isCollapsed.value = false
        if (hasCluster.value && options.currentPath?.value) {
          const category = resolveCategoryForPath(options.currentPath.value)
          if (category) {
            activeTab.value = category
          }
        }
      }
    } catch (e) {
      console.warn(`Failed to load stored sidebar state for ${storageKey}:`, e)
    }
  }

  const syncWithRoute = (path: string, routeName?: string, routeKind?: string): void => {
    // Invariant: when collapsed, switching routes or tabs MUST NOT auto-expand the sidebar
    if (isCollapsed.value) {
      return
    }

    const category = resolveCategoryForPath(path, routeName, routeKind)
    if (category && activeTab.value !== 'clusters' && hasCluster.value) {
      activeTab.value = category
    }
  }

  const toggleCategory = (cat: SidebarCategory): void => {
    if (cat.requiresCluster && !hasCluster.value) {
      return
    }

    if (!isCollapsed.value && cat.id === activeTab.value) {
      // Collapse sidebar when active category is clicked
      isCollapsed.value = true
      activeTab.value = null
      void persistCollapsedState(true)
      return
    }

    // Expand or switch category
    isCollapsed.value = false
    activeTab.value = cat.id
    void persistCollapsedState(false)

    if (cat.defaultPath && options.onNavigate) {
      options.onNavigate(cat.defaultPath)
    }
  }

  const handleCollapse = (): void => {
    isCollapsed.value = true
    activeTab.value = null
    void persistCollapsedState(true)
  }

  const handleClusterSwitched = (path: string, routeName?: string, routeKind?: string): void => {
    if (!isCollapsed.value) {
      activeTab.value = resolveCategoryForPath(path, routeName, routeKind) || 'core'
    }
  }

  return {
    isCollapsed,
    activeTab,
    toggleCategory,
    handleCollapse,
    handleClusterSwitched,
    syncWithRoute,
    loadStoredState,
    persistCollapsedState
  }
}
