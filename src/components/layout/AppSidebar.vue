<script setup lang="ts">
import { useSidebarState } from '@/composables/useSidebarState'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppSidebarActivityBar from './sidebar/AppSidebarActivityBar.vue'
import AppSidebarClusters from './sidebar/AppSidebarClusters.vue'
import AppSidebarLogsMenu from './sidebar/AppSidebarLogsMenu.vue'
import AppSidebarNavMenu from './sidebar/AppSidebarNavMenu.vue'
import AppSidebarPanel from './sidebar/AppSidebarPanel.vue'

const k8sStore = useKubernetesStore()
const hasActiveCluster = computed(() => k8sStore.activeClusterId !== null)

const route = useRoute()
const router = useRouter()

const {
  isCollapsed,
  activeTab,
  toggleCategory,
  handleCollapse,
  handleClusterSwitched,
  syncWithRoute,
  loadStoredState
} = useSidebarState({
  storageKey: 'orbit_sidebar_collapsed',
  currentPath: computed(() => route.path),
  hasActiveCluster,
  onNavigate: (path) => void router.push(path)
})

watch(
  () => [route.path, route.name, route.params.kind],
  ([newPath]) => {
    syncWithRoute(
      newPath as string,
      route.name as string | undefined,
      route.params.kind as string | undefined
    )
  },
  { immediate: true }
)

watch(
  () => k8sStore.activeClusterId,
  (clusterId) => {
    if (!clusterId && activeTab.value !== 'clusters') {
      if (route.path !== '/settings') {
        activeTab.value = 'clusters'
      } else {
        activeTab.value = null
      }
    }
  }
)

onMounted(() => {
  void loadStoredState()
})

const onClusterSwitched = () => {
  handleClusterSwitched(
    route.path,
    route.name as string | undefined,
    route.params.kind as string | undefined
  )
}
</script>

<template>
  <aside class="flex h-full text-primary select-none">
    <!-- Activity Bar (Far Left Strip) -->
    <AppSidebarActivityBar
      :active-tab="isCollapsed ? null : activeTab"
      :has-active-cluster="hasActiveCluster"
      @toggle-category="toggleCategory"
    />

    <!-- Contextual Sidebar Panel -->
    <AppSidebarPanel :active-tab="isCollapsed ? null : activeTab" @collapse="handleCollapse">
      <AppSidebarLogsMenu v-if="activeTab === 'logs'" />
      <AppSidebarNavMenu v-else-if="activeTab !== 'clusters'" :active-tab="activeTab" />
      <AppSidebarClusters v-else @cluster-switched="onClusterSwitched" />
    </AppSidebarPanel>
  </aside>
</template>
