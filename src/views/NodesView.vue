<script setup lang="ts">
import { ref } from 'vue'
import NodeMetricsCards from '../components/nodes/NodeMetricsCards.vue'
import NodesTable from '../components/nodes/NodesTable.vue'
import WorkloadsTable from '../components/nodes/WorkloadsTable.vue'

const activeTab = ref('0')
</script>

<template>
  <div class="flex flex-col gap-6">
    <Tabs v-model:value="activeTab">
      <TabList class="border-b border-(--border)">
        <Tab value="0" class="px-5 py-3 text-sm font-semibold">Nodes</Tab>
        <Tab value="1" class="px-5 py-3 text-sm font-semibold">Workloads</Tab>
      </TabList>
      <TabPanels class="pt-6">
        <TabPanel value="0">
          <div class="flex flex-col gap-6">
            <NodeMetricsCards />
            <NodesTable />
            <WorkloadsTable />
          </div>
        </TabPanel>
        <TabPanel value="1">
          <div
            class="bg-(--bg-card) border border-(--border) rounded-xl p-8 text-center text-(--text-muted)"
          >
            Detailed workload management is available under the main Workloads menu.
          </div>
        </TabPanel>
      </TabPanels>
    </Tabs>
  </div>
</template>
