import { updaterService } from '@/services/updaterService'
import type { UpdateManifest } from '@/types/events'
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useUpdaterStore = defineStore('updater', () => {
  const isChecking = ref(false)
  const hasUpdate = ref(false)
  const manifest = ref<UpdateManifest | null>(null)

  const isDownloading = ref(false)
  const downloadProgress = ref(0)
  const updateReady = ref<string | null>(null) // 'app'

  function checkForUpdates() {
    isChecking.value = true
    updaterService.checkForUpdates()
  }

  function applyUpdate() {
    if (!manifest.value?.url) return
    isDownloading.value = true
    downloadProgress.value = 0
    updaterService.applyUpdate(manifest.value.url)
  }

  function initListeners() {
    updaterService.onUpdateCheckFinished((data) => {
      isChecking.value = false
      hasUpdate.value = data.has_update
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
    hasUpdate,
    manifest,
    isDownloading,
    downloadProgress,
    updateReady,
    checkForUpdates,
    applyUpdate,
    initListeners
  }
})
