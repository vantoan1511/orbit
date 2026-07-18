<script setup lang="ts">
import ResourceDataTable, { type TableColumn } from '@/components/shared/ResourceDataTable.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { JobInfo } from '@/types/kubernetes'
import Column from 'primevue/column'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import { computed, onMounted, ref, watch } from 'vue'
import WorkloadDetailsDrawer from './WorkloadDetailsDrawer.vue'

const k8sStore = useKubernetesStore()

const tableColumns = ref<TableColumn[]>([
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'completions', header: 'Completions', visible: true },
  { field: 'duration', header: 'Duration', visible: true },
  { field: 'age', header: 'Age', visible: true },
  { field: 'images', header: 'Images', visible: true }
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
const selectedStatus = ref('All Statuses')
const showSystemNamespaces = ref(false)
const loading = ref(false)

// Drawer state
const drawerVisible = ref(false)
const selectedWorkload = ref<JobInfo | null>(null)

const namespaces = computed(() => {
  return k8sStore.namespaces
})

const statuses = ['All Statuses', 'Active', 'Succeeded', 'Failed', 'Unknown']

const fetchJobs = async () => {
  loading.value = true
  try {
    const ns = selectedNamespace.value === 'All Namespaces' ? undefined : selectedNamespace.value
    await kubernetesService.getJobs(ns)
  } catch (e) {
    console.error('Error fetching jobs', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchJobs()
})

watch(selectedNamespace, () => {
  fetchJobs()
})

watch(
  () => k8sStore.activeClusterId,
  () => {
    fetchJobs()
  }
)

const filteredJobs = computed(() => {
  return k8sStore.jobs.filter((j) => {
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      const matchesName = j.name.toLowerCase().includes(query)
      const matchesImage = j.images?.some((img) => img.toLowerCase().includes(query))
      if (!matchesName && !matchesImage) return false
    }

    const isSystem = ['kube-system', 'monitoring', 'logging'].includes(j.namespace)
    if (!showSystemNamespaces.value && isSystem && selectedNamespace.value === 'All Namespaces') {
      return false
    }

    if (selectedStatus.value !== 'All Statuses' && j.status !== selectedStatus.value) {
      return false
    }

    return true
  })
})

const onRowClick = (event: { data: JobInfo }) => {
  selectedWorkload.value = event.data
  drawerVisible.value = true
}
</script>

<template>
  <ResourceDataTable
    :data="filteredJobs"
    v-model:searchQuery="searchQuery"
    v-model:columns="tableColumns"
    searchPlaceholder="Search jobs or images..."
    emptyMessage="No jobs found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} jobs"
    :loading="loading"
    @refresh="fetchJobs"
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
            :class="
              data.status === 'Succeeded'
                ? 'bg-emerald-500'
                : data.status === 'Active'
                  ? 'bg-amber-500'
                  : 'bg-rose-500'
            "
          ></span>
          <span
            class="font-medium"
            :class="
              data.status === 'Succeeded'
                ? 'text-emerald-500'
                : data.status === 'Active'
                  ? 'text-amber-500'
                  : 'text-rose-500'
            "
          >
            {{ data.status }}
          </span>
        </div>
      </template>
    </Column>

    <!-- Completions Column -->
    <Column
      v-if="visibleCols['completions']"
      field="completions"
      header="Completions"
      sortable
      class="p-3 font-mono text-(--text-secondary)"
    ></Column>

    <!-- Duration Column -->
    <Column
      v-if="visibleCols['duration']"
      field="duration"
      header="Duration"
      sortable
      class="p-3 font-mono text-(--text-secondary)"
    >
      <template #body="{ data }">
        <span>{{ data.duration || '-' }}</span>
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
