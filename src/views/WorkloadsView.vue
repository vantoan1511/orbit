<script setup lang="ts">
import ViewLayout from '@/components/shared/ViewLayout.vue'
import { ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import CronJobsTable from '../components/workloads/CronJobsTable.vue'
import DaemonSetsTable from '../components/workloads/DaemonSetsTable.vue'
import DeploymentsTable from '../components/workloads/DeploymentsTable.vue'
import JobsTable from '../components/workloads/JobsTable.vue'
import ReplicaSetsTable from '../components/workloads/ReplicaSetsTable.vue'
import StatefulSetsTable from '../components/workloads/StatefulSetsTable.vue'
import WorkloadMetricsCards from '../components/workloads/WorkloadMetricsCards.vue'

const route = useRoute()
const activeTab = ref((route.query.tab as string) || 'deployments')
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
  <ViewLayout title="Workloads">
    <Tabs v-model:value="activeTab">
      <TabPanels>
        <!-- Overview Tab -->
        <TabPanel value="overview">
          <WorkloadMetricsCards v-if="visitedTabs.has('overview')" />
        </TabPanel>

        <!-- Deployments Tab -->
        <TabPanel value="deployments">
          <DeploymentsTable v-if="visitedTabs.has('deployments')" />
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
  </ViewLayout>
</template>
