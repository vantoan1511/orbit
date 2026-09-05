<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppSidebarActivityBar from './sidebar/AppSidebarActivityBar.vue'
import AppSidebarClusters from './sidebar/AppSidebarClusters.vue'
import AppSidebarLogsMenu from './sidebar/AppSidebarLogsMenu.vue'
import AppSidebarNavMenu from './sidebar/AppSidebarNavMenu.vue'
import AppSidebarPanel from './sidebar/AppSidebarPanel.vue'
import { type CategoryId, type SidebarCategory } from './sidebar/navigation'

const k8sStore = useKubernetesStore()
const hasActiveCluster = computed(() => k8sStore.activeClusterId !== null)

const activeTab = ref<CategoryId | null>('clusters')
const route = useRoute()
const router = useRouter()

const getCategoryForRoute = (path: string): CategoryId | null => {
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

watch(
  () => route.path,
  (newPath) => {
    const category = getCategoryForRoute(newPath)
    if (category && activeTab.value !== 'clusters' && hasActiveCluster.value) {
      activeTab.value = category
    }
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

const toggleCategory = (cat: SidebarCategory) => {
  if (cat.requiresCluster && !hasActiveCluster.value) {
    return
  }
  if (cat.id === activeTab.value) {
    activeTab.value = null
    return
  }

  activeTab.value = cat.id
  if (cat.defaultPath) {
    router.push(cat.defaultPath)
  }
}

const handleClusterSwitched = () => {
  activeTab.value = getCategoryForRoute(route.path) || 'core'
}
</script>

<template>
  <aside class="flex h-full text-primary select-none">
    <!-- Activity Bar (Far Left Strip) -->
    <AppSidebarActivityBar
      :active-tab="activeTab"
      :has-active-cluster="hasActiveCluster"
      @toggle-category="toggleCategory"
    />

    <!-- Contextual Sidebar Panel -->
    <AppSidebarPanel :active-tab="activeTab" @collapse="activeTab = null">
      <AppSidebarLogsMenu v-if="activeTab === 'logs'" />
      <AppSidebarNavMenu v-else-if="activeTab !== 'clusters'" :active-tab="activeTab" />
      <AppSidebarClusters v-else @cluster-switched="handleClusterSwitched" />
    </AppSidebarPanel>
  </aside>
</template>
