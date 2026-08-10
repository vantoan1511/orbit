<script setup lang="ts">
import { computed, onUnmounted, ref } from 'vue'
import { categories, type CategoryId } from './navigation'

const props = defineProps<{
  activeTab: CategoryId | null
}>()

const emit = defineEmits<{
  (e: 'collapse'): void
}>()

const currentCategoryTitle = computed(() => {
  if (!props.activeTab) return ''
  const cat = categories.find((c) => c.id === props.activeTab)
  return cat ? cat.name : ''
})

const panelRef = ref<HTMLElement | null>(null)
const minWidth = 180
const maxWidth = 600
const defaultWidth = 260
const collapseThreshold = 90

const initialWidth = parseInt(
  localStorage.getItem('orbit_sidebar_panel_width') || `${defaultWidth}`,
  10
)
const panelWidth = ref<number>(
  isNaN(initialWidth) || initialWidth < minWidth ? defaultWidth : initialWidth
)
const isResizing = ref(false)
let startLeft = 0
let lastValidWidth = panelWidth.value

const startResize = (e: MouseEvent) => {
  e.preventDefault()
  if (!panelRef.value) return
  isResizing.value = true
  startLeft = panelRef.value.getBoundingClientRect().left
  lastValidWidth = panelWidth.value > 0 ? panelWidth.value : defaultWidth

  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', stopResize)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
}

const handleMouseMove = (e: MouseEvent) => {
  if (!isResizing.value) return
  const rawWidth = e.clientX - startLeft

  if (rawWidth < collapseThreshold) {
    panelWidth.value = 0
  } else {
    panelWidth.value = Math.max(minWidth, Math.min(maxWidth, rawWidth))
    lastValidWidth = panelWidth.value
  }
}

const stopResize = () => {
  if (!isResizing.value) return
  isResizing.value = false
  document.removeEventListener('mousemove', handleMouseMove)
  document.removeEventListener('mouseup', stopResize)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''

  if (panelWidth.value < collapseThreshold) {
    panelWidth.value = lastValidWidth >= minWidth ? lastValidWidth : defaultWidth
    emit('collapse')
  } else {
    localStorage.setItem('orbit_sidebar_panel_width', panelWidth.value.toString())
  }
}

onUnmounted(() => {
  stopResize()
})
</script>

<template>
  <div
    v-if="activeTab"
    ref="panelRef"
    :style="{ width: `${panelWidth}px` }"
    class="relative flex flex-col h-full overflow-hidden border-r border-surface-200 dark:border-surface-700 bg-surface-0 dark:bg-surface-900 select-none shrink-0"
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

    <!-- Resize Handle -->
    <div
      class="absolute top-0 right-0 w-1.5 h-full cursor-col-resize hover:bg-primary/40 active:bg-primary transition-colors z-20 -mr-0.5"
      @mousedown="startResize"
    />
  </div>
</template>
