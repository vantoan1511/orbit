import { appSettingsService } from '@/services/appSettingsService'
import type { Configuration, ConfigurationMap } from '@/types/settings'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<Configuration[]>([])
  const isLoading = ref(false)
  const isSaving = ref(false)

  const settingsMap = computed(() => {
    const map = new Map<string, Configuration>()
    for (const item of settings.value) {
      map.set(item.key, item)
    }
    return map
  })

  function setSettings(data: Configuration[] | ConfigurationMap) {
    if (Array.isArray(data)) {
      settings.value = data
    } else if (data && typeof data === 'object') {
      // If a key-value map was received, update values in existing configuration list immutably
      settings.value = settings.value.map((item) => {
        if (Object.prototype.hasOwnProperty.call(data, item.key)) {
          return { ...item, value: data[item.key] }
        }
        return item
      })
    }
    isLoading.value = false
    isSaving.value = false
  }

  function getConfig(key: string): Configuration | undefined {
    return settingsMap.value.get(key)
  }

  function getConfigValue<T = unknown>(key: string, fallback?: T): T {
    const config = settingsMap.value.get(key)
    if (!config) return fallback as T
    return (config.value ?? config.defaultValue ?? fallback) as T
  }

  async function fetchSettings() {
    isLoading.value = true
    try {
      await appSettingsService.getAppSettings()
    } catch (e) {
      console.error('Failed to fetch app settings:', e)
    } finally {
      isLoading.value = false
    }
  }

  async function updateConfigValue(key: string, value: unknown) {
    const index = settings.value.findIndex((c) => c.key === key)
    if (index !== -1) {
      const target = settings.value[index]
      if (target) {
        settings.value[index] = { ...target, value }
      }
    }
    isSaving.value = true
    try {
      await appSettingsService.updateAppSettings(settings.value)
    } catch (e) {
      console.error(`Failed to update config ${key}:`, e)
    } finally {
      isSaving.value = false
    }
  }

  async function updateSettings(newSettings: Configuration[] | ConfigurationMap) {
    isSaving.value = true
    try {
      await appSettingsService.updateAppSettings(newSettings)
    } catch (e) {
      console.error('Failed to update app settings:', e)
    } finally {
      isSaving.value = false
    }
  }

  return {
    settings,
    settingsMap,
    isLoading,
    isSaving,
    setSettings,
    getConfig,
    getConfigValue,
    fetchSettings,
    updateConfigValue,
    updateSettings
  }
})
