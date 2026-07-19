<script setup lang="ts">
import NamespaceBadge from '@/components/shared/NamespaceBadge.vue'
import NamespaceFilter from '@/components/shared/NamespaceFilter.vue'
import ResourceDataTable from '@/components/shared/ResourceDataTable.vue'
import StatusBadge from '@/components/shared/StatusBadge.vue'
import SystemNamespaceToggle from '@/components/shared/SystemNamespaceToggle.vue'
import ResourceActionMenu from '@/components/shared/ResourceActionMenu.vue'
import { useResourceFilters } from '@/composables/useResourceFilters'
import { useTableColumns } from '@/composables/useTableColumns'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { PodInfo } from '@/types/kubernetes'
import { MoreVertical } from '@lucide/vue'
import Button from 'primevue/button'
import Column from 'primevue/column'
import Select from 'primevue/select'
import { computed, ref } from 'vue'
import PodDetailsDrawer from './PodDetailsDrawer.vue'
import { useWorkloadActions } from '@/composables/useWorkloadActions'

const k8sStore = useKubernetesStore()

const { tableColumns, visibleCols } = useTableColumns([
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'node', header: 'Node', visible: true },
  { field: 'restarts', header: 'Restarts', visible: true },
  { field: 'cpu', header: 'CPU', visible: true },
  { field: 'memory', header: 'Memory', visible: true },
  { field: 'age', header: 'Age', visible: true }
])

const { searchQuery, selectedNamespace, showSystemNamespaces, filteredResources } =
  useResourceFilters(computed(() => k8sStore.pods))

const selectedStatus = ref('All Statuses')

// Drawer state
const drawerVisible = ref(false)
const selectedPod = ref<PodInfo | null>(null)

const namespaces = computed(() => {
  return k8sStore.namespaces
})

const statuses = [
  'All Statuses',
  'Running',
  'Pending',
  'Failed',
  'CrashLoopBackOff',
  'Completed',
  'Unknown'
]

const filteredPods = computed(() => {
  return filteredResources.value.filter((p) => {
    if (selectedStatus.value !== 'All Statuses' && p.status !== selectedStatus.value) {
      return false
    }
    return true
  })
})

const onRowClick = (event: { data: PodInfo }) => {
  selectedPod.value = event.data
  drawerVisible.value = true
}

const actionMenu = ref<InstanceType<typeof ResourceActionMenu> | null>(null)
const selectedActionRow = ref<PodInfo | null>(null)

const toggleActionMenu = (event: Event, data: PodInfo) => {
  event.stopPropagation()
  selectedActionRow.value = data
  actionMenu.value?.toggle(event)
}

const { actionMenuItems } = useWorkloadActions(selectedActionRow, drawerVisible, selectedPod, 'Pod')
</script>

<template>
  <ResourceDataTable
    :data="filteredPods"
    v-model:searchQuery="searchQuery"
    v-model:columns="tableColumns"
    searchPlaceholder="Search pods, images or nodes..."
    emptyMessage="No pods found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} pods"
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

    <!-- Node Column -->
    <Column v-if="visibleCols['node']" field="node" header="Node" sortable class="p-3">
      <template #body="{ data }">
        <span
          class="text-(--text-secondary) font-mono truncate block max-w-44"
          :title="data.node || 'N/A'"
        >
          {{ data.node ? data.node.split('.')[0] : 'N/A' }}
        </span>
      </template>
    </Column>

    <!-- Restarts Column -->
    <Column
      v-if="visibleCols['restarts']"
      field="restarts"
      header="Restarts"
      sortable
      class="p-3 text-center"
    >
      <template #body="{ data }">
        <span
          class="font-mono px-1.5 py-0.5 rounded text-[10px]"
          :class="
            (data.restarts || 0) > 0
              ? 'bg-rose-500/10 text-rose-400 border border-rose-500/20'
              : 'text-(--text-muted)'
          "
        >
          {{ data.restarts || 0 }}
        </span>
      </template>
    </Column>

    <!-- CPU Column -->
    <Column v-if="visibleCols['cpu']" field="cpu" header="CPU" sortable class="p-3">
      <template #body="{ data }">
        <div class="flex flex-col gap-1 w-24">
          <div class="flex justify-between font-mono text-[10px]">
            <span class="text-(--text-secondary)">{{ data.cpu || '-' }}</span>
            <span class="text-(--text-muted)" v-if="data.cpu && data.cpu !== '-'"
              >{{ data.cpuPct || 0 }}%</span
            >
          </div>
          <div
            class="w-full h-1 bg-(--bg-hover) rounded-full overflow-hidden"
            v-if="data.cpu && data.cpu !== '-'"
          >
            <div
              class="h-full rounded-full bg-violet-500"
              :style="{ width: (data.cpuPct || 0) + '%' }"
            ></div>
          </div>
        </div>
      </template>
    </Column>

    <!-- Memory Column -->
    <Column v-if="visibleCols['memory']" field="memory" header="Memory" sortable class="p-3">
      <template #body="{ data }">
        <div class="flex flex-col gap-1 w-24">
          <div class="flex justify-between font-mono text-[10px]">
            <span class="text-(--text-secondary)">{{ data.memory || '-' }}</span>
            <span class="text-(--text-muted)" v-if="data.memory && data.memory !== '-'"
              >{{ data.memoryPct || 0 }}%</span
            >
          </div>
          <div
            class="w-full h-1 bg-(--bg-hover) rounded-full overflow-hidden"
            v-if="data.memory && data.memory !== '-'"
          >
            <div
              class="h-full rounded-full bg-blue-500"
              :style="{ width: (data.memoryPct || 0) + '%' }"
            ></div>
          </div>
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
      <PodDetailsDrawer v-model:visible="drawerVisible" :pod="selectedPod" />
      <ResourceActionMenu ref="actionMenu" :items="actionMenuItems" />
    </template>
  </ResourceDataTable>
</template>
