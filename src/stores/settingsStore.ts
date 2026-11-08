import { appSettingsService } from '../services/appSettingsService.ts'
import { CONFIG_KEYS, type Configuration, type ConfigurationMap } from '../types/settings.ts'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

export function isConfigurationEqual(a: Configuration, b: Configuration): boolean {
  if (a === b) return true
  if (
    a.key !== b.key ||
    a.name !== b.name ||
    a.description !== b.description ||
    a.datatype !== b.datatype ||
    a.enable !== b.enable ||
    a.isConfidential !== b.isConfidential ||
    a.cardinality !== b.cardinality
  ) {
    return false
  }

  const aEffective = a.value ?? a.defaultValue
  const bEffective = b.value ?? b.defaultValue
  if (JSON.stringify(aEffective) !== JSON.stringify(bEffective)) {
    return false
  }

  return JSON.stringify(a.defaultValue) === JSON.stringify(b.defaultValue)
}

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
      if (settings.value.length === 0) {
        settings.value = data
      } else {
        const next = data.map((incoming) => {
          const existing = settingsMap.value.get(incoming.key)
          if (existing && isConfigurationEqual(existing, incoming)) {
            return existing
          }
          return incoming
        })

        const hasChanged =
          settings.value.length !== next.length ||
          settings.value.some((item, i) => item !== next[i])

        if (hasChanged) {
          settings.value = next
        }
      }
    } else if (data && typeof data === 'object') {
      // If a key-value map was received, update values in existing configuration list immutably
      let hasChanged = false
      const next = settings.value.map((item) => {
        if (Object.prototype.hasOwnProperty.call(data, item.key)) {
          const nextVal = data[item.key]
          if (JSON.stringify(item.value) !== JSON.stringify(nextVal)) {
            hasChanged = true
            return { ...item, value: nextVal }
          }
        }
        return item
      })

      if (hasChanged) {
        settings.value = next
      }
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
      if (target && JSON.stringify(target.value) !== JSON.stringify(value)) {
        const next = [...settings.value]
        next[index] = { ...target, value }
        settings.value = next
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

  async function setLaunchOnStartup(enabled: boolean) {
    const index = settings.value.findIndex((c) => c.key === CONFIG_KEYS.LAUNCH_ON_STARTUP)
    if (index !== -1) {
      const target = settings.value[index]
      if (target && target.value !== enabled) {
        const next = [...settings.value]
        next[index] = { ...target, value: enabled }
        settings.value = next
      }
    }
    isSaving.value = true
    try {
      await appSettingsService.setLaunchOnStartup(enabled)
    } catch (e) {
      console.error('Failed to update launchOnStartup:', e)
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
    updateSettings,
    setLaunchOnStartup
  }
})
