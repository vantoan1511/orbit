<script setup lang="ts">
import NamespaceBadge from '@/components/shared/NamespaceBadge.vue'
import NamespaceFilter from '@/components/shared/NamespaceFilter.vue'
import ResourceDataTable from '@/components/shared/ResourceDataTable.vue'
import SystemNamespaceToggle from '@/components/shared/SystemNamespaceToggle.vue'
import { useResourceFilters } from '@/composables/useResourceFilters'
import { useTableColumns } from '@/composables/useTableColumns'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { CronJobInfo } from '@/types/kubernetes'
import Column from 'primevue/column'
import Select from 'primevue/select'
import { computed, onMounted, ref, watch } from 'vue'
import WorkloadDetailsDrawer from './WorkloadDetailsDrawer.vue'

const k8sStore = useKubernetesStore()

const { tableColumns, visibleCols } = useTableColumns([
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'schedule', header: 'Schedule', visible: true },
  { field: 'suspend', header: 'Suspend', visible: true },
  { field: 'active', header: 'Active Jobs', visible: true },
  { field: 'lastSchedule', header: 'Last Schedule', visible: true },
  { field: 'age', header: 'Age', visible: true }
])

const { searchQuery, selectedNamespace, showSystemNamespaces, filteredResources } =
  useResourceFilters(
    computed(() => k8sStore.cronJobs),
    ['name', 'images']
  )

const selectedSuspend = ref('All Suspend States')
const loading = ref(false)

// Drawer state
const drawerVisible = ref(false)
const selectedWorkload = ref<CronJobInfo | null>(null)

const namespaces = computed(() => {
  return k8sStore.namespaces
})

const suspendOptions = ['All Suspend States', 'Suspended', 'Active']

const fetchCronJobs = async () => {
  loading.value = true
  try {
    const ns = selectedNamespace.value === 'All Namespaces' ? undefined : selectedNamespace.value
    await kubernetesService.getCronJobs(ns)
  } catch (e) {
    console.error('Error fetching cronjobs', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchCronJobs()
})

watch(selectedNamespace, () => {
  fetchCronJobs()
})

watch(
  () => k8sStore.activeClusterId,
  () => {
    fetchCronJobs()
  }
)

const filteredCronJobs = computed(() => {
  return filteredResources.value.filter((cj) => {
    if (selectedSuspend.value !== 'All Suspend States') {
      const isSuspended = selectedSuspend.value === 'Suspended'
      if (cj.suspend !== isSuspended) return false
    }
    return true
  })
})

const onRowClick = (event: { data: CronJobInfo }) => {
  selectedWorkload.value = event.data
  drawerVisible.value = true
}
</script>

<template>
  <ResourceDataTable
    :data="filteredCronJobs"
    v-model:searchQuery="searchQuery"
    v-model:columns="tableColumns"
    searchPlaceholder="Search cronjobs or images..."
    emptyMessage="No cronjobs found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} cronjobs"
    :loading="loading"
    @refresh="fetchCronJobs"
    @row-click="onRowClick"
  >
    <!-- Filters -->
    <template #filters>
      <!-- Namespace Select -->
      <NamespaceFilter v-model="selectedNamespace" :namespaces="namespaces" />

      <!-- Suspend Select -->
      <Select
        v-model="selectedSuspend"
        :options="suspendOptions"
        class="text-xs min-w-44 bg-(--bg-hover)/30 border-(--border)"
      />
    </template>

    <!-- Actions Left -->
    <template #actions-left>
      <SystemNamespaceToggle v-model="showSystemNamespaces" />
    </template>

    <!-- Columns -->
    <!-- Name Column -->
    <Column field="name" header="Name" sortable class="font-medium p-3 text-(--text-primary)">
      <template #body="{ data }">
        <span class="font-semibold hover:text-violet-400 transition-colors">{{ data.name }}</span>
      </template>
    </Column>

    <!-- Namespace Column -->
    <Column
      v-if="visibleCols['namespace']"
      field="namespace"
      header="Namespace"
      sortable
      class="p-3"
    >
      <template #body="{ data }">
        <NamespaceBadge :namespace="data.namespace" />
      </template>
    </Column>

    <!-- Schedule Column -->
    <Column
      v-if="visibleCols['schedule']"
      field="schedule"
      header="Schedule"
      sortable
      class="p-3 font-mono text-(--text-primary)"
    ></Column>

    <!-- Suspend Column -->
    <Column v-if="visibleCols['suspend']" field="suspend" header="Suspend" sortable class="p-3">
      <template #body="{ data }">
        <span
          class="font-medium px-2 py-0.5 rounded text-[10px] uppercase tracking-wider"
          :class="
            data.suspend
              ? 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
              : 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
          "
        >
          {{ data.suspend ? 'True' : 'False' }}
        </span>
      </template>
    </Column>

    <!-- Active Column -->
    <Column
      v-if="visibleCols['active']"
      field="active"
      header="Active Jobs"
      sortable
      class="p-3 text-center"
    >
      <template #body="{ data }">
        <span class="font-mono text-(--text-secondary)">{{ data.active }}</span>
      </template>
    </Column>

    <!-- Last Schedule Column -->
    <Column
      v-if="visibleCols['lastSchedule']"
      field="lastSchedule"
      header="Last Schedule"
      sortable
      class="p-3 font-mono text-(--text-secondary)"
    >
      <template #body="{ data }">
        <span>{{ data.lastSchedule || '-' }}</span>
      </template>
    </Column>

    <!-- Age Column -->
    <Column
      v-if="visibleCols['age']"
      field="age"
      header="Age"
      sortable
      class="p-3 text-(--text-muted) font-mono"
    ></Column>

    <!-- Drawer -->
    <template #drawer>
      <WorkloadDetailsDrawer v-model:visible="drawerVisible" :workload="selectedWorkload" />
    </template>
  </ResourceDataTable>
</template>
