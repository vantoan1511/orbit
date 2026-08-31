<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import TableFilterSelect from '@/components/shared/TableFilterSelect.vue'
import { KUBERNETES_NAMESPACE_STATUS, KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { NamespaceInfo } from '@/types/kubernetes'
import Chart from 'primevue/chart'
import Column from 'primevue/column'
import { computed, onMounted, ref } from 'vue'
import NamespaceDetailsDrawer from './NamespaceDetailsDrawer.vue'

interface MappedNamespaceInfo extends NamespaceInfo {
  pods: number
  podSparkline: number[]
  workloads: number
  services: number
  configMaps: number
  secrets: number
}

const store = useKubernetesStore()
const selectedLabel = ref('All Labels')

const columns = [
  { field: 'name', header: 'Name', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'pods', header: 'Pods', visible: true },
  { field: 'workloads', header: 'Workloads', visible: true },
  { field: 'services', header: 'Services', visible: true },
  { field: 'configMaps', header: 'ConfigMaps', visible: true },
  { field: 'secrets', header: 'Secrets', visible: true },
  { field: 'age', header: 'Age', visible: true },
  { field: 'labels', header: 'Labels', visible: true }
]

const statuses = ['All Statuses', ...Object.values(KUBERNETES_NAMESPACE_STATUS)]

// Collect unique label keys across all namespaces
const labelOptions = computed(() => {
  const keys = new Set<string>()
  store.namespaceList.forEach((ns) => {
    Object.keys(ns.labels).forEach((k) => keys.add(k))
  })
  return ['All Labels', ...Array.from(keys)]
})

// Dynamically map and compute resource counts for each namespace
const mappedNamespaces = computed(() => {
  return store.namespaceList.map((ns) => {
    const podsList = store.pods.filter((p) => p.namespace === ns.name)
    const podsCount = podsList.length

    const deploymentsCount = store.deployments.filter((d) => d.namespace === ns.name).length
    const statefulSetsCount = store.statefulSets.filter((s) => s.namespace === ns.name).length
    const daemonSetsCount = store.daemonSets.filter((d) => d.namespace === ns.name).length
    const replicaSetsCount = store.replicaSets.filter((r) => r.namespace === ns.name).length
    const jobsCount = store.jobs.filter((j) => j.namespace === ns.name).length
    const cronJobsCount = store.cronJobs.filter((c) => c.namespace === ns.name).length
    const workloadsCount =
      deploymentsCount +
      statefulSetsCount +
      daemonSetsCount +
      replicaSetsCount +
      jobsCount +
      cronJobsCount

    const servicesCount = store.services.filter((s) => s.namespace === ns.name).length
    const configMapsCount = store.configMaps.filter((c) => c.namespace === ns.name).length
    const secretsCount = store.secrets.filter((s) => s.namespace === ns.name).length

    const podSparkline = [
      podsCount,
      podsCount,
      podsCount,
      podsCount,
      podsCount,
      podsCount,
      podsCount
    ]

    return {
      ...ns,
      pods: podsCount,
      podSparkline,
      workloads: workloadsCount,
      services: servicesCount,
      configMaps: configMapsCount,
      secrets: secretsCount
    }
  })
})

const filteredNamespaces = computed(() => {
  return mappedNamespaces.value.filter((ns) => {
    if (selectedLabel.value !== 'All Labels') {
      if (!(selectedLabel.value in ns.labels)) return false
    }
    return true
  })
})

const handleRefresh = async () => {
  try {
    await kubernetesService.getNamespaces()
  } catch (error) {
    console.error('Error fetching namespaces:', error)
  }
}

// Sparkline chart options - minimal, no axes, no tooltips
const sparklineOptions = ref()

onMounted(() => {
  sparklineOptions.value = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: { display: false },
      tooltip: { enabled: false }
    },
    scales: {
      x: { display: false },
      y: { display: false }
    },
    elements: {
      point: { radius: 0, hoverRadius: 0 },
      line: { tension: 0.4, borderWidth: 1.5 }
    },
    events: []
  }
})

const getSparklineData = (ns: MappedNamespaceInfo) => ({
  labels: ['1', '2', '3', '4', '5', '6', '7'],
  datasets: [
    {
      data: ns.podSparkline,
      borderColor: ns.status === KUBERNETES_NAMESPACE_STATUS.Terminating ? '#f59e0b' : '#10b981',
      backgroundColor: 'transparent',
      fill: false
    }
  ]
})

const MAX_VISIBLE_LABELS = 2
</script>

<template>
  <GenericResourceTable
    :data="filteredNamespaces"
    :initialColumns="columns"
    :statuses="statuses"
    :hideNamespaceFilter="true"
    :hideNamespaceColumn="true"
    :kind="KUBERNETES_RESOURCE_KIND.Namespace"
    searchPlaceholder="Search namespaces..."
    emptyMessage="No namespaces found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} namespaces"
    :loading="store.namespacesLoading"
    @refresh="handleRefresh"
  >
    <!-- Extra Filter -->
    <template #filters>
      <TableFilterSelect v-model="selectedLabel" :options="labelOptions" />
    </template>

    <!-- Custom Name -->
    <template #name="{ data }">
      <div class="flex items-center gap-2">
        <span class="font-semibold">{{ data.name }}</span>
        <Tag v-if="data.isSystem" severity="info" value="System" />
      </div>
    </template>

    <template #default="{ visibleCols }">
      <!-- Pods Column with sparkline -->
      <Column v-if="visibleCols['pods']" field="pods" header="Pods" sortable class="p-3">
        <template #body="{ data }">
          <div class="flex items-center gap-3">
            <span class="font-mono text-primary min-w-6">{{ data.pods }}</span>
            <div class="w-16 h-6 shrink-0" v-if="sparklineOptions">
              <Chart
                type="line"
                :data="getSparklineData(data)"
                :options="sparklineOptions"
                class="w-full h-full"
              />
            </div>
          </div>
        </template>
      </Column>

      <!-- Workloads Column -->
      <Column
        v-if="visibleCols['workloads']"
        field="workloads"
        header="Workloads"
        sortable
        class="p-3"
        bodyClass="text-muted-color"
      >
        <template #body="{ data }">
          <span class="font-mono">{{ data.workloads }}</span>
        </template>
      </Column>

      <!-- Services Column -->
      <Column
        v-if="visibleCols['services']"
        field="services"
        header="Services"
        sortable
        class="p-3"
        bodyClass="text-muted-color"
      >
        <template #body="{ data }">
          <span class="font-mono">{{ data.services }}</span>
        </template>
      </Column>

      <!-- ConfigMaps Column -->
      <Column
        v-if="visibleCols['configMaps']"
        field="configMaps"
        header="ConfigMaps"
        sortable
        class="p-3"
        bodyClass="text-muted-color"
      >
        <template #body="{ data }">
          <span class="font-mono">{{ data.configMaps }}</span>
        </template>
      </Column>

      <!-- Secrets Column -->
      <Column
        v-if="visibleCols['secrets']"
        field="secrets"
        header="Secrets"
        sortable
        class="p-3"
        bodyClass="text-muted-color"
      >
        <template #body="{ data }">
          <span class="font-mono">{{ data.secrets }}</span>
        </template>
      </Column>

      <!-- Labels Column -->
      <Column v-if="visibleCols['labels']" field="labels" header="Labels" class="p-3">
        <template #body="{ data }">
          <div class="flex items-center gap-1 flex-wrap">
            <Tag
              v-for="(val, key) in Object.fromEntries(
                Object.entries(data.labels).slice(0, MAX_VISIBLE_LABELS)
              )"
              :key="key"
              severity="secondary"
              class="font-mono whitespace-nowrap"
              :value="`${key}: ${val}`"
            />
            <Tag
              v-if="Object.keys(data.labels).length > MAX_VISIBLE_LABELS"
              severity="secondary"
              :value="`+${Object.keys(data.labels).length - MAX_VISIBLE_LABELS}`"
            />
          </div>
        </template>
      </Column>
    </template>

    <!-- Drawer -->
    <template #drawer="{ selectedItem, visible, close }">
      <NamespaceDetailsDrawer
        :visible="visible"
        :namespace="selectedItem"
        @update:visible="!$event && close()"
      />
    </template>
  </GenericResourceTable>
</template>
