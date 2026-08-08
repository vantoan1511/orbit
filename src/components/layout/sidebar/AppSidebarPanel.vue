<script setup lang="ts">
import { computed } from 'vue'
import { categories, type CategoryId } from './navigation'

const props = defineProps<{
  activeTab: CategoryId | null
}>()

const currentCategoryTitle = computed(() => {
  if (!props.activeTab) return ''
  const cat = categories.find((c) => c.id === props.activeTab)
  return cat ? cat.name : ''
})
</script>

<template>
  <div
    v-if="activeTab"
    class="w-52 flex flex-col h-full overflow-hidden border-r border-surface-200 dark:border-surface-700 bg-surface-0 dark:bg-surface-900 select-none"
  >
    <!-- Panel Header -->
    <div
      class="h-10 px-4 flex items-center justify-between border-b border-surface-100 dark:border-surface-800 shrink-0"
    >
      <span class="text-sm font-medium tracking-tighter uppercase text-primary font-ui truncate">
        {{ activeTab === 'clusters' ? 'Clusters' : currentCategoryTitle }}
      </span>
    </div>

    <!-- Panel Content -->
    <slot />
  </div>
</template>
