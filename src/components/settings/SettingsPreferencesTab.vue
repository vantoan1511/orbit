<script setup lang="ts">
import { useShortcutStore } from '@/stores/shortcutStore'
import { RotateCcw } from '@lucide/vue'
import Button from 'primevue/button'
import { useToast } from 'primevue/usetoast'
import KeyboardShortcutsList from './preferences/KeyboardShortcutsList.vue'

const shortcutStore = useShortcutStore()
const toast = useToast()

async function handleResetAll() {
  await shortcutStore.resetAll()
  toast.add({
    severity: 'success',
    summary: 'Shortcuts Reset',
    detail: 'All keyboard shortcuts have been restored to defaults.',
    life: 2500
  })
}
</script>

<template>
  <div class="flex flex-col gap-10">
    <!-- Header row with Title and Reset All Action -->
    <div class="flex items-start justify-between gap-4">
      <div>
        <h3 class="text-base font-semibold text-primary">Preferences</h3>
        <p class="text-xs text-muted-color mt-1">
          Configure application behaviors, global interaction keys, and developer preferences.
        </p>
      </div>

      <Button
        type="button"
        :disabled="!shortcutStore.hasAnyCustomizations"
        v-tooltip.top="
          shortcutStore.hasAnyCustomizations
            ? 'Reset all shortcuts to defaults'
            : 'All shortcuts are currently set to defaults'
        "
        severity="secondary"
        variant="text"
        size="small"
        class="text-xs text-muted-color hover:text-primary shrink-0 transition-opacity"
        @click="handleResetAll"
      >
        <template #icon>
          <RotateCcw class="w-3.5 h-3.5 mr-1" />
        </template>
        Reset All Shortcuts
      </Button>
    </div>

    <!-- Section 1: Keyboard Shortcuts -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <h4 class="text-xs font-semibold text-primary uppercase tracking-wider">
          Keyboard Shortcuts
        </h4>
        <p class="text-[11px] text-muted-color">
          Customize hotkeys for rapid workspace navigation, tab management, and terminal toggling.
        </p>
      </div>
      <div class="w-full md:w-2/3 xl:w-3/4">
        <KeyboardShortcutsList />
      </div>
    </div>
  </div>
</template>
