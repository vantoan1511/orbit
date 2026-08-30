<script setup lang="ts">
import ViewLayout from '@/components/shared/ViewLayout.vue'
import Button from 'primevue/button'
import TabPanel from 'primevue/tabpanel'
import TabPanels from 'primevue/tabpanels'
import Tabs from 'primevue/tabs'
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

export interface ResourceTab {
  id: string
  createAction?: {
    label?: string
    icon?: string
    handler: () => void
  }
}

const props = defineProps<{
  title?: string
  defaultTab: string
  tabs: ResourceTab[]
}>()

const route = useRoute()

const activeTab = ref((route.query.tab as string) || props.defaultTab)
const visitedTabs = ref(new Set([activeTab.value]))

watch(
  () => route.query.tab,
  (newTab) => {
    if (newTab && typeof newTab === 'string' && props.tabs.some((t) => t.id === newTab)) {
      activeTab.value = newTab
      visitedTabs.value.add(newTab)
    }
  },
  { immediate: true }
)

watch(activeTab, (newTab) => {
  visitedTabs.value.add(newTab)
})

const currentTabConfig = computed(() => props.tabs.find((t) => t.id === activeTab.value))
</script>

<template>
  <ViewLayout :title="title">
    <template v-if="$slots['leading-actions']" #leading-actions>
      <slot name="leading-actions" />
    </template>

    <template v-if="currentTabConfig?.createAction || $slots.actions" #actions>
      <slot name="actions" :active-tab="activeTab">
        <Button
          v-if="currentTabConfig?.createAction"
          :label="currentTabConfig.createAction.label || 'Create'"
          :icon="currentTabConfig.createAction.icon || 'pi pi-plus'"
          size="small"
          @click="currentTabConfig.createAction.handler"
        />
      </slot>
    </template>

    <Tabs v-model:value="activeTab">
      <TabPanels>
        <TabPanel v-for="tab in tabs" :key="tab.id" :value="tab.id">
          <slot v-if="visitedTabs.has(tab.id)" :name="`tab-${tab.id}`" :active-tab="activeTab" />
        </TabPanel>
      </TabPanels>
    </Tabs>
  </ViewLayout>
</template>
