<script setup lang="ts">
import { ref } from 'vue'
import WorkloadMetricsCards from '../components/workloads/WorkloadMetricsCards.vue'
import WorkloadsDataTable from '../components/workloads/WorkloadsDataTable.vue'

const activeTab = ref('deployments')
</script>

<template>
  <div class="flex flex-col gap-6">
    <Tabs v-model:value="activeTab">
      <TabList class="border-b border-(--border)">
        <Tab value="deployments" class="px-5 py-3 text-sm font-semibold">Deployments</Tab>
        <Tab value="statefulsets" class="px-5 py-3 text-sm font-semibold">StatefulSets</Tab>
        <Tab value="daemonsets" class="px-5 py-3 text-sm font-semibold">DaemonSets</Tab>
        <Tab value="jobs" class="px-5 py-3 text-sm font-semibold">Jobs</Tab>
        <Tab value="cronjobs" class="px-5 py-3 text-sm font-semibold">CronJobs</Tab>
      </TabList>

      <TabPanels class="pt-6">
        <!-- Deployments Tab -->
        <TabPanel value="deployments">
          <div class="flex flex-col gap-6">
            <WorkloadMetricsCards />
            <WorkloadsDataTable />
          </div>
        </TabPanel>

        <!-- StatefulSets Tab -->
        <TabPanel value="statefulsets">
          <div
            class="bg-(--bg-card) border border-(--border) rounded-xl p-12 text-center flex flex-col items-center justify-center gap-3"
          >
            <div
              class="w-12 h-12 rounded-xl bg-cyan-500/10 flex items-center justify-center text-cyan-400"
            >
              <span class="pi pi-database text-xl"></span>
            </div>
            <div class="text-sm font-semibold text-(--text-primary)">StatefulSets Management</div>
            <div class="text-xs text-(--text-muted) max-w-sm">
              View and manage stateful workloads with sticky identities. Detailed StatefulSet views
              are coming in the next release.
            </div>
          </div>
        </TabPanel>

        <!-- DaemonSets Tab -->
        <TabPanel value="daemonsets">
          <div
            class="bg-(--bg-card) border border-(--border) rounded-xl p-12 text-center flex flex-col items-center justify-center gap-3"
          >
            <div
              class="w-12 h-12 rounded-xl bg-orange-500/10 flex items-center justify-center text-orange-400"
            >
              <span class="pi pi-server text-xl"></span>
            </div>
            <div class="text-sm font-semibold text-(--text-primary)">DaemonSets Management</div>
            <div class="text-xs text-(--text-muted) max-w-sm">
              Monitor daemon processes running on every node. Detailed DaemonSet views are coming in
              the next release.
            </div>
          </div>
        </TabPanel>

        <!-- Jobs Tab -->
        <TabPanel value="jobs">
          <div
            class="bg-(--bg-card) border border-(--border) rounded-xl p-12 text-center flex flex-col items-center justify-center gap-3"
          >
            <div
              class="w-12 h-12 rounded-xl bg-blue-500/10 flex items-center justify-center text-blue-400"
            >
              <span class="pi pi-play text-xl"></span>
            </div>
            <div class="text-sm font-semibold text-(--text-primary)">Jobs Management</div>
            <div class="text-xs text-(--text-muted) max-w-sm">
              Track run-to-completion batch jobs. Detailed Job views are coming in the next release.
            </div>
          </div>
        </TabPanel>

        <!-- CronJobs Tab -->
        <TabPanel value="cronjobs">
          <div
            class="bg-(--bg-card) border border-(--border) rounded-xl p-12 text-center flex flex-col items-center justify-center gap-3"
          >
            <div
              class="w-12 h-12 rounded-xl bg-purple-500/10 flex items-center justify-center text-purple-400"
            >
              <span class="pi pi-clock text-xl"></span>
            </div>
            <div class="text-sm font-semibold text-(--text-primary)">CronJobs Management</div>
            <div class="text-xs text-(--text-muted) max-w-sm">
              Schedule time-based batch runs. Detailed CronJob scheduler views are coming in the
              next release.
            </div>
          </div>
        </TabPanel>
      </TabPanels>
    </Tabs>
  </div>
</template>
