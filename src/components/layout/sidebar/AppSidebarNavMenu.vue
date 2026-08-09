<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { categoryNavItems, type AppSidebarMenuItem, type CategoryId } from './navigation'

const props = defineProps<{
  activeTab: CategoryId | null
}>()

const k8sStore = useKubernetesStore()
const route = useRoute()

const flatItems = computed<AppSidebarMenuItem[]>(() => {
  if (!props.activeTab || props.activeTab === 'clusters') return []
  return categoryNavItems[props.activeTab as Exclude<CategoryId, 'clusters'>] || []
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
    <div v-if="k8sStore.activeClusterId !== null" class="flex flex-col py-1">
      <router-link
        v-for="item in flatItems"
        :key="item.key"
        v-slot="{ href, navigate }"
        :to="item.route!"
        custom
      >
        <a
          :href="href"
          :class="[
            isLinkActive(item.route as string)
              ? 'border-l-3 rounded-none!'
              : 'text-muted-color font-medium!',
            'flex items-center gap-2.5 px-3 py-1.5 transition-colors text-sm cursor-pointer no-underline select-none my-0.5'
          ]"
          @click="navigate"
        >
          <component :is="item.customIcon" class="w-4 h-4 shrink-0" />
          <span class="truncate">{{ item.label }}</span>
        </a>
      </router-link>
    </div>
    <p v-else class="text-xs text-muted-color p-2">Select or add a cluster to view resources.</p>
  </nav>
</template>
