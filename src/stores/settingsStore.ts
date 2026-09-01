import { appSettingsService } from '@/services/appSettingsService'
import type { OrbitConfig } from '@/types/settings'
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<OrbitConfig>({
    customKubeconfigPaths: [],
    maxLogFiles: 10
  })
  const isLoading = ref(false)
  const isSaving = ref(false)

  function setSettings(data: OrbitConfig | Record<string, unknown>) {
    settings.value = {
      customKubeconfigPaths:
        (data as OrbitConfig).customKubeconfigPaths ??
        (data as Record<string, unknown>).custom_kubeconfig_paths ??
        [],
      maxLogFiles:
        (data as OrbitConfig).maxLogFiles ?? (data as Record<string, unknown>).max_log_files ?? 10
    } as OrbitConfig
    isLoading.value = false
    isSaving.value = false
  }

  async function fetchSettings() {
    isLoading.value = true
    try {
      await appSettingsService.getAppSettings()
    } catch (e) {
      console.error('Failed to fetch app settings:', e)
      isLoading.value = false
    }
  }

  async function updateSettings(newSettings: Partial<OrbitConfig>) {
    isSaving.value = true
    const updated: OrbitConfig = {
      customKubeconfigPaths:
        newSettings.customKubeconfigPaths ?? settings.value.customKubeconfigPaths ?? [],
      maxLogFiles: newSettings.maxLogFiles ?? settings.value.maxLogFiles ?? 10
    }
    try {
      await appSettingsService.updateAppSettings(updated)
    } catch (e) {
      console.error('Failed to update app settings:', e)
      isSaving.value = false
    }
  }

  return {
    settings,
    isLoading,
    isSaving,
    setSettings,
    fetchSettings,
    updateSettings
  }
})
