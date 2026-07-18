<script setup lang="ts">
import NamespaceBadge from '@/components/shared/NamespaceBadge.vue'
import NamespaceFilter from '@/components/shared/NamespaceFilter.vue'
import ResourceDataTable from '@/components/shared/ResourceDataTable.vue'
import StatusBadge from '@/components/shared/StatusBadge.vue'
import SystemNamespaceToggle from '@/components/shared/SystemNamespaceToggle.vue'
import ResourceActionMenu from '@/components/shared/ResourceActionMenu.vue'
import { useResourceFilters } from '@/composables/useResourceFilters'
import { useTableColumns } from '@/composables/useTableColumns'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { StatefulSetInfo } from '@/types/kubernetes'
import { MoreVertical, Info, FileEdit, Scale, ScrollText, Trash2 } from '@lucide/vue'
import Button from 'primevue/button'
import Column from 'primevue/column'
import Select from 'primevue/select'
import { useToast } from 'primevue/usetoast'
import { computed, onMounted, ref, watch } from 'vue'
import WorkloadDetailsDrawer from './WorkloadDetailsDrawer.vue'

const toast = useToast()

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
    computed(() => k8sStore.statefulSets),
    ['name', 'images']
  )

const selectedStatus = ref('All Statuses')
const loading = ref(false)

// Drawer state
const drawerVisible = ref(false)
const selectedWorkload = ref<StatefulSetInfo | null>(null)

const namespaces = computed(() => {
  return k8sStore.namespaces
})

const statuses = ['All Statuses', 'Running', 'Progressing']

const fetchStatefulSets = async () => {
  loading.value = true
  try {
    const ns = selectedNamespace.value.length === 1 ? selectedNamespace.value[0] : undefined
    await kubernetesService.getStatefulSets(ns)
  } catch (e) {
    console.error('Error fetching statefulsets', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchStatefulSets()
})

watch(selectedNamespace, () => {
  fetchStatefulSets()
})

watch(
  () => k8sStore.activeClusterId,
  () => {
    fetchStatefulSets()
  }
)

const filteredStatefulSets = computed(() => {
  return filteredResources.value.filter((s) => {
    if (selectedStatus.value !== 'All Statuses' && s.status !== selectedStatus.value) {
      return false
    }
    return true
  })
})

const onRowClick = (event: { data: StatefulSetInfo }) => {
  selectedWorkload.value = event.data
  drawerVisible.value = true
}

const actionMenu = ref<InstanceType<typeof ResourceActionMenu> | null>(null)
const selectedActionRow = ref<StatefulSetInfo | null>(null)

const toggleActionMenu = (event: Event, data: StatefulSetInfo) => {
  event.stopPropagation()
  selectedActionRow.value = data
  actionMenu.value?.toggle(event)
}

const actionMenuItems = computed(() => [
  {
    label: 'View Details',
    icon: Info,
    command: () => {
      if (selectedActionRow.value) {
        selectedWorkload.value = selectedActionRow.value
        drawerVisible.value = true
      }
    }
  },
  {
    label: 'Edit (YAML)',
    icon: FileEdit,
    command: () => {
      toast.add({
        severity: 'info',
        summary: 'Edit YAML',
        detail: `Edit YAML triggered for ${selectedActionRow.value?.name}`,
        life: 3000
      })
    }
  },
  {
    label: 'Scale',
    icon: Scale,
    command: () => {
      toast.add({
        severity: 'info',
        summary: 'Scale',
        detail: `Scale triggered for ${selectedActionRow.value?.name}`,
        life: 3000
      })
    }
  },
  {
    label: 'View Logs',
    icon: ScrollText,
    class:
      'text-violet-400 hover:text-violet-300 font-semibold border border-violet-500/20 bg-violet-500/5',
    command: () => {
      toast.add({
        severity: 'info',
        summary: 'View Logs',
        detail: `View Logs triggered for ${selectedActionRow.value?.name}`,
        life: 3000
      })
    }
  },
  {
    label: 'Delete',
    icon: Trash2,
    class: 'text-red-400 hover:text-red-300',
    command: () => {
      toast.add({
        severity: 'warn',
        summary: 'Delete',
        detail: `Delete triggered for ${selectedActionRow.value?.name}`,
        life: 3000
      })
    }
  }
])
</script>

<template>
  <ResourceDataTable
    :data="filteredStatefulSets"
    v-model:searchQuery="searchQuery"
    v-model:columns="tableColumns"
    searchPlaceholder="Search statefulsets or images..."
    emptyMessage="No statefulsets found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} statefulsets"
    :loading="loading"
    @refresh="fetchStatefulSets"
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
