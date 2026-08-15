<script
  setup
  lang="ts"
  generic="
    T extends {
      name: string
      namespace?: string
      status?: string
      age?: string
      [key: string]: any
    }
  "
>
import NamespaceBadge from '@/components/shared/NamespaceBadge.vue'
import NamespaceFilter from '@/components/shared/NamespaceFilter.vue'
import ResourceActionMenu from '@/components/shared/ResourceActionMenu.vue'
import ResourceDataTable from '@/components/shared/ResourceDataTable.vue'
import StatusBadge from '@/components/shared/StatusBadge.vue'
import SystemNamespaceToggle from '@/components/shared/SystemNamespaceToggle.vue'
import TableFilterSelect from '@/components/shared/TableFilterSelect.vue'
import { useResourceActionMenu } from '@/composables/useResourceActionMenu'
import { useResourceFilters } from '@/composables/useResourceFilters'
import { useTableColumns, type TableColumn } from '@/composables/useTableColumns'
import { useWorkloadActions, type WorkloadActionOptions } from '@/composables/useWorkloadActions'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { MoreVertical } from '@lucide/vue'
import Button from 'primevue/button'
import Column from 'primevue/column'
import { computed, ref, toRef, watch } from 'vue'

const props = withDefaults(
  defineProps<{
    data: T[]
    initialColumns?: TableColumn[]
    searchFields?: string[]
    statuses?: string[]
    kind?: string
    searchPlaceholder?: string
    emptyMessage?: string
    reportTemplate?: string
    loading?: boolean
    hideNamespaceFilter?: boolean
    hideStatusFilter?: boolean
    hideSystemNamespaceToggle?: boolean
    hideNameColumn?: boolean
    hideNamespaceColumn?: boolean
    hideStatusColumn?: boolean
    hideAgeColumn?: boolean
    hideActionsColumn?: boolean
    actionOptions?: WorkloadActionOptions<T>
  }>(),
  {
    initialColumns: () => [
      { field: 'namespace', header: 'Namespace', visible: true },
      { field: 'status', header: 'Status', visible: true },
      { field: 'age', header: 'Age', visible: true }
    ],
    searchFields: () => ['name'],
    statuses: () => [],
    kind: 'Deployment',
    searchPlaceholder: 'Search...',
    emptyMessage: 'No records found matching the filter criteria.',
    reportTemplate: 'Showing {first} to {last} of {totalRecords} items',
    loading: false,
    hideNamespaceFilter: false,
    hideStatusFilter: false,
    hideSystemNamespaceToggle: false,
    hideNameColumn: false,
    hideNamespaceColumn: false,
    hideStatusColumn: false,
    hideAgeColumn: false,
    hideActionsColumn: false
  }
)

const emit = defineEmits<{
  (e: 'refresh'): void
  (e: 'row-click', item: T): void
  (e: 'namespace-change', namespace?: string): void
}>()

const k8sStore = useKubernetesStore()

// Column visibility
const { tableColumns, visibleCols } = useTableColumns(props.initialColumns)

// Resource filtering (search, namespace, system namespaces)
const dataRef = toRef(props, 'data')
const { searchQuery, selectedNamespace, showSystemNamespaces, filteredResources } =
  useResourceFilters(dataRef, props.searchFields as (keyof T)[])

// Status filtering
const selectedStatus = ref(props.statuses.length > 0 ? props.statuses[0] : 'All Statuses')

watch(selectedNamespace, (newNs) => {
  const ns = newNs.length === 1 ? newNs[0] : undefined
  emit('namespace-change', ns)
})

const filteredData = computed(() => {
  return filteredResources.value.filter((item) => {
    if (
      props.statuses.length > 0 &&
      selectedStatus.value !== 'All Statuses' &&
      item.status !== selectedStatus.value
    ) {
      return false
    }
    return true
  })
})

// Drawer & Selection state
const drawerVisible = ref(false)
const selectedItem = ref<T | null>(null)

const handleViewDetails = (item: T) => {
  selectedItem.value = item
  drawerVisible.value = true
}

const onRowClick = (event: { data: T }) => {
  selectedItem.value = event.data
  drawerVisible.value = true
  emit('row-click', event.data)
}

// Action menu wiring
const { actionMenu, selectedActionRow, toggleActionMenu, onRowContextMenu } =
  useResourceActionMenu<T>()

const { actionMenuItems } = useWorkloadActions(selectedActionRow, {
  kind: props.kind,
  onViewDetails: (row) => {
    handleViewDetails(row)
    props.actionOptions?.onViewDetails?.(row)
  }
})
</script>

<template>
  <ResourceDataTable
    :data="filteredData"
    v-model:searchQuery="searchQuery"
    v-model:columns="tableColumns"
    :searchPlaceholder="searchPlaceholder"
    :emptyMessage="emptyMessage"
    :reportTemplate="reportTemplate"
    :loading="loading"
    @refresh="emit('refresh')"
    @row-click="onRowClick"
    @row-contextmenu="onRowContextMenu"
  >
    <!-- Filters -->
    <template #filters>
      <!-- Namespace Filter -->
      <NamespaceFilter
        v-if="!hideNamespaceFilter"
        v-model="selectedNamespace"
        :namespaces="k8sStore.namespaces"
      />

      <!-- Status Filter -->
      <TableFilterSelect
        v-if="!hideStatusFilter && statuses.length > 0"
        v-model="selectedStatus"
        :options="statuses"
      />

      <slot name="filters"></slot>
    </template>

    <!-- Actions Left -->
    <template #actions-left>
      <SystemNamespaceToggle v-if="!hideSystemNamespaceToggle" v-model="showSystemNamespaces" />
      <slot name="actions-left"></slot>
    </template>

    <!-- Name Column -->
    <Column
      v-if="!hideNameColumn"
      field="name"
      header="Name"
      sortable
      class="p-3"
      bodyClass="font-medium text-primary"
    >
      <template #body="{ data }">
        <slot name="name" :data="data">
          <span class="font-semibold transition-colors">{{ data.name }}</span>
        </slot>
      </template>
    </Column>

    <!-- Namespace Column -->
    <Column
      v-if="!hideNamespaceColumn && visibleCols['namespace']"
      field="namespace"
      header="Namespace"
      sortable
      class="p-3"
    >
      <template #body="{ data }">
        <slot name="namespace" :data="data">
          <NamespaceBadge :namespace="data.namespace" />
        </slot>
      </template>
    </Column>

    <!-- Status Column -->
    <Column
      v-if="!hideStatusColumn && visibleCols['status']"
      field="status"
      header="Status"
      sortable
      class="p-3"
    >
      <template #body="{ data }">
        <slot name="status" :data="data">
          <StatusBadge :status="data.status" />
        </slot>
      </template>
    </Column>

    <!-- Custom Middle Columns Slot -->
    <slot :visibleCols="visibleCols"></slot>

    <!-- Age Column -->
    <Column
      v-if="!hideAgeColumn && visibleCols['age']"
      field="age"
      header="Age"
      sortable
      class="p-3"
      bodyClass="text-muted-color font-mono"
    >
      <template #body="{ data }">
        <slot name="age" :data="data">
          <span>{{ data.age }}</span>
        </slot>
      </template>
    </Column>

    <!-- Actions Column -->
    <Column v-if="!hideActionsColumn" class="p-3 text-center w-12 shrink-0">
      <template #body="{ data }">
        <Button
          severity="secondary"
          variant="text"
          size="small"
          class="p-1"
          title="Actions"
          @click="toggleActionMenu($event, data)"
        >
          <MoreVertical class="w-4 h-4 text-muted-color" />
        </Button>
      </template>
    </Column>

    <!-- Drawer and Action Menu -->
    <template #drawer>
      <slot
        name="drawer"
        :selectedItem="selectedItem"
        :visible="drawerVisible"
        :close="() => (drawerVisible = false)"
      ></slot>
      <ResourceActionMenu ref="actionMenu" :items="actionMenuItems" />
    </template>
  </ResourceDataTable>
</template>
