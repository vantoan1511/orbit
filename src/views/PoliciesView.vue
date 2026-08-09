<script setup lang="ts">
import ViewLayout from '@/components/shared/ViewLayout.vue'
import TabPanel from 'primevue/tabpanel'
import TabPanels from 'primevue/tabpanels'
import Tabs from 'primevue/tabs'
import { ref } from 'vue'

import PoliciesDataTable from '../components/policies/PoliciesDataTable.vue'
import PoliciesMetricsCards from '../components/policies/PoliciesMetricsCards.vue'

const activeTab = ref('overview')
</script>

<template>
  <ViewLayout title="Policies">
    <!-- Tabs Navigation -->
    <Tabs v-model:value="activeTab" class="flex-1 flex flex-col min-h-0">
      <TabPanels>
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
