<script setup lang="ts">
import ContextMenu from 'primevue/contextmenu'
import Menu from 'primevue/menu'
import type { MenuItem } from 'primevue/menuitem'
import { computed, h, ref } from 'vue'

const props = defineProps<{
  items: MenuItem[]
}>()

const menuModel = computed(() => props.items)

const menuRef = ref<InstanceType<typeof Menu> | null>(null)
const contextMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null)

const show = (event: Event) => {
  contextMenuRef.value?.show(event)
}

const hide = () => {
  menuRef.value?.hide()
  contextMenuRef.value?.hide()
}

const toggle = (event: Event) => {
  menuRef.value?.toggle(event)
}

defineExpose({
  show,
  hide,
  toggle
})

/* eslint-disable @typescript-eslint/no-explicit-any */
const MenuItemButton = (props: { item: MenuItem; menuProps: any }) => {
  return h(
    'button',
    {
      ...props.menuProps?.action,
      class: [
        'flex items-center gap-2 px-3 py-2 w-full text-left text-xs text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--bg-hover)/20 rounded-md transition-colors cursor-pointer select-none',
        props.item.class
      ]
    },
    [
      props.item.icon ? h('i', { class: [props.item.icon, 'w-4 h-4 shrink-0'] }) : null,
      h('span', props.item.label)
    ]
  )
}
</script>

<template>
  <Menu
    ref="menuRef"
    :model="menuModel"
    popup
    class="min-w-44 bg-(--bg-card) border border-(--border) p-1 rounded-lg shadow-lg"
  >
    <template #item="{ item, props: menuProps }">
      <MenuItemButton :item="item" :menuProps="menuProps" />
    </template>
  </Menu>

  <ContextMenu
    ref="contextMenuRef"
    :model="menuModel"
    class="min-w-44 bg-(--bg-card) border border-(--border) p-1 rounded-lg shadow-lg"
  >
    <template #item="{ item, props: menuProps }">
      <MenuItemButton :item="item" :menuProps="menuProps" />
    </template>
  </ContextMenu>
</template>
