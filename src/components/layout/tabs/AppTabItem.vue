<script setup lang="ts">
import type { AppTab } from '@/types/tabs'
import { getTabMetadataForRoute } from '@/utils/tabIcons'
import { getTabCloseButtonClass, getTabContainerClass } from '@/utils/tabStyles'
import { X } from '@lucide/vue'
import Button from 'primevue/button'
import ContextMenu from 'primevue/contextmenu'
import type { MenuItem } from 'primevue/menuitem'
import { computed, ref } from 'vue'

const props = defineProps<{
  tab: AppTab
  isActive: boolean
}>()

const emit = defineEmits<{
  (e: 'select'): void
  (e: 'close'): void
  (e: 'close-others'): void
  (e: 'close-right'): void
  (e: 'close-all'): void
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
    :class="getTabContainerClass(isActive)"
    @click="emit('select')"
    @mousedown.middle.prevent="handleMiddleClick"
    @contextmenu="handleContextMenu"
  >
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
          class="flex items-center justify-between w-full h-7 px-2.5 rounded-md text-xs font-medium text-muted-color hover:text-primary hover:bg-(--bg-hover) transition-colors cursor-pointer select-none no-underline"
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
