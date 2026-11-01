<script setup lang="ts">
import { computed } from 'vue'
import { useSettingsStore } from '@/stores/settingsStore'
import { CONFIG_KEYS, type NotificationPreferences } from '@/types/settings'
import { RotateCcw } from '@lucide/vue'
import Button from 'primevue/button'
import ToggleSwitch from 'primevue/toggleswitch'
import { useToast } from 'primevue/usetoast'

const settingsStore = useSettingsStore()
const toast = useToast()

const DEFAULT_PREFERENCES: NotificationPreferences = {
  enableToasts: true,
  notifyOnSuccess: true,
  notifyOnError: true,
  notifyOnUpdates: true
}

const preferences = computed<NotificationPreferences>(() => {
  return settingsStore.getConfigValue<NotificationPreferences>(
    CONFIG_KEYS.NOTIFICATION_PREFERENCES,
    DEFAULT_PREFERENCES
  )
})

const isModified = computed(() => {
  const current = preferences.value
  return (
    current.enableToasts !== DEFAULT_PREFERENCES.enableToasts ||
    current.notifyOnSuccess !== DEFAULT_PREFERENCES.notifyOnSuccess ||
    current.notifyOnError !== DEFAULT_PREFERENCES.notifyOnError ||
    current.notifyOnUpdates !== DEFAULT_PREFERENCES.notifyOnUpdates
  )
})

async function updatePreference<K extends keyof NotificationPreferences>(
  key: K,
  val: NotificationPreferences[K]
) {
  const updated: NotificationPreferences = {
    ...preferences.value,
    [key]: val
  }
  await settingsStore.updateConfigValue(CONFIG_KEYS.NOTIFICATION_PREFERENCES, updated)
}

async function handleReset() {
  await settingsStore.updateConfigValue(CONFIG_KEYS.NOTIFICATION_PREFERENCES, DEFAULT_PREFERENCES)
  toast.add({
    severity: 'success',
    summary: 'Notifications Reset',
    detail: 'Notification preferences have been restored to defaults.',
    life: 2500
  })
}
</script>

<template>
  <div data-component="settings-notifications-tab" class="flex flex-col gap-10">
    <!-- Header row with Title and Reset Action -->
    <div class="flex items-start justify-between gap-4">
      <div>
        <h3 class="text-base font-semibold text-primary">Notifications</h3>
        <p class="text-xs text-muted-color mt-1">
          Configure application toast alerts, event notifications, and update popups.
        </p>
      </div>

      <Button
        type="button"
        :disabled="!isModified"
        v-tooltip.top="
          isModified
            ? 'Reset all notification preferences to defaults'
            : 'Notification preferences are currently set to defaults'
        "
        severity="secondary"
        variant="text"
        size="small"
        class="text-xs text-muted-color hover:text-primary shrink-0 transition-opacity flex items-center gap-1.5"
        @click="handleReset"
      >
        <RotateCcw class="w-3.5 h-3.5" />
        <span>Reset to Defaults</span>
      </Button>
    </div>

    <!-- Section 1: Master Toast Switch -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <h4 class="text-xs font-semibold text-primary uppercase tracking-wider">
          Toast Notifications
        </h4>
        <p class="text-[11px] text-muted-color">
          Master switch to enable or suppress all transient pop-up toasts on the screen.
        </p>
      </div>

      <div
        class="w-full md:w-2/3 xl:w-3/4 flex items-center justify-between p-4 bg-surface-50 dark:bg-surface-900 rounded-[var(--ui-radius-card)]"
      >
        <div class="flex flex-col gap-0.5">
          <span class="text-xs font-medium text-primary">Enable Toast Popups</span>
          <span class="text-[11px] text-muted-color">
            Display floating transient notifications for cluster status and system events
          </span>
        </div>
        <ToggleSwitch
          :model-value="preferences.enableToasts"
          @update:model-value="(val) => updatePreference('enableToasts', Boolean(val))"
        />
      </div>
    </div>

    <!-- Section 2: Event Notification Toggles -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <h4 class="text-xs font-semibold text-primary uppercase tracking-wider">Event Triggers</h4>
        <p class="text-[11px] text-muted-color">
          Fine-tune which operational categories generate notification toasts.
        </p>
      </div>

      <div class="w-full md:w-2/3 xl:w-3/4 flex flex-col gap-3">
        <!-- Success events -->
        <div
          class="flex items-center justify-between p-4 bg-surface-50 dark:bg-surface-900 rounded-[var(--ui-radius-card)]"
        >
          <div class="flex flex-col gap-0.5">
            <span class="text-xs font-medium text-primary">Command & Operation Success</span>
            <span class="text-[11px] text-muted-color">
              Notify when Kubernetes resource creation, edits, scale, and delete operations succeed
            </span>
          </div>
          <ToggleSwitch
            :disabled="!preferences.enableToasts"
            :model-value="preferences.notifyOnSuccess"
            @update:model-value="(val) => updatePreference('notifyOnSuccess', Boolean(val))"
          />
        </div>

        <!-- Error events -->
        <div
          class="flex items-center justify-between p-4 bg-surface-50 dark:bg-surface-900 rounded-[var(--ui-radius-card)]"
        >
          <div class="flex flex-col gap-0.5">
            <span class="text-xs font-medium text-primary">Errors & Connection Failures</span>
            <span class="text-[11px] text-muted-color">
              Notify when API errors, network disconnects, or operation failures occur
            </span>
          </div>
          <ToggleSwitch
            :disabled="!preferences.enableToasts"
            :model-value="preferences.notifyOnError"
            @update:model-value="(val) => updatePreference('notifyOnError', Boolean(val))"
          />
        </div>

        <!-- Update events -->
        <div
          class="flex items-center justify-between p-4 bg-surface-50 dark:bg-surface-900 rounded-[var(--ui-radius-card)]"
        >
          <div class="flex flex-col gap-0.5">
            <span class="text-xs font-medium text-primary">Application Updates</span>
            <span class="text-[11px] text-muted-color">
              Notify when newer versions or release packages are detected
            </span>
          </div>
          <ToggleSwitch
            :disabled="!preferences.enableToasts"
            :model-value="preferences.notifyOnUpdates"
            @update:model-value="(val) => updatePreference('notifyOnUpdates', Boolean(val))"
          />
        </div>
      </div>
    </div>
  </div>
</template>
