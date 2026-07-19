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
import type { DaemonSetInfo } from '@/types/kubernetes'
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
  { field: 'desiredCurrent', header: 'Desired/Current', visible: true },
  { field: 'ready', header: 'Ready', visible: true },
  { field: 'available', header: 'Available', visible: true },
  { field: 'age', header: 'Age', visible: true },
  { field: 'images', header: 'Images', visible: true }
])

const { searchQuery, selectedNamespace, showSystemNamespaces, filteredResources } =
  useResourceFilters(
    computed(() => k8sStore.daemonSets),
    ['name', 'images']
  )

const selectedStatus = ref('All Statuses')
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
    const ns = selectedNamespace.value.length === 1 ? selectedNamespace.value[0] : undefined
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
  return filteredResources.value.filter((d) => {
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

const actionMenu = ref<InstanceType<typeof ResourceActionMenu> | null>(null)
const selectedActionRow = ref<DaemonSetInfo | null>(null)

const toggleActionMenu = (event: Event, data: DaemonSetInfo) => {
  event.stopPropagation()
  selectedActionRow.value = data
  actionMenu.value?.toggle(event)
}

const { actionMenuItems } = useWorkloadActions(
  selectedActionRow,
  drawerVisible,
  selectedWorkload,
  'DaemonSet'
)
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
