<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import ViewLayout from '@/components/shared/ViewLayout.vue'
import ServicesTable from '../components/network/ServicesTable.vue'
import IngressesTable from '../components/network/IngressesTable.vue'

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
      <TabList class="border-b border-(--border)">
        <Tab value="services" class="px-5 py-3 text-sm font-semibold">Services</Tab>
        <Tab value="ingresses" class="px-5 py-3 text-sm font-semibold">Ingresses</Tab>
      </TabList>

      <TabPanels class="pt-6">
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
