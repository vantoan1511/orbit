<script setup lang="ts">
import NamespaceBadge from '@/components/shared/NamespaceBadge.vue'
import { Clock } from '@lucide/vue'

defineProps<{
  name: string
  kind?: string
  namespace?: string
  age?: string
}>()
</script>

<template>
  <div class="flex flex-col">
    <!-- Header bar inside drawer -->
    <div class="flex items-center gap-2 mb-2">
      <span
        v-if="kind"
        class="text-xs font-semibold text-blue-400 bg-blue-500/10 px-2 py-0.5 rounded border border-blue-500/20"
      >
        {{ kind }}
      </span>
      <NamespaceBadge v-if="namespace" :namespace="namespace" />
    </div>

    <!-- Title & Age section -->
    <div class="p-4 border-b border-(--border) bg-(--bg-hover)/50 rounded-lg">
      <h2 class="text-lg font-bold text-primary font-ui truncate mb-1">
        {{ name }}
      </h2>
      <div v-if="age" class="text-xs text-muted-color flex items-center gap-2">
        <Clock class="w-3.5 h-3.5" />
        <span>Age: {{ age }}</span>
      </div>
    </div>
  </div>
</template>
