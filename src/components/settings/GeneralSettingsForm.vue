<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { useProfileStore } from '@/stores/profileStore'
import { useSettingsStore } from '@/stores/settingsStore'
import { useNotificationStore } from '@/stores/notificationStore'
import { useTableFilterStore } from '@/stores/tableFilterStore'
import { appSettingsService } from '@/services/appSettingsService'
import { CONFIG_KEYS, isGeneralConfigKey, type Configuration } from '@/types/settings'
import { AlertTriangle, Folder, Lock, RefreshCw, RotateCcw } from '@lucide/vue'
import Button from 'primevue/button'
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import Tag from 'primevue/tag'
import ToggleSwitch from 'primevue/toggleswitch'
import { useConfirm } from 'primevue/useconfirm'
import { useToast } from 'primevue/usetoast'
import { computed, ref } from 'vue'

const profileStore = useProfileStore()
const k8sStore = useKubernetesStore()
const settingsStore = useSettingsStore()
const notificationStore = useNotificationStore()
const tableFilterStore = useTableFilterStore()
const toast = useToast()
const confirm = useConfirm()

const isReloading = ref(false)

const generalSettings = computed(() =>
  settingsStore.settings.filter((config) => isGeneralConfigKey(config.key))
)

const logRetentionOptions = [
  { label: '5 files', value: 5 },
  { label: '10 files', value: 10 },
  { label: '20 files', value: 20 },
  { label: '50 files', value: 50 },
  { label: 'Unlimited (Keep all)', value: 0 }
]

const kubeconfigDisplayPath = computed(() => {
  if (profileStore.profile?.kubeconfigPaths && profileStore.profile.kubeconfigPaths.length > 0) {
    return profileStore.profile.kubeconfigPaths.join(', ')
  }
  return 'Auto-detected'
})

const isConfigModified = (config: Configuration): boolean => {
  if (config.value === undefined || config.value === null) return false
  if (config.defaultValue === undefined || config.defaultValue === null) return false
  return JSON.stringify(config.value) !== JSON.stringify(config.defaultValue)
}

const handleReset = async (config: Configuration) => {
  if (!config.enable) return
  const defaultVal = Array.isArray(config.defaultValue)
    ? [...config.defaultValue]
    : (config.defaultValue ?? null)
  await handleValueChange(config, defaultVal)
}

const handleReloadKubeconfig = async () => {
  if (isReloading.value) return
  isReloading.value = true
  try {
    await Promise.all([profileStore.fetchProfile(), k8sStore.loadInitialData()])
    toast.add({
      severity: 'success',
      summary: 'Kubeconfig Reloaded',
      detail: 'Kubeconfig and cluster contexts reloaded.',
      life: 3000
    })
  } catch (error) {
    console.error('Failed to reload kubeconfig:', error)
    toast.add({
      severity: 'error',
      summary: 'Error',
      detail: 'Failed to reload kubeconfig.',
      life: 3000
    })
  } finally {
    isReloading.value = false
  }
}

const handleValueChange = async (config: Configuration, newValue: unknown) => {
  if (!config.enable) return
  const isResetting =
    newValue === null || JSON.stringify(newValue) === JSON.stringify(config.defaultValue)
  if (config.key === CONFIG_KEYS.LAUNCH_ON_STARTUP) {
    await settingsStore.setLaunchOnStartup(Boolean(newValue))
  } else {
    await settingsStore.updateConfigValue(config.key, newValue)
  }
  toast.add({
    severity: 'success',
    summary: isResetting ? 'Settings Reset' : 'Settings Saved',
    detail: isResetting ? `${config.name} reset to default.` : `${config.name} updated.`,
    life: 2000
  })
}

const handleArrayChange = async (config: Configuration, rawText: string) => {
  if (!config.enable) return
  const items = rawText
    .split(',')
    .map((s) => s.trim())
    .filter(Boolean)
  await settingsStore.updateConfigValue(config.key, items)
  toast.add({
    severity: 'success',
    summary: 'Settings Saved',
    detail: `${config.name} updated.`,
    life: 2000
  })
}

const formatArrayValue = (val: unknown): string => {
  if (Array.isArray(val)) {
    return val.join(', ')
  }
  return typeof val === 'string' ? val : ''
}

const handleResetCache = () => {
  confirm.require({
    header: 'Reset Application Cache',
    message:
      'This will clear all in-memory table filters, reset notification history, and refresh active cluster resources. Are you sure?',
    icon: 'pi pi-exclamation-triangle',
    rejectProps: {
      label: 'Cancel',
      severity: 'secondary',
      variant: 'text',
      size: 'small'
    },
    acceptProps: {
      label: 'Reset Cache',
      severity: 'danger',
      size: 'small'
    },
    accept: async () => {
      notificationStore.clearAll()
      tableFilterStore.resetAllSelections()
      await k8sStore.loadInitialData()
      toast.add({
        severity: 'success',
        summary: 'Cache Cleared',
        detail: 'Local application cache and filters have been reset.',
        life: 2500
      })
    }
  })
}

const handleDeleteCachedContexts = () => {
  confirm.require({
    header: 'Delete Custom Contexts & Customizations',
    message:
      'This will remove all custom added kubeconfig file paths and cluster visual customizations (aliases, tags, colors). Auto-discovered clusters remain intact. Proceed?',
    icon: 'pi pi-exclamation-triangle',
    rejectProps: {
      label: 'Cancel',
      severity: 'secondary',
      variant: 'text',
      size: 'small'
    },
    acceptProps: {
      label: 'Delete Contexts',
      severity: 'danger',
      size: 'small'
    },
    accept: async () => {
      await settingsStore.updateConfigValue(CONFIG_KEYS.CUSTOM_KUBECONFIG_PATHS, [])
      await settingsStore.updateConfigValue(CONFIG_KEYS.CLUSTER_CUSTOMIZATIONS, {})
      await profileStore.fetchProfile()
      await k8sStore.loadInitialData()
      toast.add({
        severity: 'success',
        summary: 'Contexts Cleared',
        detail: 'Custom kubeconfig paths and cluster customizations have been removed.',
        life: 2500
      })
    }
  })
}

const handleResetAllSettings = () => {
  confirm.require({
    header: 'Reset All Orbit Settings',
    message:
      'Are you sure you want to restore all Orbit application settings to factory defaults? All preferences, appearance tokens, and configs will be reset.',
    icon: 'pi pi-exclamation-triangle',
    rejectProps: {
      label: 'Cancel',
      severity: 'secondary',
      variant: 'text',
      size: 'small'
    },
    acceptProps: {
      label: 'Reset Everything',
      severity: 'danger',
      size: 'small'
    },
    accept: async () => {
      const resetConfigs = settingsStore.settings.map((cfg) => ({
        ...cfg,
        value: cfg.defaultValue ?? null
      }))
      await appSettingsService.updateAppSettings(resetConfigs)
      await settingsStore.fetchSettings()
      toast.add({
        severity: 'success',
        summary: 'Settings Restored',
        detail: 'All application settings have been restored to defaults.',
        life: 2500
      })
    }
  })
}
</script>

<template>
  <div data-component="general-settings-form" class="flex flex-col gap-10">
    <!-- Header info -->
    <div>
      <h3 class="text-base font-semibold text-primary">General Settings</h3>
      <p class="text-xs text-muted-color mt-1">
        Configure startup, updates, logs, and core application behaviors.
      </p>
    </div>

    <!-- Dynamic Configuration List -->
    <div
      v-for="config in generalSettings"
      :key="config.key"
      class="flex flex-col md:flex-row gap-6 lg:gap-10"
      :class="{ 'opacity-60': !config.enable }"
    >
      <!-- Section Label & Description -->
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <div class="flex items-center gap-2">
          <h4 class="text-xs font-semibold text-primary uppercase tracking-wider">
            {{ config.name }}
          </h4>
          <Tag v-if="!config.enable" severity="secondary" value="Coming soon" />
          <span
            v-if="config.isConfidential"
            v-tooltip.top="'Confidential / Encrypted'"
            class="inline-flex items-center text-muted-color"
          >
            <Lock class="w-3 h-3" />
          </span>
        </div>
        <p class="text-[11px] text-muted-color">{{ config.description }}</p>
      </div>

      <!-- Control Field -->
      <div class="w-full md:w-2/3 xl:w-3/4 flex flex-col gap-3">
        <!-- Boolean Type -->
        <div
          v-if="config.datatype === 'boolean'"
          class="flex items-center justify-between gap-4 max-w-lg"
        >
          <span class="text-xs text-primary font-medium select-none">{{ config.name }}</span>
          <div class="flex items-center gap-2">
            <Button
              v-if="isConfigModified(config)"
              v-tooltip.top="'Reset to default'"
              severity="secondary"
              variant="text"
              rounded
              class="shrink-0"
              :disabled="!config.enable || settingsStore.isSaving"
              @click="handleReset(config)"
            >
              <RotateCcw class="w-3.5 h-3.5" />
            </Button>
            <ToggleSwitch
              :modelValue="Boolean(config.value ?? config.defaultValue ?? false)"
              :disabled="!config.enable || settingsStore.isSaving"
              @update:modelValue="(val: boolean) => handleValueChange(config, val)"
            />
          </div>
        </div>

        <!-- Number Type (Specialized for Log Retention or Generic InputNumber) -->
        <div v-else-if="config.datatype === 'number'" class="flex items-center gap-2 max-w-xs">
          <!-- Preset options selector for log retention limit -->
          <div class="flex-1">
            <Select
              v-if="config.key === CONFIG_KEYS.MAX_LOG_FILES"
              :modelValue="Number(config.value ?? config.defaultValue ?? 10)"
              :options="logRetentionOptions"
              optionLabel="label"
              optionValue="value"
              :disabled="!config.enable || settingsStore.isSaving"
              class="text-xs w-full"
              @update:modelValue="(val: number) => handleValueChange(config, val)"
            />
            <InputNumber
              v-else
              :modelValue="Number(config.value ?? config.defaultValue ?? 0)"
              :disabled="!config.enable || settingsStore.isSaving"
              class="text-xs w-full"
              @update:modelValue="(val: number | null) => handleValueChange(config, val ?? 0)"
            />
          </div>
          <Button
            v-if="isConfigModified(config)"
            v-tooltip.top="'Reset to default'"
            severity="secondary"
            variant="text"
            rounded
            class="shrink-0"
            :disabled="!config.enable || settingsStore.isSaving"
            @click="handleReset(config)"
          >
            <RotateCcw class="w-3.5 h-3.5" />
          </Button>
        </div>

        <!-- Multi-value String Array (e.g. 0..* or 1..*) -->
        <div
          v-else-if="
            config.datatype === 'string' &&
            (config.cardinality === '0..*' || config.cardinality === '1..*')
          "
          class="flex flex-col gap-1.5 max-w-lg"
        >
          <!-- Specialized kubeconfig path display with cluster context reload action -->
          <div v-if="config.key === CONFIG_KEYS.CUSTOM_KUBECONFIG_PATHS" class="flex gap-2">
            <div class="relative flex-1">
              <Folder class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-color" />
              <InputText
                :modelValue="kubeconfigDisplayPath"
                readonly
                disabled
                class="pl-9 pr-4 w-full text-xs"
              />
            </div>
            <Button
              v-if="isConfigModified(config)"
              v-tooltip.top="'Reset to default'"
              severity="secondary"
              variant="text"
              rounded
              class="shrink-0 self-center"
              :disabled="!config.enable || settingsStore.isSaving"
              @click="handleReset(config)"
            >
              <RotateCcw class="w-3.5 h-3.5" />
            </Button>
            <Button
              severity="secondary"
              variant="outlined"
              :loading="isReloading"
              :disabled="isReloading"
              class="text-xs font-semibold flex items-center gap-1.5"
              @click="handleReloadKubeconfig"
            >
              <RefreshCw class="w-3.5 h-3.5" :class="{ 'animate-spin': isReloading }" />
              <span>Reload</span>
            </Button>
          </div>
          <div v-else class="flex items-center gap-2">
            <InputText
              :modelValue="formatArrayValue(config.value ?? config.defaultValue)"
              :disabled="!config.enable || settingsStore.isSaving"
              class="text-xs w-full flex-1"
              placeholder="Comma-separated values"
              @change="
                (e: Event) => handleArrayChange(config, (e.target as HTMLInputElement).value)
              "
            />
            <Button
              v-if="isConfigModified(config)"
              v-tooltip.top="'Reset to default'"
              severity="secondary"
              variant="text"
              rounded
              class="shrink-0"
              :disabled="!config.enable || settingsStore.isSaving"
              @click="handleReset(config)"
            >
              <RotateCcw class="w-3.5 h-3.5" />
            </Button>
          </div>
        </div>

        <!-- Single String Type -->
        <div v-else class="flex items-center gap-2 max-w-lg">
          <InputText
            :modelValue="String(config.value ?? config.defaultValue ?? '')"
            :disabled="!config.enable || settingsStore.isSaving"
            :type="config.isConfidential ? 'password' : 'text'"
            class="text-xs w-full flex-1"
            @change="(e: Event) => handleValueChange(config, (e.target as HTMLInputElement).value)"
          />
          <Button
            v-if="isConfigModified(config)"
            v-tooltip.top="'Reset to default'"
            severity="secondary"
            variant="text"
            rounded
            class="shrink-0"
            :disabled="!config.enable || settingsStore.isSaving"
            @click="handleReset(config)"
          >
            <RotateCcw class="w-3.5 h-3.5" />
          </Button>
        </div>
      </div>
    </div>

    <!-- Danger Zone Section -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10 pt-2">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <div class="flex items-center gap-1.5 text-danger">
          <AlertTriangle class="w-3.5 h-3.5" />
          <h4 class="text-xs font-semibold uppercase tracking-wider">Danger Zone</h4>
        </div>
        <p class="text-[11px] text-muted-color">Irreversible actions and configuration resets.</p>
      </div>
      <div class="w-full md:w-2/3 xl:w-3/4 flex flex-col gap-3">
        <div
          class="bg-(--danger-soft) p-4 rounded-[var(--ui-radius-card)] flex flex-col gap-3 max-w-lg"
        >
          <span class="text-xs text-muted-color">
            These operations immediately wipe local caches and restore application defaults.
          </span>
          <div class="flex flex-wrap gap-2">
            <Button
              severity="danger"
              variant="outlined"
              class="text-xs font-medium"
              @click="handleResetCache"
            >
              Reset Cache
            </Button>
            <Button
              severity="danger"
              variant="outlined"
              class="text-xs font-medium"
              @click="handleDeleteCachedContexts"
            >
              Delete Cached Contexts
            </Button>
            <Button
              severity="danger"
              variant="outlined"
              class="text-xs font-medium"
              @click="handleResetAllSettings"
            >
              Reset All Settings
            </Button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
