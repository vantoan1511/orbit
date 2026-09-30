<script setup lang="ts">
import type { AppTab } from '@/types/tabs'
import { getTabMetadataForRoute } from '@/utils/tabIcons'
import { getTabCloseButtonClass, getTabContainerClass } from '@/utils/tabStyles'
import { X } from '@lucide/vue'
import Button from 'primevue/button'
import ContextMenu from 'primevue/contextmenu'
import type { MenuItem } from 'primevue/menuitem'
import { computed, ref } from 'vue'

const props = withDefaults(
  defineProps<{
    tab: AppTab
    isActive: boolean
    index?: number
    totalTabs?: number
    isDragging?: boolean
    dropIndicator?: 'before' | 'after' | null
  }>(),
  {
    index: 0,
    totalTabs: 1,
    isDragging: false,
    dropIndicator: null
  }
)

const emit = defineEmits<{
  (e: 'select'): void
  (e: 'close'): void
  (e: 'close-others'): void
  (e: 'close-right'): void
  (e: 'close-all'): void
  (e: 'move-left'): void
  (e: 'move-right'): void
}>()

const contextMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null)

const metadata = computed(() => getTabMetadataForRoute(props.tab.path, props.tab.query))

const displayTitle = computed(() => {
  return props.tab.title && props.tab.title !== 'Tab' ? props.tab.title : metadata.value.title
})

const tabTooltip = computed(() => {
  if (props.tab.path === '/logs') {
    const ns = props.tab.query?.namespace
    return ns ? `Logs: ${displayTitle.value} (${ns})` : `Logs: ${displayTitle.value}`
  }
  return displayTitle.value
})

const isMac =
  typeof navigator !== 'undefined' &&
  navigator.platform &&
  navigator.platform.toUpperCase().indexOf('MAC') >= 0

interface TabMenuItem extends MenuItem {
  shortcut?: string
}

const menuItems = computed<TabMenuItem[]>(() => [
  {
    label: 'Move Tab Left',
    shortcut: isMac ? '⌥←' : 'Alt+←',
    disabled: props.index <= 0,
    command: () => emit('move-left')
  },
  {
    label: 'Move Tab Right',
    shortcut: isMac ? '⌥→' : 'Alt+→',
    disabled: props.index >= props.totalTabs - 1,
    command: () => emit('move-right')
  },
  {
    separator: true
  },
  {
    label: 'Close Tab',
    shortcut: isMac ? '⌘W' : 'Ctrl+W',
    command: () => emit('close')
  },
  {
    label: 'Close Others',
    command: () => emit('close-others')
  },
  {
    label: 'Close Tabs to the Right',
    command: () => emit('close-right')
  },
  {
    separator: true
  },
  {
    label: 'Close All',
    command: () => emit('close-all')
  }
])

const handleContextMenu = (event: MouseEvent) => {
  event.preventDefault()
  contextMenuRef.value?.show(event)
}

const handleCloseClick = (event: MouseEvent) => {
  event.stopPropagation()
  emit('close')
}

const handleMiddleClick = () => {
  emit('close')
}
</script>

<template>
  <div
    :class="[getTabContainerClass(isActive), isDragging && 'opacity-40']"
    draggable="true"
    @click="emit('select')"
    @mousedown.middle.prevent="handleMiddleClick"
    @contextmenu="handleContextMenu"
  >
    <!-- Drop Indicator Line (VS Code style 2px accent bar) -->
    <div
      v-if="dropIndicator === 'before'"
      class="absolute left-0 top-0 bottom-0 w-0.5 bg-(--accent) z-20 pointer-events-none"
    />
    <div
      v-if="dropIndicator === 'after'"
      class="absolute right-0 top-0 bottom-0 w-0.5 bg-(--accent) z-20 pointer-events-none"
    />

    <!-- Resource Icon -->
    <component
      :is="metadata.icon"
      :class="[
        'w-3.5 h-3.5 shrink-0 transition-colors',
        isActive ? metadata.iconColorClass : 'text-muted-color'
      ]"
    />

    <!-- Tab Title -->
    <span v-tooltip.bottom="tabTooltip" class="truncate flex-1 font-medium">
      {{ displayTitle }}
    </span>

    <!-- Close Button -->
    <Button
      v-if="tab.closable !== false"
      variant="text"
      rounded
      size="small"
      :class="getTabCloseButtonClass(isActive)"
      v-tooltip.bottom="'Close (Ctrl+W)'"
      draggable="false"
      @dragstart.stop.prevent
      @click="handleCloseClick"
    >
      <template #icon>
        <X class="w-3 h-3" />
      </template>
    </Button>

    <!-- Context Menu -->
    <ContextMenu
      ref="contextMenuRef"
      :model="menuItems"
      class="min-w-48 bg-(--bg-card) border border-(--border) p-1 rounded-lg shadow-xl text-xs select-none"
    >
      <template #item="{ item, props: menuProps }">
        <div v-if="item.separator" class="h-px bg-(--border) my-1 mx-1" />
        <a
          v-else
          v-bind="menuProps.action"
          :class="[
            'flex items-center justify-between w-full h-7 px-2.5 rounded-md text-xs font-medium transition-colors select-none no-underline',
            item.disabled
              ? 'opacity-40 cursor-not-allowed pointer-events-none text-muted-color'
              : 'text-muted-color hover:text-primary hover:bg-(--bg-hover) cursor-pointer'
          ]"
        >
          <span>{{ item.label }}</span>
          <kbd
            v-if="(item as TabMenuItem).shortcut"
            class="ml-4 text-[10px] font-mono text-muted-color tracking-normal"
          >
            {{ (item as TabMenuItem).shortcut }}
          </kbd>
        </a>
      </template>
    </ContextMenu>
  </div>
</template>
