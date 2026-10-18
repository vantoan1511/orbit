<script setup lang="ts">
import { Check, Copy } from '@lucide/vue'
import Button from 'primevue/button'

defineProps<{
  displayedYaml: string
  isYamlLoading: boolean
  copied: boolean
}>()

const emit = defineEmits<{
  (e: 'copy-yaml'): void
}>()
</script>

<template>
  <div data-component="resource-yaml-tab" class="h-full flex flex-col space-y-3">
    <div class="flex items-center justify-between">
      <span class="text-[10px] font-bold text-muted-color uppercase tracking-wider">
        Live Kubernetes Manifest
      </span>
      <Button
        severity="secondary"
        size="small"
        variant="outlined"
        class="text-xs"
        @click="emit('copy-yaml')"
      >
        <Component :is="copied ? Check : Copy" class="w-3.5 h-3.5 mr-1.5" />
        <span>{{ copied ? 'Copied!' : 'Copy YAML' }}</span>
      </Button>
    </div>

    <div
      class="flex-1 min-h-64 bg-(--bg-card) rounded-[var(--ui-radius-card)] border border-(--border) p-4 overflow-auto font-mono text-[10px] text-primary leading-relaxed"
    >
      <div v-if="isYamlLoading" class="text-muted-color italic py-4 text-center">
        Loading live YAML manifest...
      </div>
      <pre v-else>{{ displayedYaml }}</pre>
    </div>
  </div>
</template>
