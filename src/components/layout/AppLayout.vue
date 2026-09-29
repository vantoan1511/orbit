<script setup lang="ts">
import { useCluster } from '@/composables/useCluster.ts'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { useTabsStore } from '@/stores/tabsStore'
import OfflineClusterView from '@/views/OfflineClusterView.vue'
import WelcomeView from '@/views/WelcomeView.vue'
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { getTabMetadataForRoute } from '@/utils/tabIcons'
import AppFooter from './AppFooter.vue'
import AppHeader from './AppHeader.vue'
import AppLoadingScreen from './AppLoadingScreen.vue'
import AppSidebar from './AppSidebar.vue'
import AppTabBar from './tabs/AppTabBar.vue'
import AppTabResourcePicker from './tabs/AppTabResourcePicker.vue'
import EmptyWorkspaceView from './tabs/EmptyWorkspaceView.vue'

const k8sStore = useKubernetesStore()
const tabsStore = useTabsStore()
const { activeCluster } = useCluster()
const route = useRoute()
const router = useRouter()

const emptyPickerRef = ref<InstanceType<typeof AppTabResourcePicker> | null>(null)

// Synchronize current route with tabsStore
watch(
  () => [route.path, route.query],
  ([newPath, newQuery]) => {
    if (k8sStore.activeClusterId !== null && newPath !== '/welcome' && newPath !== '/settings') {
      const meta = getTabMetadataForRoute(newPath as string, newQuery as Record<string, string>)
      tabsStore.syncWithRoute(
        newPath as string,
        newQuery as Record<string, string>,
        meta.title,
        meta.iconName,
        meta.category
      )
    }
  },
  { immediate: true }
)

// Rehydrate tabs on active cluster change
watch(
  () => k8sStore.activeClusterId,
  (newClusterId) => {
    if (newClusterId) {
      void tabsStore.init(newClusterId)
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

// Global shortcut: Ctrl+W / Cmd+W to close active tab
const handleKeyDown = (e: KeyboardEvent) => {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'w') {
    const activeEl = document.activeElement as HTMLElement | null
    if (
      activeEl &&
      (activeEl.tagName === 'INPUT' ||
        activeEl.tagName === 'TEXTAREA' ||
        activeEl.classList.contains('monaco-editor'))
    ) {
      return
    }
    if (tabsStore.activeTabId) {
      e.preventDefault()
      const wasActive = tabsStore.activeTabId
      tabsStore.closeTab(wasActive)
      if (tabsStore.activeTab) {
        void router.push(tabsStore.activeTab.route)
      }
    }
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown)
})
</script>

<template>
  <Transition name="page" mode="out-in">
    <AppLoadingScreen v-if="k8sStore.isAppLoading" @complete="k8sStore.setAppLoading(false)" />
    <div v-else class="flex flex-col h-screen w-screen overflow-hidden text-primary font-sans">
      <div class="flex-1 flex overflow-hidden">
        <!-- Sidebar -->
        <AppSidebar />

        <!-- Main Content Area -->
        <div class="flex-1 flex flex-col h-full overflow-hidden">
          <AppHeader />

          <!-- Top Main Tabs -->
          <AppTabBar v-if="k8sStore.activeClusterId !== null" />

          <main class="flex-1 h-full overflow-y-auto relative">
            <div class="p-8">
              <template v-if="k8sStore.activeClusterId !== null || route.path === '/settings'">
                <EmptyWorkspaceView
                  v-if="tabsStore.tabs.length === 0 && route.path !== '/settings'"
                  @open-picker="handleOpenEmptyPicker"
                />
                <OfflineClusterView
                  v-else-if="
                    activeCluster &&
                    activeCluster.status !== 'healthy' &&
                    route.path !== '/settings'
                  "
                />
                <RouterView v-else v-slot="{ Component }">
                  <transition name="page" mode="out-in">
                    <component :is="Component" />
                  </transition>
                </RouterView>
              </template>
              <WelcomeView v-else />
            </div>
          </main>
        </div>
      </div>

      <!-- Footer -->
      <AppFooter />

      <!-- Empty Workspace Resource Picker Popover -->
      <AppTabResourcePicker ref="emptyPickerRef" @select-resource="handleEmptyResourceSelected" />
    </div>
  </Transition>
</template>
