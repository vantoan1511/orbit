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
import type { ReplicaSetInfo } from '@/types/kubernetes'
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
  { field: 'replicas', header: 'Replicas', visible: true },
  { field: 'age', header: 'Age', visible: true },
  { field: 'images', header: 'Images', visible: true }
])

const { searchQuery, selectedNamespace, showSystemNamespaces, filteredResources } =
  useResourceFilters(
    computed(() => k8sStore.replicaSets),
    ['name', 'images']
  )

const selectedStatus = ref('All Statuses')
const loading = ref(false)

// Drawer state
const drawerVisible = ref(false)
const selectedWorkload = ref<ReplicaSetInfo | null>(null)

const namespaces = computed(() => {
  return k8sStore.namespaces
})

const statuses = ['All Statuses', 'Running', 'Progressing']

const fetchReplicaSets = async () => {
  loading.value = true
  try {
    const ns = selectedNamespace.value.length === 1 ? selectedNamespace.value[0] : undefined
    await kubernetesService.getReplicaSets(ns)
  } catch (e) {
    console.error('Error fetching replicasets', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchReplicaSets()
})

watch(selectedNamespace, () => {
  fetchReplicaSets()
})

watch(
  () => k8sStore.activeClusterId,
  () => {
    fetchReplicaSets()
  }
)

const filteredReplicaSets = computed(() => {
  return filteredResources.value.filter((r) => {
    if (selectedStatus.value !== 'All Statuses' && r.status !== selectedStatus.value) {
      return false
    }
    return true
  })
})

const onRowClick = (event: { data: ReplicaSetInfo }) => {
  selectedWorkload.value = event.data
  drawerVisible.value = true
}

const actionMenu = ref<InstanceType<typeof ResourceActionMenu> | null>(null)
const selectedActionRow = ref<ReplicaSetInfo | null>(null)

const toggleActionMenu = (event: Event, data: ReplicaSetInfo) => {
  event.stopPropagation()
  selectedActionRow.value = data
  actionMenu.value?.toggle(event)
}

const { actionMenuItems } = useWorkloadActions(selectedActionRow, {
  kind: 'ReplicaSet',
  onViewDetails: (row) => {
    selectedWorkload.value = row
    drawerVisible.value = true
  }
})
</script>

<template>
  <ResourceDataTable
    :data="filteredReplicaSets"
    v-model:searchQuery="searchQuery"
    v-model:columns="tableColumns"
    searchPlaceholder="Search replicasets or images..."
    emptyMessage="No replicasets found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} replicasets"
    :loading="loading"
    @refresh="fetchReplicaSets"
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

    <!-- Replicas Column -->
    <Column v-if="visibleCols['replicas']" header="Replicas" class="p-3">
      <template #body="{ data }">
        <div class="flex items-center gap-2 font-mono text-(--text-secondary)">
          <span class="font-bold">{{ data.replicas.current }}</span>
          <span class="text-(--text-muted)">/</span>
          <span>{{ data.replicas.desired }}</span>
        </div>
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
