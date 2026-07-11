import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { PodInfo, ClusterInfo } from '@/types/kubernetes'
import { kubernetesService } from '@/services/kubernetesService'

export const useKubernetesStore = defineStore('kubernetes', () => {
  const isEngineReady = ref(false)
  const pods = ref<PodInfo[]>([])
  const namespaces = ref<string[]>(['All Namespaces'])
  const clusters = ref<ClusterInfo[]>([])
  const activeClusterId = ref<string | null>(null)

  function setEngineReady(ready: boolean) {
    isEngineReady.value = ready
  }

  function setPods(newPods: PodInfo[]) {
    pods.value = newPods
  }

  function setNamespaces(newNamespaces: string[]) {
    namespaces.value = ['All Namespaces', ...newNamespaces]
  }

  function setClusters(newClusters: ClusterInfo[]) {
    clusters.value = newClusters
  }

  function setActiveClusterId(id: string | null) {
    activeClusterId.value = id
  }

  async function loadInitialData() {
    if (isEngineReady.value) {
      await kubernetesService.getClusters()
      await kubernetesService.getNamespaces()
      await kubernetesService.getPods()
    }
  }

  return {
    isEngineReady,
    pods,
    namespaces,
    clusters,
    activeClusterId,
    setEngineReady,
    setPods,
    setNamespaces,
    setClusters,
    setActiveClusterId,
    loadInitialData
  }
})
