<script setup lang="ts">
import {
  categoryNavItems,
  type AppSidebarMenuItem,
  type CategoryId
} from '@/components/layout/sidebar/navigation'
import InputText from 'primevue/inputtext'
import Popover from 'primevue/popover'
import { computed, ref } from 'vue'

const emit = defineEmits<{
  (
    e: 'select-resource',
    payload: { route: string; title: string; iconName?: string; category: string }
  ): void
}>()

const popoverRef = ref<InstanceType<typeof Popover> | null>(null)
const searchFilter = ref('')

interface FlatResourceItem {
  key: string
  label: string
  route: string
  category: CategoryId
  categoryLabel: string
  customIcon: unknown
}

const CATEGORY_NAMES: Record<CategoryId, string> = {
  clusters: 'Clusters',
  core: 'Core & Overview',
  workloads: 'Workloads',
  network: 'Network',
  storage: 'Storage',
  config: 'Config & Secrets',
  security: 'Security',
  logs: 'Logs'
}

const allResources = computed<FlatResourceItem[]>(() => {
  const list: FlatResourceItem[] = []
  for (const [catKey, items] of Object.entries(categoryNavItems)) {
    const category = catKey as CategoryId
    for (const item of items as AppSidebarMenuItem[]) {
      if (item.route && item.label) {
        list.push({
          key: item.key || item.route,
          label: item.label,
          route: item.route,
          category,
          categoryLabel: CATEGORY_NAMES[category] || category,
          customIcon: item.customIcon
        })
      }
    }
  }
  return list
})

const filteredResources = computed<FlatResourceItem[]>(() => {
  const query = searchFilter.value.trim().toLowerCase()
  if (!query) return allResources.value
  return allResources.value.filter(
    (item) =>
      item.label.toLowerCase().includes(query) ||
      item.categoryLabel.toLowerCase().includes(query) ||
      item.route.toLowerCase().includes(query)
  )
})

const handleSelect = (item: FlatResourceItem) => {
  emit('select-resource', {
    route: item.route,
    title: item.label,
    iconName: item.key,
    category: item.category
  })
  popoverRef.value?.hide()
}

const toggle = (event: Event) => {
  searchFilter.value = ''
  popoverRef.value?.toggle(event)
}

const show = (event: Event) => {
  searchFilter.value = ''
  popoverRef.value?.show(event)
}

const hide = () => {
  popoverRef.value?.hide()
}

defineExpose({
  toggle,
  show,
  hide
})
</script>

<template>
  <Popover ref="popoverRef" class="w-72 shadow-xl border border-(--border)" @click.stop>
    <div class="flex flex-col gap-2 p-1 select-none">
      <div class="px-2 pt-1 pb-1">
        <span class="text-xs font-semibold text-muted-color uppercase tracking-wider">
          Open Resource Tab
        </span>
      </div>

      <div class="px-1">
        <InputText
          v-model="searchFilter"
          placeholder="Filter resources..."
          size="small"
          class="w-full text-xs"
          autofocus
        />
      </div>

      <div class="max-h-64 overflow-y-auto flex flex-col py-1 gap-0.5">
        <div
          v-for="item in filteredResources"
          :key="item.key"
          class="flex items-center justify-between px-2.5 py-1.5 rounded-md hover:bg-(--bg-hover) text-xs cursor-pointer transition-colors group"
          @click="handleSelect(item)"
        >
          <div class="flex items-center gap-2 min-w-0">
            <component
              :is="item.customIcon"
              class="w-3.5 h-3.5 shrink-0 text-muted-color group-hover:text-primary transition-colors"
            />
            <span class="truncate font-medium text-primary">{{ item.label }}</span>
          </div>
          <span class="text-[10px] text-muted-color shrink-0 ml-2">
            {{ item.categoryLabel }}
          </span>
        </div>

        <div
          v-if="filteredResources.length === 0"
          class="py-4 text-center text-xs text-muted-color"
        >
          No matching resources found
        </div>
      </div>
    </div>
  </Popover>
</template>
