<script setup lang="ts">
import Button from 'primevue/button'
import { Boxes, Plus } from '@lucide/vue'

const emit = defineEmits<{
  (e: 'open-picker', event: MouseEvent): void
}>()
</script>

<template>
  <div class="flex flex-col items-center justify-center min-h-[60vh] select-none text-center px-4">
    <div
      class="flex items-center justify-center w-16 h-16 rounded-2xl bg-(--bg-card) border border-(--border) mb-4 text-muted-color shadow-sm"
    >
      <Boxes class="w-8 h-8 text-muted-color/80" />
    </div>

    <h2 class="text-base font-semibold text-primary mb-1 tracking-tight">No Open Tabs</h2>
    <p class="text-xs text-muted-color max-w-sm mb-6 leading-relaxed">
      Select a resource from the sidebar or open a new tab to start inspecting your cluster.
    </p>

    <div class="flex items-center gap-3">
      <Button
        severity="primary"
        size="small"
        class="flex items-center gap-2 px-3 py-1.5 text-xs font-medium rounded-md shadow-xs cursor-pointer"
        v-tooltip.bottom="'Open resource picker'"
        @click="emit('open-picker', $event)"
      >
        <Plus class="w-3.5 h-3.5" />
        <span>Open New Tab</span>
      </Button>
    </div>
  </div>
</template>
