<script setup lang="ts">
import ResourceDataTable, { type TableColumn } from '@/components/shared/ResourceDataTable.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { DaemonSetInfo } from '@/types/kubernetes'
import Column from 'primevue/column'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import { computed, onMounted, ref, watch } from 'vue'
import WorkloadDetailsDrawer from './WorkloadDetailsDrawer.vue'

const k8sStore = useKubernetesStore()

const tableColumns = ref<TableColumn[]>([
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'desiredCurrent', header: 'Desired/Current', visible: true },
  { field: 'ready', header: 'Ready', visible: true },
  { field: 'available', header: 'Available', visible: true },
  { field: 'age', header: 'Age', visible: true },
  { field: 'images', header: 'Images', visible: true }
])

const visibleCols = computed(() =>
  Object.fromEntries(tableColumns.value.map((col) => [col.field, col.visible]))
)

const searchQuery = ref('')
const selectedNamespace = ref('All Namespaces')
const selectedStatus = ref('All Statuses')
const showSystemNamespaces = ref(false)
const loading = ref(false)

// Drawer state
const drawerVisible = ref(false)
const selectedWorkload = ref<DaemonSetInfo | null>(null)

const namespaces = computed(() => {
  return k8sStore.namespaces
})

const statuses = ['All Statuses', 'Running', 'Progressing']

const fetchDaemonSets = async () => {
  loading.value = true
  try {
    const ns = selectedNamespace.value === 'All Namespaces' ? undefined : selectedNamespace.value
    await kubernetesService.getDaemonSets(ns)
  } catch (e) {
    console.error('Error fetching daemonsets', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchDaemonSets()
})

watch(selectedNamespace, () => {
  fetchDaemonSets()
})

watch(
  () => k8sStore.activeClusterId,
  () => {
    fetchDaemonSets()
  }
)

const filteredDaemonSets = computed(() => {
  return k8sStore.daemonSets.filter((d) => {
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      const matchesName = d.name.toLowerCase().includes(query)
      const matchesImage = d.images?.some((img) => img.toLowerCase().includes(query))
      if (!matchesName && !matchesImage) return false
    }

    const isSystem = ['kube-system', 'monitoring', 'logging'].includes(d.namespace)
    if (!showSystemNamespaces.value && isSystem && selectedNamespace.value === 'All Namespaces') {
      return false
    }

    if (selectedStatus.value !== 'All Statuses' && d.status !== selectedStatus.value) {
      return false
    }

    return true
  })
})

const onRowClick = (event: { data: DaemonSetInfo }) => {
  selectedWorkload.value = event.data
  drawerVisible.value = true
}
</script>

<template>
  <ResourceDataTable
    :data="filteredDaemonSets"
    v-model:searchQuery="searchQuery"
    v-model:columns="tableColumns"
    searchPlaceholder="Search daemonsets or images..."
    emptyMessage="No daemonsets found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} daemonsets"
    :loading="loading"
    @refresh="fetchDaemonSets"
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

      <!-- Status Select -->
      <Select
        v-model="selectedStatus"
        :options="statuses"
        class="text-xs min-w-40 bg-(--bg-hover)/30 border-(--border)"
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

    <!-- Status Column -->
    <Column v-if="visibleCols['status']" field="status" header="Status" sortable class="p-3">
      <template #body="{ data }">
        <div class="flex items-center gap-1.5">
          <span
            class="w-1.5 h-1.5 rounded-full"
            :class="data.status === 'Running' ? 'bg-emerald-500' : 'bg-amber-500'"
          ></span>
          <span
            class="font-medium"
            :class="data.status === 'Running' ? 'text-emerald-500' : 'text-amber-500'"
          >
            {{ data.status }}
          </span>
        </div>
      </template>
    </Column>

    <!-- Desired/Current Column -->
    <Column v-if="visibleCols['desiredCurrent']" header="Desired/Current" class="p-3">
      <template #body="{ data }">
        <span class="font-mono text-(--text-secondary)">
          {{ data.replicas.desired }} / {{ data.replicas.current }}
        </span>
      </template>
    </Column>

    <!-- Ready Column -->
    <Column v-if="visibleCols['ready']" header="Ready" class="p-3">
      <template #body="{ data }">
        <span class="font-mono text-emerald-500">{{ data.replicas.ready }}</span>
      </template>
    </Column>

    <!-- Available Column -->
    <Column v-if="visibleCols['available']" header="Available" class="p-3">
      <template #body="{ data }">
        <span class="font-mono text-emerald-400">{{ data.replicas.available }}</span>
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

    <!-- Images Column -->
    <Column v-if="visibleCols['images']" header="Images" class="p-3 max-w-48">
      <template #body="{ data }">
        <div class="flex flex-wrap gap-1">
          <span
            v-for="img in data.images"
            :key="img"
            class="px-1.5 py-0.5 rounded bg-(--bg-hover) text-(--text-secondary) text-[10px] border border-(--border) font-mono truncate max-w-full"
            :title="img"
          >
            {{ img.split('/').pop() }}
          </span>
        </div>
      </template>
    </Column>

    <!-- Drawer -->
    <template #drawer>
      <WorkloadDetailsDrawer v-model:visible="drawerVisible" :workload="selectedWorkload" />
    </template>
  </ResourceDataTable>
</template>
