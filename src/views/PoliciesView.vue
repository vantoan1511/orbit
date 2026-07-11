<script setup lang="ts">
import { ref } from 'vue'
import Tabs from 'primevue/tabs'
import TabList from 'primevue/tablist'
import Tab from 'primevue/tab'
import TabPanels from 'primevue/tabpanels'
import TabPanel from 'primevue/tabpanel'

import PoliciesMetricsCards from '../components/policies/PoliciesMetricsCards.vue'
import PoliciesDataTable from '../components/policies/PoliciesDataTable.vue'

const activeTab = ref('overview')
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Header/Title -->
    <div class="flex items-center justify-between mb-6">
      <h2 class="text-xl font-bold tracking-tight text-(--text-primary)">Policies</h2>
    </div>

    <!-- Tabs Navigation -->
    <Tabs v-model:value="activeTab" class="flex-1 flex flex-col min-h-0">
      <TabList class="mb-6">
        <Tab value="overview" class="text-sm font-medium">Overview</Tab>
        <Tab value="network" class="text-sm font-medium">Network Policies</Tab>
        <Tab value="pod-security" class="text-sm font-medium">Pod Security</Tab>
        <Tab value="resource" class="text-sm font-medium">Resource Policies</Tab>
        <Tab value="rbac" class="text-sm font-medium">RBAC</Tab>
        <Tab value="admission" class="text-sm font-medium">Admission Policies</Tab>
      </TabList>

      <TabPanels class="flex-1 min-h-0 p-0 bg-transparent border-none">
        <!-- Overview Tab -->
        <TabPanel value="overview" class="flex flex-col gap-6">
          <PoliciesMetricsCards />
          <PoliciesDataTable />
        </TabPanel>

        <!-- Other Tabs (Placeholder logic) -->
        <TabPanel value="network">
          <div
            class="text-(--text-muted) flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            Network Policies specific view coming soon.
          </div>
        </TabPanel>

        <TabPanel value="pod-security">
          <div
            class="text-(--text-muted) flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            Pod Security specific view coming soon.
          </div>
        </TabPanel>

        <TabPanel value="resource">
          <div
            class="text-(--text-muted) flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            Resource Policies specific view coming soon.
          </div>
        </TabPanel>

        <TabPanel value="rbac">
          <div
            class="text-(--text-muted) flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            RBAC specific view coming soon.
          </div>
        </TabPanel>

        <TabPanel value="admission">
          <div
            class="text-(--text-muted) flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            Admission Policies specific view coming soon.
          </div>
        </TabPanel>
      </TabPanels>
    </Tabs>
  </div>
</template>
