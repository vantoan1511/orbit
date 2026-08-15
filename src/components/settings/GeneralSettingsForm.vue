<script setup lang="ts">
import { ref } from 'vue'
import Checkbox from 'primevue/checkbox'
import Select from 'primevue/select'
import InputText from 'primevue/inputtext'
import ToggleSwitch from 'primevue/toggleswitch'
import Button from 'primevue/button'
import { RefreshCw, Folder, AlertTriangle } from '@lucide/vue'
import { useToast } from 'primevue/usetoast'
import { useConfirm } from 'primevue/useconfirm'

const toast = useToast()
const confirm = useConfirm()

// Startup settings state
const launchOnStartup = ref(true)
const startMinimized = ref(false)

// Updates settings state
const autoCheckUpdates = ref(true)
const updateChannel = ref('Stable')
const channels = ref(['Stable', 'Beta', 'Nightly'])

// Kubeconfig settings state
const kubeconfigPath = ref('~/.kube/config')

// Telemetry settings state
const shareTelemetry = ref(false)

const handleReloadKubeconfig = () => {
  toast.add({
    severity: 'success',
    summary: 'Kubeconfig Reloaded',
    detail: 'Kubeconfig reloaded successfully!',
    life: 3000
  })
}

const handleResetCache = () => {
  confirm.require({
    message: 'Are you sure you want to reset the application cache?',
    header: 'Reset Cache',
    accept: () => {
      toast.add({
        severity: 'success',
        summary: 'Cache Cleared',
        detail: 'Application cache cleared.',
        life: 3000
      })
    }
  })
}

const handleResetAllSettings = () => {
  confirm.require({
    message: 'Are you sure you want to reset all settings to defaults?',
    header: 'Reset Settings',
    accept: () => {
      toast.add({
        severity: 'success',
        summary: 'Settings Reset',
        detail: 'Settings reset to defaults.',
        life: 3000
      })
    }
  })
}

const handleDeleteCachedContexts = () => {
  confirm.require({
    message: 'Are you sure you want to delete all cached context metadata?',
    header: 'Delete Contexts',
    accept: () => {
      toast.add({
        severity: 'success',
        summary: 'Contexts Deleted',
        detail: 'Cached contexts deleted.',
        life: 3000
      })
    }
  })
}
</script>

<template>
  <div class="flex flex-col gap-10">
    <!-- Header info -->
    <div>
      <h3 class="text-base font-semibold text-primary">General Settings</h3>
      <p class="text-xs text-muted-color mt-1">
        Configure startup, updates, and core application behaviors.
      </p>
    </div>

    <!-- Startup Section -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10 opacity-60">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <div class="flex items-center gap-2">
          <h4 class="text-xs font-semibold text-primary uppercase tracking-wider">Startup</h4>
          <span
            class="text-[10px] px-1.5 py-0.2 rounded bg-surface-200 dark:bg-surface-800 text-muted-color"
            >Coming soon</span
          >
        </div>
        <p class="text-[11px] text-muted-color">Configure how Orbit behaves when launched.</p>
      </div>
      <div class="w-full md:w-2/3 xl:w-3/4 flex flex-col gap-3">
        <div class="flex items-start gap-3">
          <Checkbox
            v-model="launchOnStartup"
            :binary="true"
            inputId="launch-startup"
            disabled
            class="mt-0.5"
          />
          <div class="flex flex-col">
            <label
              for="launch-startup"
              class="text-xs font-semibold text-muted-color cursor-not-allowed select-none"
            >
              Launch Orbit on system startup
            </label>
            <span class="text-[11px] text-muted-color"
              >Automatically start the application when you log in.</span
            >
          </div>
        </div>

        <div class="flex items-start gap-3">
          <Checkbox
            v-model="startMinimized"
            :binary="true"
            inputId="start-minimized"
            disabled
            class="mt-0.5"
          />
          <div class="flex flex-col">
            <label
              for="start-minimized"
              class="text-xs font-semibold text-muted-color cursor-not-allowed select-none"
            >
              Start minimized to system tray
            </label>
            <span class="text-[11px] text-muted-color"
              >Keep Orbit running in the background when launched.</span
            >
          </div>
        </div>
      </div>
    </div>

    <!-- Updates Section -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <h4 class="text-xs font-semibold text-primary uppercase tracking-wider">Updates</h4>
        <p class="text-[11px] text-muted-color">
          Manage release channel and automated update checks.
        </p>
      </div>
      <div class="w-full md:w-2/3 xl:w-3/4 flex flex-col gap-4">
        <div class="flex items-center justify-between gap-4 max-w-lg">
          <div class="flex flex-col">
            <span class="text-xs font-semibold text-primary">Automatically check for updates</span>
            <span class="text-[11px] text-muted-color"
              >Get notified when a new version of Orbit is available.</span
            >
          </div>
          <ToggleSwitch v-model="autoCheckUpdates" disabled />
        </div>

        <div class="flex flex-col gap-1.5 max-w-xs opacity-60">
          <div class="flex items-center gap-2">
            <label class="text-xs font-semibold text-muted-color">Update Channel</label>
            <span
              class="text-[10px] px-1.5 py-0.2 rounded bg-surface-200 dark:bg-surface-800 text-muted-color"
              >Stable only</span
            >
          </div>
          <Select v-model="updateChannel" :options="channels" disabled class="text-xs w-full" />
          <span class="text-[11px] text-muted-color"
            >Beta and Nightly channels will be available in future releases.</span
          >
        </div>
      </div>
    </div>

    <!-- Kubeconfig Section -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10 opacity-60">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <div class="flex items-center gap-2">
          <h4 class="text-xs font-semibold text-primary uppercase tracking-wider">Kubeconfig</h4>
          <span
            class="text-[10px] px-1.5 py-0.2 rounded bg-surface-200 dark:bg-surface-800 text-muted-color"
            >Auto-detected</span
          >
        </div>
        <p class="text-[11px] text-muted-color">File path location used for cluster contexts.</p>
      </div>
      <div class="w-full md:w-2/3 xl:w-3/4 flex flex-col gap-3 max-w-lg">
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-semibold text-muted-color">Default Path</label>
          <div class="flex gap-2">
            <div class="relative flex-1">
              <Folder class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-color" />
              <InputText
                v-model="kubeconfigPath"
                disabled
                class="pl-9 pr-4 py-1.5 w-full text-xs"
              />
            </div>
            <Button
              size="small"
              severity="secondary"
              variant="outlined"
              disabled
              class="text-xs font-semibold flex items-center gap-1.5"
              @click="handleReloadKubeconfig"
            >
              <RefreshCw class="w-3.5 h-3.5" />
              <span>Reload</span>
            </Button>
          </div>
          <span class="text-[11px] text-muted-color"
            >Orbit currently monitors standard system kubeconfig locations.</span
          >
        </div>
      </div>
    </div>

    <!-- Telemetry Section -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10 opacity-60">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <div class="flex items-center gap-2">
          <h4 class="text-xs font-semibold text-primary uppercase tracking-wider">Telemetry</h4>
          <span
            class="text-[10px] px-1.5 py-0.2 rounded bg-surface-200 dark:bg-surface-800 text-muted-color"
            >Disabled</span
          >
        </div>
        <p class="text-[11px] text-muted-color">Help improve Orbit performance and stability.</p>
      </div>
      <div class="w-full md:w-2/3 xl:w-3/4 flex items-center justify-between gap-4 max-w-lg">
        <div class="flex flex-col">
          <span class="text-xs font-semibold text-muted-color"
            >Share anonymous usage statistics</span
          >
          <span class="text-[11px] text-muted-color"
            >Telemetry collection is currently disabled in this build.</span
          >
        </div>
        <ToggleSwitch v-model="shareTelemetry" disabled />
      </div>
    </div>

    <!-- Danger Zone Section -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10 pt-2 opacity-60">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <div class="flex items-center gap-1.5 text-muted-color">
          <AlertTriangle class="w-3.5 h-3.5" />
          <h4 class="text-xs font-semibold uppercase tracking-wider">Danger Zone</h4>
          <span
            class="text-[10px] px-1.5 py-0.2 rounded bg-surface-200 dark:bg-surface-800 text-muted-color"
            >Coming soon</span
          >
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
              @click="handleResetCache"
            >
              Reset Cache
            </Button>
            <Button
              size="small"
              severity="danger"
              variant="outlined"
              disabled
              class="text-xs font-medium"
              @click="handleDeleteCachedContexts"
            >
              Delete Cached Contexts
            </Button>
            <Button
              size="small"
              severity="danger"
              variant="outlined"
              disabled
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
