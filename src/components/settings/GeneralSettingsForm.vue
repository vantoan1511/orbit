<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { useProfileStore } from '@/stores/profileStore'
import { useSettingsStore } from '@/stores/settingsStore'
import type { Configuration } from '@/types/settings'
import { AlertTriangle, Folder, Lock, RefreshCw } from '@lucide/vue'
import Button from 'primevue/button'
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import Tag from 'primevue/tag'
import ToggleSwitch from 'primevue/toggleswitch'
import { useToast } from 'primevue/usetoast'
import { computed, ref } from 'vue'

const profileStore = useProfileStore()
const k8sStore = useKubernetesStore()
const settingsStore = useSettingsStore()
const toast = useToast()

const isReloading = ref(false)

const logRetentionOptions = [
  { label: '5 files', value: 5 },
  { label: '10 files (Default)', value: 10 },
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
  await settingsStore.updateConfigValue(config.key, newValue)
  toast.add({
    severity: 'success',
    summary: 'Settings Saved',
    detail: `${config.name} updated.`,
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
</script>

<template>
  <div class="flex flex-col gap-10">
    <!-- Header info -->
    <div>
      <h3 class="text-base font-semibold text-primary">General Settings</h3>
      <p class="text-xs text-muted-color mt-1">
        Configure startup, updates, logs, and core application behaviors.
      </p>
    </div>

    <!-- Dynamic Configuration List -->
    <div
      v-for="config in settingsStore.settings"
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
          <ToggleSwitch
            :modelValue="Boolean(config.value ?? config.defaultValue ?? false)"
            :disabled="!config.enable || settingsStore.isSaving"
            @update:modelValue="(val: boolean) => handleValueChange(config, val)"
          />
        </div>

        <!-- Number Type (Specialized for Log Retention or Generic InputNumber) -->
        <div v-else-if="config.datatype === 'number'" class="flex flex-col gap-1.5 max-w-xs">
          <Select
            v-if="config.key === 'maxLogFiles'"
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

        <!-- Multi-value String Array (e.g. 0..* or 1..*) -->
        <div
          v-else-if="
            config.datatype === 'string' &&
            (config.cardinality === '0..*' || config.cardinality === '1..*')
          "
          class="flex flex-col gap-1.5 max-w-lg"
        >
          <div v-if="config.key === 'customKubeconfigPaths'" class="flex gap-2">
            <div class="relative flex-1">
              <Folder class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-color" />
              <InputText
                :modelValue="kubeconfigDisplayPath"
                readonly
                disabled
                class="pl-9 pr-4 py-1.5 w-full text-xs"
              />
            </div>
            <Button
              size="small"
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
          <div v-else class="flex flex-col gap-1">
            <InputText
              :modelValue="formatArrayValue(config.value ?? config.defaultValue)"
              :disabled="!config.enable || settingsStore.isSaving"
              class="text-xs w-full"
              placeholder="Comma-separated values"
              @change="
                (e: Event) => handleArrayChange(config, (e.target as HTMLInputElement).value)
              "
            />
          </div>
        </div>

        <!-- Single String Type -->
        <div v-else class="flex flex-col gap-1.5 max-w-lg">
          <InputText
            :modelValue="String(config.value ?? config.defaultValue ?? '')"
            :disabled="!config.enable || settingsStore.isSaving"
            :type="config.isConfidential ? 'password' : 'text'"
            class="text-xs w-full"
            @change="(e: Event) => handleValueChange(config, (e.target as HTMLInputElement).value)"
          />
        </div>
      </div>
    </div>

    <!-- Danger Zone Section -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10 pt-2 opacity-60">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <div class="flex items-center gap-1.5 text-muted-color">
          <AlertTriangle class="w-3.5 h-3.5" />
          <h4 class="text-xs font-semibold uppercase tracking-wider">Danger Zone</h4>
          <Tag severity="secondary" value="Coming soon" />
        </div>
        <p class="text-[11px] text-muted-color">Irreversible actions and configuration resets.</p>
      </div>
      <div class="w-full md:w-2/3 xl:w-3/4 flex flex-col gap-3">
        <div class="bg-rose-500/5 p-4 rounded-lg flex flex-col gap-3 max-w-lg">
          <span class="text-xs text-muted-color">
            These operations immediately wipe local caches and restore application defaults.
          </span>
          <div class="flex flex-wrap gap-2">
            <Button
              size="small"
              severity="danger"
              variant="outlined"
              disabled
              class="text-xs font-medium"
            >
              Reset Cache
            </Button>
            <Button
              size="small"
              severity="danger"
              variant="outlined"
              disabled
              class="text-xs font-medium"
            >
              Delete Cached Contexts
            </Button>
            <Button
              size="small"
              severity="danger"
              variant="outlined"
              disabled
              class="text-xs font-medium"
            >
              Reset All Settings
            </Button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
