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
  <div class="flex flex-col gap-6">
    <!-- General Settings Form Card -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-6 shadow-sm flex flex-col gap-6"
    >
      <div class="border-b border-(--border) pb-4">
        <h3 class="text-base font-semibold text-(--text-primary)">General Settings</h3>
        <p class="text-xs text-(--text-muted) mt-1">
          Configure startup, updates, and core application behaviors.
        </p>
      </div>

      <!-- Startup Section -->
      <div class="flex flex-col gap-4">
        <h4 class="text-sm font-semibold text-(--text-primary)">Startup</h4>
        <div class="flex flex-col gap-3">
          <div class="flex items-start gap-3">
            <Checkbox
              v-model="launchOnStartup"
              :binary="true"
              inputId="launch-startup"
              class="mt-0.5"
            />
            <div class="flex flex-col">
              <label
                for="launch-startup"
                class="text-xs font-semibold text-(--text-secondary) cursor-pointer select-none"
              >
                Launch Orbit on system startup
              </label>
              <span class="text-[11px] text-(--text-muted)"
                >Automatically start the application when you log in.</span
              >
            </div>
          </div>

          <div class="flex items-start gap-3">
            <Checkbox
              v-model="startMinimized"
              :binary="true"
              inputId="start-minimized"
              class="mt-0.5"
            />
            <div class="flex flex-col">
              <label
                for="start-minimized"
                class="text-xs font-semibold text-(--text-secondary) cursor-pointer select-none"
              >
                Start minimized to system tray
              </label>
              <span class="text-[11px] text-(--text-muted)"
                >Keep Orbit running in the background when launched.</span
              >
            </div>
          </div>
        </div>
      </div>

      <hr class="border-(--border)" />

      <!-- Updates Section -->
      <div class="flex flex-col gap-4">
        <h4 class="text-sm font-semibold text-(--text-primary)">Updates</h4>
        <div class="flex flex-col gap-4">
          <div class="flex items-center justify-between gap-4">
            <div class="flex flex-col">
              <span class="text-xs font-semibold text-(--text-secondary)"
                >Automatically check for updates</span
              >
              <span class="text-[11px] text-(--text-muted)"
                >Get notified when a new version of Orbit is available.</span
              >
            </div>
            <ToggleSwitch v-model="autoCheckUpdates" />
          </div>

          <div class="flex flex-col gap-2 max-w-sm">
            <label class="text-xs font-semibold text-(--text-secondary)">Update Channel</label>
            <Select
              v-model="updateChannel"
              :options="channels"
              class="text-xs w-full bg-(--bg-hover)/30 border-(--border)"
            />
            <span class="text-[11px] text-(--text-muted)"
              >Choose Beta or Nightly to test experimental features early.</span
            >
          </div>
        </div>
      </div>

      <hr class="border-(--border)" />

      <!-- Kubeconfig Section -->
      <div class="flex flex-col gap-4">
        <h4 class="text-sm font-semibold text-(--text-primary)">Kubeconfig</h4>
        <div class="flex flex-col gap-3">
          <div class="flex flex-col gap-2">
            <label class="text-xs font-semibold text-(--text-secondary)">Default Path</label>
            <div class="flex gap-2">
              <div class="relative flex-1">
                <Folder
                  class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-(--text-muted)"
                />
                <InputText
                  v-model="kubeconfigPath"
                  class="pl-9 pr-4 py-2 w-full text-xs bg-(--bg-hover)/30 border-(--border) text-(--text-primary) rounded-lg focus:border-violet-500 focus:ring-1 focus:ring-violet-500"
                />
              </div>
              <Button
                @click="handleReloadKubeconfig"
                class="flex items-center gap-2 px-4 py-2 text-xs font-semibold bg-[var(--bg-active)] border border-[var(--border)] rounded-lg text-[var(--text-primary)] hover:bg-[var(--bg-hover)] transition-all"
              >
                <RefreshCw class="w-3.5 h-3.5" />
                Reload
              </Button>
            </div>
            <span class="text-[11px] text-(--text-muted)"
              >Orbit will monitor this path for Kubernetes config changes.</span
            >
          </div>
        </div>
      </div>

      <hr class="border-(--border)" />

      <!-- Telemetry Section -->
      <div class="flex flex-col gap-4">
        <h4 class="text-sm font-semibold text-(--text-primary)">Telemetry</h4>
        <div class="flex items-center justify-between gap-4">
          <div class="flex flex-col">
            <span class="text-xs font-semibold text-(--text-secondary)"
              >Share anonymous usage statistics</span
            >
            <span class="text-[11px] text-(--text-muted)"
              >Help us improve Orbit by sending telemetry data. No secrets or PII are
              collected.</span
            >
          </div>
          <ToggleSwitch v-model="shareTelemetry" />
        </div>
      </div>
    </div>

    <!-- Danger Zone Card -->
    <div class="bg-red-500/5 border border-red-500/20 rounded-xl p-6 shadow-sm flex flex-col gap-6">
      <div>
        <div class="flex items-center gap-2 text-red-500">
          <AlertTriangle class="w-4 h-4" />
          <h3 class="text-base font-semibold">Danger Zone</h3>
        </div>
        <p class="text-xs text-(--text-muted) mt-1">
          Actions here can result in data loss or configuration resetting.
        </p>
      </div>

      <div class="flex flex-wrap gap-3">
        <Button
          @click="handleResetCache"
          class="px-4 py-2 text-xs font-semibold border border-red-500/30 rounded-lg text-red-500 hover:bg-red-500/10 transition-all bg-transparent cursor-pointer"
        >
          Reset Application Cache
        </Button>
        <Button
          @click="handleDeleteCachedContexts"
          class="px-4 py-2 text-xs font-semibold border border-red-500/30 rounded-lg text-red-500 hover:bg-red-500/10 transition-all bg-transparent cursor-pointer"
        >
          Delete Cached Contexts
        </Button>
        <Button
          @click="handleResetAllSettings"
          class="px-4 py-2 text-xs font-semibold border border-red-500/30 rounded-lg text-red-500 hover:bg-red-500/10 transition-all bg-transparent cursor-pointer"
        >
          Reset All Settings
        </Button>
      </div>
    </div>
  </div>
</template>
