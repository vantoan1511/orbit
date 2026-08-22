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
    <h3 v-if="title" class="text-[10px] font-bold text-muted-color uppercase tracking-wider mb-3">
      {{ title }}
    </h3>

    <!-- Tag Variant (used for Labels) -->
    <div v-if="variant === 'tag'" class="flex flex-wrap gap-2">
      <Tag
        v-for="(val, key) in items"
        :key="key"
        severity="secondary"
        class="font-mono"
        :value="`${key}=${val}`"
      >
        <template #icon>
          <TagIcon class="w-3 h-3" />
        </template>
      </Tag>
    </div>

    <!-- List Variant (used for Annotations) -->
    <div v-else class="space-y-1.5">
      <div
        v-for="(val, key) in items"
        :key="key"
        class="p-2 rounded bg-(--bg-hover)/50 border border-(--border) text-[10px] font-mono text-muted-color flex justify-between gap-4"
      >
        <span class="text-muted-color truncate shrink-0">{{ key }}</span>
        <span class="truncate text-right">{{ val }}</span>
      </div>
    </div>
  </div>
</template>
