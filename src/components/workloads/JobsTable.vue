<script setup lang="ts">
import NamespaceBadge from '@/components/shared/NamespaceBadge.vue'
import NamespaceFilter from '@/components/shared/NamespaceFilter.vue'
import ResourceDataTable from '@/components/shared/ResourceDataTable.vue'
import StatusBadge from '@/components/shared/StatusBadge.vue'
import SystemNamespaceToggle from '@/components/shared/SystemNamespaceToggle.vue'
import { useResourceFilters } from '@/composables/useResourceFilters'
import { useTableColumns } from '@/composables/useTableColumns'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { JobInfo } from '@/types/kubernetes'
import Column from 'primevue/column'
import Select from 'primevue/select'
import { computed, onMounted, ref, watch } from 'vue'
import WorkloadDetailsDrawer from './WorkloadDetailsDrawer.vue'
import ResourceActionMenu from '@/components/shared/ResourceActionMenu.vue'
import { useWorkloadActions } from '@/composables/useWorkloadActions'
import { MoreVertical } from '@lucide/vue'
import Button from 'primevue/button'

const k8sStore = useKubernetesStore()

const { tableColumns, visibleCols } = useTableColumns([
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'completions', header: 'Completions', visible: true },
  { field: 'duration', header: 'Duration', visible: true },
  { field: 'age', header: 'Age', visible: true },
  { field: 'images', header: 'Images', visible: true }
])

const { searchQuery, selectedNamespace, showSystemNamespaces, filteredResources } =
  useResourceFilters(
    computed(() => k8sStore.jobs),
    ['name', 'images']
  )

const selectedStatus = ref('All Statuses')
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
    const ns = selectedNamespace.value.length === 1 ? selectedNamespace.value[0] : undefined
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
  return filteredResources.value.filter((j) => {
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

const actionMenu = ref<InstanceType<typeof ResourceActionMenu> | null>(null)
const selectedActionRow = ref<JobInfo | null>(null)

const toggleActionMenu = (event: Event, data: JobInfo) => {
  event.stopPropagation()
  selectedActionRow.value = data
  actionMenu.value?.toggle(event)
}

const { actionMenuItems } = useWorkloadActions(selectedActionRow, {
  kind: 'Job',
  onViewDetails: (row) => {
    selectedWorkload.value = row
    drawerVisible.value = true
  }
})
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
      <NamespaceFilter v-model="selectedNamespace" :namespaces="namespaces" />

      <!-- Status Select -->
      <Select
        v-model="selectedStatus"
        :options="statuses"
        class="text-xs min-w-40 bg-(--bg-hover)/30 border-(--border)"
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

    <!-- Status Column -->
    <Column v-if="visibleCols['status']" field="status" header="Status" sortable class="p-3">
      <template #body="{ data }">
        <StatusBadge :status="data.status" />
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

    <!-- Actions Column -->
    <Column class="p-3 text-center w-12 shrink-0">
      <template #body="{ data }">
        <Button
          severity="secondary"
          variant="text"
          size="small"
          class="p-1"
          title="Actions"
          @click="toggleActionMenu($event, data)"
        >
          <MoreVertical class="w-4 h-4 text-(--text-muted)" />
        </Button>
      </template>
    </Column>

    <!-- Drawer -->
    <template #drawer>
      <WorkloadDetailsDrawer v-model:visible="drawerVisible" :workload="selectedWorkload" />
      <ResourceActionMenu ref="actionMenu" :items="actionMenuItems" />
    </template>
  </ResourceDataTable>
</template>
