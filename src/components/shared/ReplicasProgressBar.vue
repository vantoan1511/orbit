<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  desired?: number
  current?: number
  ready?: number
  available?: number
}>()

const safeDesired = computed(() => props.desired ?? 0)

const currentPct = computed(() => {
  if (safeDesired.value <= 0) return 0
  return Math.min(100, Math.round(((props.current ?? 0) / safeDesired.value) * 100))
})

const readyPct = computed(() => {
  if (safeDesired.value <= 0 || props.ready === undefined) return 0
  return Math.min(100, Math.round((props.ready / safeDesired.value) * 100))
})

const availablePct = computed(() => {
  if (safeDesired.value <= 0 || props.available === undefined) return 0
  return Math.min(100, Math.round((props.available / safeDesired.value) * 100))
})
</script>

<template>
  <div v-if="desired !== undefined">
    <h3 class="text-[10px] font-bold text-muted-color uppercase tracking-wider mb-3">
      Replicas Status
    </h3>
    <div class="bg-(--bg-hover)/50 border border-(--border) rounded-xl p-4 space-y-4">
      <div>
        <div class="flex justify-between text-xs mb-1">
          <span class="text-muted-color font-medium">Desired Replicas</span>
          <span class="font-mono font-bold text-primary">{{ desired }}</span>
        </div>
        <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
          <div class="h-full rounded-full bg-blue-500" style="width: 100%"></div>
        </div>
      </div>

      <div v-if="current !== undefined">
        <div class="flex justify-between text-xs mb-1">
          <span class="text-muted-color font-medium">Current Replicas</span>
          <span class="font-mono font-bold text-primary">{{ current }}</span>
        </div>
        <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
          <div class="h-full rounded-full bg-indigo-500" :style="{ width: currentPct + '%' }"></div>
        </div>
      </div>

      <div v-if="ready !== undefined">
        <div class="flex justify-between text-xs mb-1">
          <span class="text-muted-color font-medium">Ready Replicas</span>
          <span class="font-mono font-bold text-primary">{{ ready }}</span>
        </div>
        <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
          <div class="h-full rounded-full bg-emerald-500" :style="{ width: readyPct + '%' }"></div>
        </div>
      </div>

      <div v-if="available !== undefined">
        <div class="flex justify-between text-xs mb-1">
          <span class="text-muted-color font-medium">Available Replicas</span>
          <span class="font-mono font-bold text-primary">{{ available }}</span>
        </div>
        <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
          <div class="h-full rounded-full bg-teal-500" :style="{ width: availablePct + '%' }"></div>
        </div>
      </div>
    </div>
  </div>
</template>
