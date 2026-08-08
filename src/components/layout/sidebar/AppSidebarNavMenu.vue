<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { PanelMenu } from 'primevue'
import type { MenuItem } from 'primevue/menuitem'
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import {
  categories,
  categoryNavItems,
  type AppSidebarMenuItem,
  type CategoryId
} from './navigation'

const props = defineProps<{
  activeTab: CategoryId | null
}>()

const expandedKeys = defineModel<Record<string, boolean>>('expandedKeys', { required: true })

const k8sStore = useKubernetesStore()
const route = useRoute()

const currentCategoryPanelItems = computed<AppSidebarMenuItem[]>(() => {
  if (!props.activeTab || props.activeTab === 'clusters') return []
  const cat = categories.find((c) => c.id === props.activeTab)
  if (!cat) return []
  const children = categoryNavItems[cat.id as Exclude<CategoryId, 'clusters'>] || []
  return [
    {
      key: cat.id,
      label: cat.name,
      customIcon: cat.icon,
      items: children
    }
  ]
})

const isLinkActive = (linkPath: string) => {
  if (linkPath.includes('?')) {
    const [path, queryStr] = linkPath.split('?')
    if (route.path !== path || !queryStr) return false
    const [key, val] = queryStr.split('=')
    if (!key || !val) return false
    return route.query[key] === val
  }
  return route.path === linkPath
}
</script>

<template>
  <nav class="flex-1 overflow-y-auto">
    <PanelMenu
      v-if="k8sStore.activeClusterId !== null"
      :model="currentCategoryPanelItems as unknown as MenuItem[]"
      v-model:expandedKeys="expandedKeys"
      multiple
      class="w-full border-none bg-transparent"
      :pt="{
        panel: { class: 'p-0 border-none bg-transparent mb-1' },
        header: { class: 'hover:border-none border-none bg-transparent p-0' },
        headercontent: { class: 'border-none bg-transparent p-0' },
        menucontent: { class: 'border-none bg-transparent p-0 pl-2' },
        root: { class: 'p-0 border-none bg-transparent' }
      }"
    >
      <template #item="{ item, active }">
        <!-- Category Header Group (item with sub-items) -->
        <div
          v-if="item.items && item.items.length > 0"
          class="flex items-center justify-between px-3 py-1.5 text-xs font-bold uppercase tracking-wider text-muted-color hover:text-color hover:bg-surface-100 dark:hover:bg-surface-800/60 cursor-pointer transition-colors select-none my-0.5"
        >
          <div class="flex items-center gap-2 min-w-0">
            <component
              :is="(item as AppSidebarMenuItem).customIcon"
              class="w-4 h-4 shrink-0 text-muted-color"
            />
            <span class="truncate">{{ item.label }}</span>
          </div>
          <span
            :class="[
              'pi text-xs transition-transform duration-200 shrink-0 ml-1',
              active ? 'pi-chevron-down' : 'pi-chevron-right'
            ]"
          ></span>
        </div>

        <!-- Leaf Menu Link (child item with route) -->
        <router-link v-else-if="item.route" v-slot="{ href, navigate }" :to="item.route" custom>
          <a
            :href="href"
            :class="[
              isLinkActive(item.route as string) ? 'border-r-3' : 'text-muted-color font-medium!',
              'flex items-center gap-2.5 px-3 py-1.5 transition-colors text-sm cursor-pointer no-underline select-none my-0.5'
            ]"
            @click="navigate"
          >
            <component :is="(item as AppSidebarMenuItem).customIcon" class="w-4 h-4 shrink-0" />
            <span class="truncate">{{ item.label }}</span>
          </a>
        </router-link>
      </template>
    </PanelMenu>
    <p v-else class="text-xs text-muted-color p-2">Select or add a cluster to view resources.</p>
  </nav>
</template>
