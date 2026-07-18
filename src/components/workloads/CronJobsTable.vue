<script setup lang="ts">
import ResourceDataTable, { type TableColumn } from '@/components/shared/ResourceDataTable.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { CronJobInfo } from '@/types/kubernetes'
import Column from 'primevue/column'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import { computed, onMounted, ref, watch } from 'vue'
import WorkloadDetailsDrawer from './WorkloadDetailsDrawer.vue'

const k8sStore = useKubernetesStore()

const tableColumns = ref<TableColumn[]>([
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'schedule', header: 'Schedule', visible: true },
  { field: 'suspend', header: 'Suspend', visible: true },
  { field: 'active', header: 'Active Jobs', visible: true },
  { field: 'lastSchedule', header: 'Last Schedule', visible: true },
  { field: 'age', header: 'Age', visible: true }
])

const visibleCols = computed(() => {
  return tableColumns.value.reduce(
    (acc, col) => {
      acc[col.field] = col.visible
      return acc
    },
    {} as Record<string, boolean>
  )
})

const searchQuery = ref('')
const selectedNamespace = ref('All Namespaces')
const selectedSuspend = ref('All Suspend States')
const showSystemNamespaces = ref(false)
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
  return k8sStore.cronJobs.filter((cj) => {
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      const matchesName = cj.name.toLowerCase().includes(query)
      const matchesImage = cj.images?.some((img) => img.toLowerCase().includes(query))
      if (!matchesName && !matchesImage) return false
    }

    const isSystem = ['kube-system', 'monitoring', 'logging'].includes(cj.namespace)
    if (!showSystemNamespaces.value && isSystem && selectedNamespace.value === 'All Namespaces') {
      return false
    }

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
      <Select
        v-model="selectedNamespace"
        :options="namespaces"
        class="text-xs min-w-44 bg-(--bg-hover)/30 border-(--border)"
      />

      <!-- Suspend Select -->
      <Select
        v-model="selectedSuspend"
        :options="suspendOptions"
        class="text-xs min-w-44 bg-(--bg-hover)/30 border-(--border)"
      />
    </template>

    <!-- Actions Left -->
    <template #actions-left>
      <div class="flex items-center gap-2">
        <ToggleSwitch v-model="showSystemNamespaces" inputId="system-ns-toggle" />
        <label
          for="system-ns-toggle"
          class="text-xs font-semibold text-(--text-secondary) cursor-pointer select-none"
        >
          Show System
        </label>
      </div>
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
        <span class="font-mono text-(--text-muted)">{{ data.namespace }}</span>
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
