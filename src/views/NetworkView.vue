<script setup lang="ts">
import ViewLayout from '@/components/shared/ViewLayout.vue'
import { ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import IngressesTable from '../components/network/IngressesTable.vue'
import ServicesTable from '../components/network/ServicesTable.vue'

const route = useRoute()
const activeTab = ref((route.query.tab as string) || 'services')
const visitedTabs = ref(new Set([activeTab.value]))

watch(
  () => route.query.tab,
  (newTab) => {
    if (newTab && typeof newTab === 'string') {
      activeTab.value = newTab
      visitedTabs.value.add(newTab)
    }
  },
  { immediate: true }
)

watch(activeTab, (newTab) => {
  visitedTabs.value.add(newTab)
})
</script>

<template>
  <ViewLayout title="Network">
    <Tabs v-model:value="activeTab">
      <TabPanels>
        <!-- Services Tab -->
        <TabPanel value="services">
          <ServicesTable v-if="visitedTabs.has('services')" />
        </TabPanel>

        <!-- Ingresses Tab -->
        <TabPanel value="ingresses">
          <IngressesTable v-if="visitedTabs.has('ingresses')" />
        </TabPanel>
      </TabPanels>
    </Tabs>
  </ViewLayout>
</template>
