<script setup lang="ts">
import type { ServiceInfo } from '@/types/kubernetes'

defineProps<{
  service: ServiceInfo
}>()
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <h3 class="text-[10px] font-bold text-muted-color uppercase tracking-wider">
        Target Endpoints ({{ service.endpointsList.length }})
      </h3>
    </div>

    <div
      v-if="service.endpointsList.length > 0"
      class="bg-(--bg-hover)/40 rounded-xl p-4 space-y-2"
    >
      <div
        v-for="(ep, idx) in service.endpointsList"
        :key="idx"
        class="font-mono text-xs text-primary flex items-center gap-2.5 py-1"
      >
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 shrink-0"></span>
        <span class="truncate">{{ ep }}</span>
      </div>
    </div>

    <div v-else class="bg-(--bg-hover)/20 rounded-xl p-4 text-xs text-muted-color italic">
      No active endpoints detected for this service (ExternalName or selectorless service).
    </div>
  </div>
</template>
