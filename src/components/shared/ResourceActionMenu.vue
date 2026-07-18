<script setup lang="ts">
import Menu from 'primevue/menu'
import { ref, type Component } from 'vue'

export interface ActionMenuItem {
  label: string
  icon?: Component
  command?: () => void
  class?: string
}

const props = defineProps<{
  items: ActionMenuItem[]
}>()

const menuRef = ref<InstanceType<typeof Menu> | null>(null)

const toggle = (event: Event) => {
  menuRef.value?.toggle(event)
}

defineExpose({
  toggle
})
</script>

<template>
  <Menu
    ref="menuRef"
    :model="props.items"
    popup
    class="min-w-44 bg-(--bg-card) border border-(--border) p-1 rounded-lg shadow-lg"
  >
    <template #item="{ item, props: menuProps }">
      <button
        v-bind="menuProps.action"
        class="flex items-center gap-2 px-3 py-2 w-full text-left text-xs text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--bg-hover)/20 rounded-md transition-colors cursor-pointer select-none"
        :class="[item.class]"
        @click="item.command?.({ originalEvent: $event, item })"
      >
        <component v-if="item.icon" :is="item.icon" class="w-4 h-4 shrink-0" />
        <span>{{ item.label }}</span>
      </button>
    </template>
  </Menu>
</template>
