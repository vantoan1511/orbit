<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import AppLayout from './components/layout/AppLayout.vue'
import Toast from 'primevue/toast'
import ConfirmDialog from 'primevue/confirmdialog'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { events } from '@/services/nativeService'
import type { PodInfo } from '@/types/kubernetes'

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

onMounted(() => {
  // Initialize dark mode by default
  document.documentElement.classList.add('my-app-dark')
  document.documentElement.setAttribute('data-theme', 'dark')

  events.on('engineConnected', handleEngineConnected)
  events.on('namespacesUpdated', handleNamespacesUpdated)
  events.on('podsUpdated', handlePodsUpdated)
})

onUnmounted(() => {
  events.off('engineConnected', handleEngineConnected)
  events.off('namespacesUpdated', handleNamespacesUpdated)
  events.off('podsUpdated', handlePodsUpdated)
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
