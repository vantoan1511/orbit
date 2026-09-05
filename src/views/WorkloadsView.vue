<script setup lang="ts">
import ResourceTabsLayout, { type ResourceTab } from '@/components/shared/ResourceTabsLayout.vue'
import { useDialog } from 'primevue/usedialog'
import CreateDeploymentDialog from '@/components/workloads/CreateDeploymentDialog.vue'
import CreatePodDialog from '@/components/pods/CreatePodDialog.vue'
import CronJobsTable from '@/components/workloads/CronJobsTable.vue'
import DaemonSetsTable from '@/components/workloads/DaemonSetsTable.vue'
import DeploymentsTable from '@/components/workloads/DeploymentsTable.vue'
import JobsTable from '@/components/workloads/JobsTable.vue'
import PodMetricsCards from '@/components/pods/PodMetricsCards.vue'
import PodsDataTable from '@/components/pods/PodsDataTable.vue'
import ReplicaSetsTable from '@/components/workloads/ReplicaSetsTable.vue'
import StatefulSetsTable from '@/components/workloads/StatefulSetsTable.vue'
import WorkloadMetricsCards from '@/components/workloads/WorkloadMetricsCards.vue'

const dialog = useDialog()

const openCreatePodDialog = () => {
  dialog.open(CreatePodDialog, {
    props: {
      header: 'Create Pod',
      style: {
        width: '460px'
      },
      modal: true
    }
  })
}

const openCreateDeploymentDialog = () => {
  dialog.open(CreateDeploymentDialog, {
    props: {
      header: 'Create Deployment',
      style: {
        width: '420px'
      },
      modal: true
    }
  })
}

const tabs: ResourceTab[] = [
  { id: 'overview' },
  {
    id: 'pods',
    createAction: { handler: openCreatePodDialog }
  },
  {
    id: 'deployments',
    createAction: { handler: openCreateDeploymentDialog }
  },
  { id: 'statefulsets' },
  { id: 'daemonsets' },
  { id: 'replicasets' },
  { id: 'jobs' },
  { id: 'cronjobs' }
]
</script>

<template>
  <ResourceTabsLayout title="Workloads" default-tab="deployments" :tabs="tabs">
    <!-- Overview Tab -->
    <template #tab-overview>
      <WorkloadMetricsCards />
    </template>

    <!-- Pods Tab -->
    <template #tab-pods>
      <div class="flex flex-col gap-6">
        <PodMetricsCards />
        <PodsDataTable />
      </div>
    </template>

    <!-- Deployments Tab -->
    <template #tab-deployments>
      <DeploymentsTable />
    </template>

    <!-- StatefulSets Tab -->
    <template #tab-statefulsets>
      <StatefulSetsTable />
    </template>

    <!-- DaemonSets Tab -->
    <template #tab-daemonsets>
      <DaemonSetsTable />
    </template>

    <!-- ReplicaSets Tab -->
    <template #tab-replicasets>
      <ReplicaSetsTable />
    </template>

    <!-- Jobs Tab -->
    <template #tab-jobs>
      <JobsTable />
    </template>

    <!-- CronJobs Tab -->
    <template #tab-cronjobs>
      <CronJobsTable />
    </template>
  </ResourceTabsLayout>
</template>
