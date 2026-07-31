<script setup lang="ts">
import { ref, watch } from 'vue'
import WorkloadMetricsCards from '../components/workloads/WorkloadMetricsCards.vue'
import DeploymentsTable from '../components/workloads/DeploymentsTable.vue'
import StatefulSetsTable from '../components/workloads/StatefulSetsTable.vue'
import DaemonSetsTable from '../components/workloads/DaemonSetsTable.vue'
import ReplicaSetsTable from '../components/workloads/ReplicaSetsTable.vue'
import JobsTable from '../components/workloads/JobsTable.vue'
import CronJobsTable from '../components/workloads/CronJobsTable.vue'

const activeTab = ref('deployments')
const visitedTabs = ref(new Set(['deployments']))

watch(activeTab, (newTab) => {
  visitedTabs.value.add(newTab)
})
</script>

<template>
  <div class="flex flex-col gap-6">
    <Tabs v-model:value="activeTab">
      <TabList class="border-b border-(--border)">
        <Tab value="deployments" class="px-5 py-3 text-sm font-semibold">Deployments</Tab>
        <Tab value="statefulsets" class="px-5 py-3 text-sm font-semibold">StatefulSets</Tab>
        <Tab value="daemonsets" class="px-5 py-3 text-sm font-semibold">DaemonSets</Tab>
        <Tab value="replicasets" class="px-5 py-3 text-sm font-semibold">ReplicaSets</Tab>
        <Tab value="jobs" class="px-5 py-3 text-sm font-semibold">Jobs</Tab>
        <Tab value="cronjobs" class="px-5 py-3 text-sm font-semibold">CronJobs</Tab>
      </TabList>

      <TabPanels class="pt-6">
        <!-- Deployments Tab -->
        <TabPanel value="deployments">
          <div v-if="visitedTabs.has('deployments')" class="flex flex-col gap-6">
            <WorkloadMetricsCards />
            <DeploymentsTable />
          </div>
        </TabPanel>

        <!-- StatefulSets Tab -->
        <TabPanel value="statefulsets">
          <StatefulSetsTable v-if="visitedTabs.has('statefulsets')" />
        </TabPanel>

        <!-- DaemonSets Tab -->
        <TabPanel value="daemonsets">
          <DaemonSetsTable v-if="visitedTabs.has('daemonsets')" />
        </TabPanel>

        <!-- ReplicaSets Tab -->
        <TabPanel value="replicasets">
          <ReplicaSetsTable v-if="visitedTabs.has('replicasets')" />
        </TabPanel>

        <!-- Jobs Tab -->
        <TabPanel value="jobs">
          <JobsTable v-if="visitedTabs.has('jobs')" />
        </TabPanel>

        <!-- CronJobs Tab -->
        <TabPanel value="cronjobs">
          <CronJobsTable v-if="visitedTabs.has('cronjobs')" />
        </TabPanel>
      </TabPanels>
    </Tabs>
  </div>
</template>
