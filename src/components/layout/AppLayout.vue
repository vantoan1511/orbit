<script setup lang="ts">
import { useCluster } from '@/composables/useCluster.ts'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { useShortcutStore } from '@/stores/shortcutStore'
import { useTabsStore } from '@/stores/tabsStore'
import { useTerminalStore } from '@/stores/terminalStore'
import OfflineClusterView from '@/views/OfflineClusterView.vue'
import WelcomeView from '@/views/WelcomeView.vue'
import EditView from '@/views/EditView.vue'
import { computed, onMounted, onUnmounted, ref, watch, type Component as VueComponent } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { getTabMetadataForRoute } from '@/utils/tabIcons'
import { defaultEditHostRegistry, getEditCacheNames, isEditRoutePath } from '@/utils/editKeepAlive'
import { defaultViewHostRegistry, getViewCacheNames, resolveViewKey } from '@/utils/viewKeepAlive'
import {
  getLayoutContentPaddingClass,
  getLayoutMainClass,
  resolveRouteLayoutMeta,
  type LayoutRouteMeta
} from '@/utils/layout'
import AppFooter from './AppFooter.vue'
import AppLoadingScreen from './AppLoadingScreen.vue'
import AppSidebar from './AppSidebar.vue'
import AppTabBar from './tabs/AppTabBar.vue'
import AppTabResourcePicker from './tabs/AppTabResourcePicker.vue'
import AppTitleBar from './AppTitleBar.vue'
import EmptyWorkspaceView from './tabs/EmptyWorkspaceView.vue'
import TerminalPanel from '@/components/terminal/TerminalPanel.vue'

const k8sStore = useKubernetesStore()
const tabsStore = useTabsStore()
const { activeCluster } = useCluster()
const route = useRoute()
const router = useRouter()

const emptyPickerRef = ref<InstanceType<typeof AppTabResourcePicker> | null>(null)
const isSwitchingCluster = ref(false)

const currentLayoutMeta = computed(() => {
  return resolveRouteLayoutMeta(route.path, route.meta as LayoutRouteMeta)
})

const editCacheNames = computed(() => getEditCacheNames(tabsStore.tabs))
const viewCacheNames = computed(() => getViewCacheNames(tabsStore.tabs))

watch(
  () => tabsStore.tabs.map((t) => t.id + t.path).join('|'),
  () => {
    defaultEditHostRegistry.pruneEditHosts(tabsStore.tabs.map((t) => t.id))
    defaultViewHostRegistry.prune(tabsStore.tabs.map((t) => t.path))
  }
)

function resolveViewComponent(Component: unknown, currentRoute: typeof route) {
  if (isEditRoutePath(currentRoute.path)) {
    const tabId = currentRoute.fullPath
    return defaultEditHostRegistry.getOrCreateEditHost(tabId, EditView)
  }
  return defaultViewHostRegistry.getOrCreateHost(currentRoute.path, Component as VueComponent)
}

// Synchronize current route with tabsStore
function syncRouteWithTabs(newPath: string, newQuery?: Record<string, string>) {
  if (!isSwitchingCluster.value && k8sStore.activeClusterId !== null && newPath !== '/welcome') {
    const meta = getTabMetadataForRoute(newPath, newQuery)
    tabsStore.syncWithRoute(newPath, newQuery, meta.title, meta.iconName, meta.category)
  }
}

watch(
  () => [route.path, route.query],
  ([newPath, newQuery]) => {
    syncRouteWithTabs(newPath as string, newQuery as Record<string, string>)
  },
  { immediate: true }
)

const unregisterAfterEach = router.afterEach((to) => {
  if (tabsStore.tabs.length === 0) {
    syncRouteWithTabs(to.path, to.query as Record<string, string>)
  }
})

// Rehydrate tabs on active cluster change
watch(
  () => k8sStore.activeClusterId,
  async (newClusterId) => {
    defaultViewHostRegistry.clearAll()
    defaultEditHostRegistry.clearAll()
    if (newClusterId) {
      isSwitchingCluster.value = true
      try {
        void kubernetesService.stopLogs()
        await tabsStore.init(newClusterId)
        if (tabsStore.activeTab) {
          await router.push(tabsStore.activeTab.route)
        } else if (route.path !== '/settings' && route.path !== '/welcome') {
          await router.push('/')
        }
      } finally {
        isSwitchingCluster.value = false
      }
    }
  }
)

const handleOpenEmptyPicker = (event: MouseEvent) => {
  emptyPickerRef.value?.toggle(event)
}

const handleEmptyResourceSelected = (payload: {
  route: string
  title: string
  iconName?: string
  category: string
}) => {
  const tab = tabsStore.openTab(payload)
  void router.push(tab.route)
}

const shortcutStore = useShortcutStore()
const terminalStore = useTerminalStore()

const handleKeyDown = (e: KeyboardEvent) => {
  const activeEl = document.activeElement as HTMLElement | null
  const isInputActive = Boolean(
    activeEl &&
    (activeEl.tagName === 'INPUT' ||
      activeEl.tagName === 'TEXTAREA' ||
      activeEl.classList.contains('monaco-editor') ||
      activeEl.isContentEditable)
  )

  // Don't intercept typing in inputs unless it's a window/navigation action with modifier
  if (isInputActive && !e.ctrlKey && !e.metaKey && !e.altKey) {
    return
  }

  // 1. Close Active Tab
  if (shortcutStore.matchesAction(e, 'closeTab')) {
    if (isInputActive && (activeEl?.tagName === 'INPUT' || activeEl?.tagName === 'TEXTAREA')) {
      return
    }
    if (tabsStore.activeTabId) {
      e.preventDefault()
      const nextRoute = tabsStore.closeTabAndResolveRoute(tabsStore.activeTabId)
      if (nextRoute) {
        void router.push(nextRoute)
      }
    }
    return
  }

  // 2. Next Tab
  if (shortcutStore.matchesAction(e, 'nextTab')) {
    e.preventDefault()
    const activeIndex = tabsStore.tabs.findIndex((t) => t.id === tabsStore.activeTabId)
    if (activeIndex !== -1 && tabsStore.tabs.length > 1) {
      const nextIndex = (activeIndex + 1) % tabsStore.tabs.length
      const nextTab = tabsStore.tabs[nextIndex]
      if (nextTab) {
        tabsStore.setActiveTab(nextTab.id)
        void router.push(nextTab.route)
      }
    }
    return
  }

  // 3. Previous Tab
  if (shortcutStore.matchesAction(e, 'prevTab')) {
    e.preventDefault()
    const activeIndex = tabsStore.tabs.findIndex((t) => t.id === tabsStore.activeTabId)
    if (activeIndex !== -1 && tabsStore.tabs.length > 1) {
      const prevIndex = (activeIndex - 1 + tabsStore.tabs.length) % tabsStore.tabs.length
      const prevTab = tabsStore.tabs[prevIndex]
      if (prevTab) {
        tabsStore.setActiveTab(prevTab.id)
        void router.push(prevTab.route)
      }
    }
    return
  }

  // 4. Toggle Sidebar
  if (shortcutStore.matchesAction(e, 'toggleSidebar')) {
    e.preventDefault()
    window.dispatchEvent(new CustomEvent('orbit:toggle-sidebar'))
    return
  }

  // 5. Toggle Integrated Terminal
  if (shortcutStore.matchesAction(e, 'toggleTerminal')) {
    e.preventDefault()
    terminalStore.togglePanel()
    return
  }

  // 6. Open Settings
  if (shortcutStore.matchesAction(e, 'openSettings')) {
    e.preventDefault()
    void router.push('/settings')
    return
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
  unregisterAfterEach()
  window.removeEventListener('keydown', handleKeyDown)
})
</script>

<template>
  <div
    data-component="app-layout"
    class="flex flex-col h-screen w-screen overflow-hidden text-primary font-sans bg-(--bg-app)"
  >
    <!-- Custom Borderless Window Title Bar -->
    <AppTitleBar />

    <Transition name="page" mode="out-in">
      <AppLoadingScreen
        v-if="k8sStore.isAppLoading"
        class="flex-1 h-auto w-full"
        @complete="k8sStore.setAppLoading(false)"
      />
      <div v-else class="flex-1 flex flex-col overflow-hidden">
        <div class="flex-1 flex overflow-hidden">
          <!-- Sidebar -->
          <AppSidebar />

          <!-- Main Content Area -->
          <div class="flex-1 flex flex-col h-full overflow-hidden">
            <!-- Top Main Tabs -->
            <AppTabBar v-if="k8sStore.activeClusterId !== null" />

            <main :class="getLayoutMainClass(currentLayoutMeta)">
              <div :class="getLayoutContentPaddingClass(currentLayoutMeta)">
                <template v-if="k8sStore.activeClusterId !== null || route.path === '/settings'">
                  <EmptyWorkspaceView
                    v-if="k8sStore.activeClusterId !== null && tabsStore.tabs.length === 0"
                    @open-picker="handleOpenEmptyPicker"
                  />
                  <OfflineClusterView
                    v-else-if="
                      activeCluster &&
                      activeCluster.status !== 'healthy' &&
                      route.path !== '/settings'
                    "
                  />
                  <RouterView v-else v-slot="{ Component, route: currentRoute }">
                    <KeepAlive
                      :include="[...editCacheNames, ...viewCacheNames]"
                      :max="Math.max(1, tabsStore.tabs.length)"
                    >
                      <component
                        :is="resolveViewComponent(Component, currentRoute)"
                        :key="resolveViewKey(currentRoute.path, currentRoute.fullPath)"
                        v-bind="currentRoute.params"
                      />
                    </KeepAlive>
                  </RouterView>
                </template>
                <WelcomeView v-else />
              </div>
            </main>
            <TerminalPanel />
          </div>
        </div>

        <!-- Footer -->
        <AppFooter />

        <!-- Empty Workspace Resource Picker Popover -->
        <AppTabResourcePicker ref="emptyPickerRef" @select-resource="handleEmptyResourceSelected" />
      </div>
    </Transition>
  </div>
</template>
