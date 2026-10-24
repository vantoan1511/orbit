import { computed, ref } from 'vue'
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

  const activeCluster = computed(() => {
    try {
      const k8sStore = useKubernetesStore()
      return k8sStore.clusters.find((c) => c.id === k8sStore.activeClusterId) || null
    } catch {
      return null
    }
  })

  const lastUpdatedAt = computed(() => {
    try {
      const k8sStore = useKubernetesStore()
      return k8sStore.lastUpdatedAt
    } catch {
      return null
    }
  })

  const refreshCluster = async (): Promise<void> => {
    if (isRefreshing.value) return
    isRefreshing.value = true
    try {
      const k8sStore = useKubernetesStore()
      await k8sStore.loadInitialData()
    } catch (error) {
      console.error('Failed to refresh cluster data:', error)
    } finally {
      isRefreshing.value = false
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
    lastUpdatedAt
  }
}
