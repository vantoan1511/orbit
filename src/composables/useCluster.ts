import { kubernetesService } from '@/services/kubernetesService'
import { os } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { useToast } from 'primevue'
import { computed, ref } from 'vue'

export function useCluster() {
  const k8sStore = useKubernetesStore()
  const toast = useToast()

  const isRefreshing = ref(false)

  const handleAddCluster = async () => {
    try {
      const selectedFiles = await os.showOpenDialog('Select Kubeconfig File')
      if (selectedFiles?.[0]) {
        await kubernetesService.addCluster(selectedFiles[0])
      }
    } catch (error) {
      console.error('Failed to add cluster:', error)
      toast.add({
        severity: 'error',
        summary: 'Error',
        detail: 'Failed to add cluster.',
        life: 3000
      })
    }
  }

  const activeCluster = computed(() => {
    return k8sStore.clusters.find((c) => c.id === k8sStore.activeClusterId) || null
  })

  const lastUpdatedAt = ref<Date | null>(null)

  const refreshCluster = async () => {
    if (isRefreshing.value) return
    isRefreshing.value = true
    try {
      await k8sStore.loadInitialData()
      lastUpdatedAt.value = new Date()
    } catch (error) {
      console.error('Failed to refresh cluster data:', error)
    } finally {
      isRefreshing.value = false
    }
  }

  return {
    activeCluster,
    isRefreshing,
    handleAddCluster,
    refreshCluster,
    lastUpdatedAt
  }
}
