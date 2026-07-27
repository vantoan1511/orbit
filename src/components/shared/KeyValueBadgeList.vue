<script setup lang="ts">
import { Tag as TagIcon } from '@lucide/vue'

withDefaults(
  defineProps<{
    items?: Record<string, string> | null
    title?: string
    variant?: 'tag' | 'list'
  }>(),
  {
    items: () => ({}),
    title: '',
    variant: 'tag'
  }
)
</script>

<template>
  <div v-if="items && Object.keys(items).length > 0">
    <h3
      v-if="title"
      class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3"
    >
      {{ title }}
    </h3>

    <!-- Tag Variant (used for Labels) -->
    <div v-if="variant === 'tag'" class="flex flex-wrap gap-2">
      <div
        v-for="(val, key) in items"
        :key="key"
        class="flex items-center gap-1 bg-violet-500/5 border border-violet-500/10 rounded-md text-[10px] px-2 py-0.5 text-violet-400 font-mono"
      >
        <TagIcon class="w-3 h-3" />
        <span>{{ key }}={{ val }}</span>
      </div>
    </div>

    <!-- List Variant (used for Annotations) -->
    <div v-else class="space-y-1.5">
      <div
        v-for="(val, key) in items"
        :key="key"
        class="p-2 rounded bg-(--bg-hover)/50 border border-(--border) text-[10px] font-mono text-(--text-secondary) flex justify-between gap-4"
      >
        <span class="text-(--text-muted) truncate shrink-0">{{ key }}</span>
        <span class="truncate text-right">{{ val }}</span>
      </div>
    </div>
  </div>
</template>
