<script setup lang="ts">
import { computed, ref } from 'vue'
import { categories, type CategoryId } from './navigation'
import { useResizable } from '@/composables/useResizable'

const props = withDefaults(
  defineProps<{
    activeTab: CategoryId | null
    minWidth?: number
    maxWidth?: number
    defaultWidth?: number
    collapseThreshold?: number
    storageKey?: string
  }>(),
  {
    minWidth: 180,
    maxWidth: 600,
    defaultWidth: 260,
    collapseThreshold: 90,
    storageKey: 'orbit_sidebar_panel_width'
  }
)

const emit = defineEmits<{
  (e: 'collapse'): void
}>()

const currentCategoryTitle = computed(() => {
  if (!props.activeTab) return ''
  const cat = categories.find((c) => c.id === props.activeTab)
  return cat ? cat.name : ''
})

const panelRef = ref<HTMLElement | null>(null)

const { width: panelWidth, startResize: triggerResize } = useResizable({
  minWidth: props.minWidth,
  maxWidth: props.maxWidth,
  defaultWidth: props.defaultWidth,
  collapseThreshold: props.collapseThreshold,
  storageKey: props.storageKey,
  onCollapse: () => emit('collapse')
})

const startResize = (e: MouseEvent) => {
  triggerResize(e, panelRef.value)
}
</script>

<template>
  <div
    v-if="activeTab"
    ref="panelRef"
    :style="{ width: `${panelWidth}px` }"
    class="relative flex flex-col h-full overflow-hidden border-r border-(--border) bg-(--bg-sidebar) select-none shrink-0"
  >
    <!-- Panel Header -->
    <div class="h-10 px-4 flex items-center justify-between border-b border-(--border) shrink-0">
      <span class="text-sm font-medium tracking-tighter uppercase text-primary font-ui truncate">
        {{ activeTab === 'clusters' ? 'Clusters' : currentCategoryTitle }}
      </span>
    </div>

    <!-- Panel Content -->
    <slot />

    <!-- Resize Handle -->
    <div
      class="absolute top-0 right-0 w-1.5 h-full cursor-col-resize hover:bg-primary/40 active:bg-primary transition-colors z-20 -mr-0.5"
      @mousedown="startResize"
    />
  </div>
</template>
