<script setup lang="ts">
import { ref, watch } from 'vue'
import ServicesTable from '../components/network/ServicesTable.vue'
import IngressesTable from '../components/network/IngressesTable.vue'

const activeTab = ref('services')
const visitedTabs = ref(new Set(['services']))

watch(activeTab, (newTab) => {
  visitedTabs.value.add(newTab)
})
</script>

<template>
  <div class="flex flex-col gap-6">
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
  </div>
</template>
