<script setup lang="ts">
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import type { ClusterInfo, PodInfo } from '@/types/kubernetes'
import ConfirmDialog from 'primevue/confirmdialog'
import Toast from 'primevue/toast'
import { onMounted, onUnmounted } from 'vue'
import AppLayout from './components/layout/AppLayout.vue'

const k8sStore = useKubernetesStore()

const handleEngineConnected = (payload: { status: 'ready' | 'error'; message: string }) => {
  if (payload.status === 'ready') {
    k8sStore.setEngineReady(true)
    k8sStore.loadInitialData()
  } else {
    k8sStore.setEngineReady(false)
  }
}

const handleNamespacesUpdated = (payload: { namespaces: string[] }) => {
  k8sStore.setNamespaces(payload.namespaces)
}

const handlePodsUpdated = (payload: { pods: PodInfo[] }) => {
  k8sStore.setPods(payload.pods)
}

const handleClustersUpdated = (payload: { clusters: ClusterInfo[] }) => {
  k8sStore.setClusters(payload.clusters)
}

const handleActiveClusterChanged = (payload: { active_cluster_id: string | null }) => {
  k8sStore.setActiveClusterId(payload.active_cluster_id)
}

onMounted(() => {
  // Initialize dark mode by default
  document.documentElement.classList.add('my-app-dark')
  document.documentElement.setAttribute('data-theme', 'dark')

  events.on(OrbitEvents.EngineConnected, handleEngineConnected)
  events.on(OrbitEvents.NamespacesUpdated, handleNamespacesUpdated)
  events.on(OrbitEvents.PodsUpdated, handlePodsUpdated)
  events.on(OrbitEvents.ClustersUpdated, handleClustersUpdated)
  events.on(OrbitEvents.ActiveClusterChanged, handleActiveClusterChanged)
})

onUnmounted(() => {
  events.off(OrbitEvents.EngineConnected, handleEngineConnected)
  events.off(OrbitEvents.NamespacesUpdated, handleNamespacesUpdated)
  events.off(OrbitEvents.PodsUpdated, handlePodsUpdated)
  events.off(OrbitEvents.ClustersUpdated, handleClustersUpdated)
  events.off(OrbitEvents.ActiveClusterChanged, handleActiveClusterChanged)
})
</script>

<template>
  <AppLayout />
  <Toast />
  <ConfirmDialog />
</template>

<style>
/* Global scrollbar styling matching modern dark theme */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--border-strong);
}
</style>
