import { defineStore } from 'pinia'
import { ref } from 'vue'
import { updaterService } from '@/services/updaterService'

export const useUpdaterStore = defineStore('updater', () => {
  const isChecking = ref(false)
  const hasResourcesUpdate = ref(false)
  const hasEngineUpdate = ref(false)
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const manifest = ref<any>(null)

  const isDownloading = ref(false)
  const downloadProgress = ref(0)
  const updateReady = ref<string | null>(null) // 'resources' | 'engine'

  function checkForUpdates() {
    isChecking.value = true
    updaterService.checkForUpdates()
  }

  function applyResourceUpdate() {
    if (!manifest.value?.resources?.url) return
    isDownloading.value = true
    downloadProgress.value = 0
    updaterService.applyResourceUpdate(manifest.value.resources.url)
  }

  function triggerEngineUpdate() {
    if (!manifest.value?.engine?.url) return
    isDownloading.value = true
    downloadProgress.value = 0
    updaterService.triggerEngineUpdate(manifest.value.engine.url)
  }

  function initListeners() {
    updaterService.onUpdateCheckFinished((data) => {
      isChecking.value = false
      hasResourcesUpdate.value = data.has_resources_update
      hasEngineUpdate.value = data.has_engine_update
      manifest.value = data.manifest
    })

    updaterService.onUpdateDownloadProgress((data) => {
      isDownloading.value = true
      downloadProgress.value = data.progress_percentage
    })

    updaterService.onUpdateReady((data) => {
      isDownloading.value = false
      downloadProgress.value = 100
      updateReady.value = data.component
    })
  }

  return {
    isChecking,
    hasResourcesUpdate,
    hasEngineUpdate,
    manifest,
    isDownloading,
    downloadProgress,
    updateReady,
    checkForUpdates,
    applyResourceUpdate,
    triggerEngineUpdate,
    initListeners
  }
})
