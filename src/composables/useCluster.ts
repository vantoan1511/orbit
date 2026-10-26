import { computed, ref } from 'vue'
import { getActivePinia } from 'pinia'
import { kubernetesService } from '../services/kubernetesService.ts'
import { useKubernetesStore } from '../stores/kubernetesStore.ts'

const isAddClusterModalVisible = ref(false)

export function openAddClusterModal(): void {
  isAddClusterModalVisible.value = true
}

export function closeAddClusterModal(): void {
  isAddClusterModalVisible.value = false
}

export { isAddClusterModalVisible }

export function useCluster() {
  const isRefreshing = ref(false)

  const handleAddCluster = (): void => {
    openAddClusterModal()
  }

  const getStore = () => (getActivePinia() ? useKubernetesStore() : null)

  const activeCluster = computed(() => {
    const store = getStore()
    return store?.clusters.find((c) => c.id === store.activeClusterId) ?? null
  })

  const lastUpdatedAt = computed(() => {
    const store = getStore()
    return store?.lastUpdatedAt ?? null
  })

  const refreshCluster = async (): Promise<void> => {
    if (isRefreshing.value) return
    isRefreshing.value = true
    try {
      const store = getStore()
      if (store) {
        await store.loadInitialData()
      }
    } finally {
      isRefreshing.value = false
    }
  }

  const reconnectActiveCluster = async (): Promise<void> => {
    const store = getStore()
    const clusterId = store?.activeClusterId
    if (clusterId) {
      await kubernetesService.switchCluster(clusterId)
    }
  }

  return {
    activeCluster,
    isRefreshing,
    isAddClusterModalVisible,
    openAddClusterModal,
    closeAddClusterModal,
    handleAddCluster,
    refreshCluster,
    reconnectActiveCluster,
    lastUpdatedAt
  }
}
