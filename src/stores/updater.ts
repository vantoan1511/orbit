import { updaterService } from '../services/updaterService.ts'
import type { OrbitEventMap, UpdateManifest } from '../types/events.ts'
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useUpdaterStore = defineStore('updater', () => {
  const isChecking = ref(false)
  const hasUpdate = ref(false)
  const manifest = ref<UpdateManifest | null>(null)

  const isDownloading = ref(false)
  const downloadProgress = ref(0)
  const updateReady = ref<string | null>(null) // 'app'
  const installError = ref<OrbitEventMap['updateInstallFailed'] | null>(null)
  const showUpdateDialog = ref(false)
  let listenersInitialized = false

  function checkForUpdates() {
    isChecking.value = true
    updaterService.checkForUpdates()
  }

  function applyUpdate() {
    if (!manifest.value?.url || !manifest.value?.version) return
    installError.value = null
    isDownloading.value = true
    downloadProgress.value = 0
    updaterService.applyUpdate(manifest.value.url, manifest.value.version)
  }

  function initListeners() {
    if (listenersInitialized) return
    listenersInitialized = true

    updaterService.onUpdateCheckFinished((data) => {
      isChecking.value = false
      hasUpdate.value = data.has_update
      manifest.value = data.manifest
      if (data.has_update) {
        showUpdateDialog.value = true
      }
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

    updaterService.onUpdateInstallFailed((data) => {
      isDownloading.value = false
      downloadProgress.value = 0
      updateReady.value = null
      installError.value = data
    })
  }

  return {
    isChecking,
    hasUpdate,
    manifest,
    isDownloading,
    downloadProgress,
    updateReady,
    installError,
    showUpdateDialog,
    checkForUpdates,
    applyUpdate,
    initListeners
  }
})
