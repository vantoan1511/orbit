import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { PodInfo } from '@/types/kubernetes'
import { kubernetesService } from '@/services/kubernetesService'

export const useKubernetesStore = defineStore('kubernetes', () => {
  const isEngineReady = ref(false)
  const pods = ref<PodInfo[]>([])
  const namespaces = ref<string[]>(['All Namespaces'])

  function setEngineReady(ready: boolean) {
    isEngineReady.value = ready
  }

  function setPods(newPods: PodInfo[]) {
    pods.value = newPods
  }

  function setNamespaces(newNamespaces: string[]) {
    namespaces.value = ['All Namespaces', ...newNamespaces]
  }

  async function loadInitialData() {
    if (isEngineReady.value) {
      await kubernetesService.getNamespaces()
      await kubernetesService.getPods()
    }
  }

  return {
    isEngineReady,
    pods,
    namespaces,
    setEngineReady,
    setPods,
    setNamespaces,
    loadInitialData
  }
})
