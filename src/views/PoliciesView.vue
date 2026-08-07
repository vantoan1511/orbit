<script setup lang="ts">
import { ref } from 'vue'
import ViewLayout from '@/components/shared/ViewLayout.vue'
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
  <ViewLayout title="Policies">
    <!-- Tabs Navigation -->
    <Tabs v-model:value="activeTab" class="flex-1 flex flex-col min-h-0">
      <TabList class="border-b border-(--border)">
        <Tab value="overview" class="px-5 py-3 text-sm font-semibold">Overview</Tab>
        <Tab value="network" class="px-5 py-3 text-sm font-semibold">Network Policies</Tab>
        <Tab value="pod-security" class="px-5 py-3 text-sm font-semibold">Pod Security</Tab>
        <Tab value="resource" class="px-5 py-3 text-sm font-semibold">Resource Policies</Tab>
        <Tab value="rbac" class="px-5 py-3 text-sm font-semibold">RBAC</Tab>
        <Tab value="admission" class="px-5 py-3 text-sm font-semibold">Admission Policies</Tab>
      </TabList>

      <TabPanels class="pt-6">
        <!-- Overview Tab -->
        <TabPanel value="overview" class="flex flex-col gap-6">
          <PoliciesMetricsCards />
          <PoliciesDataTable />
        </TabPanel>

        <!-- Other Tabs (Placeholder logic) -->
        <TabPanel value="network">
          <div
            class="text-muted-color flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            Network Policies specific view coming soon.
          </div>
        </TabPanel>

        <TabPanel value="pod-security">
          <div
            class="text-muted-color flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            Pod Security specific view coming soon.
          </div>
        </TabPanel>

        <TabPanel value="resource">
          <div
            class="text-muted-color flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            Resource Policies specific view coming soon.
          </div>
        </TabPanel>

        <TabPanel value="rbac">
          <div
            class="text-muted-color flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            RBAC specific view coming soon.
          </div>
        </TabPanel>

        <TabPanel value="admission">
          <div
            class="text-muted-color flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            Admission Policies specific view coming soon.
          </div>
        </TabPanel>
      </TabPanels>
    </Tabs>
  </ViewLayout>
</template>

