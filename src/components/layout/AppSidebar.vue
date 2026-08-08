<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppSidebarActivityBar from './sidebar/AppSidebarActivityBar.vue'
import AppSidebarClusters from './sidebar/AppSidebarClusters.vue'
import AppSidebarNavMenu from './sidebar/AppSidebarNavMenu.vue'
import AppSidebarPanel from './sidebar/AppSidebarPanel.vue'
import { type CategoryId, type SidebarCategory } from './sidebar/navigation'

const activeTab = ref<CategoryId | null>('clusters')
const route = useRoute()
const router = useRouter()

const expandedKeys = ref<Record<string, boolean>>({
  core: true,
  workloads: true,
  network: true,
  storage: true,
  config: true,
  security: true,
  logs: true
})

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
    if (category && activeTab.value !== 'clusters') {
      activeTab.value = category
      expandedKeys.value = { ...expandedKeys.value, [category]: true }
    }
  },
  { immediate: true }
)

const toggleCategory = (cat: SidebarCategory) => {
  if (cat.id === activeTab.value) {
    activeTab.value = null
    return
  }

  activeTab.value = cat.id
  expandedKeys.value = { ...expandedKeys.value, [cat.id]: true }
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
    <AppSidebarActivityBar :active-tab="activeTab" @toggle-category="toggleCategory" />

    <!-- Contextual Sidebar Panel -->
    <AppSidebarPanel :active-tab="activeTab">
      <AppSidebarNavMenu
        v-if="activeTab !== 'clusters'"
        :active-tab="activeTab"
        v-model:expanded-keys="expandedKeys"
      />
      <AppSidebarClusters v-else @cluster-switched="handleClusterSwitched" />
    </AppSidebarPanel>
  </aside>
</template>
