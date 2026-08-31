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
import ActivePortForwardBadge from '@/components/shared/ActivePortForwardBadge.vue'
import NamespaceBadge from '@/components/shared/NamespaceBadge.vue'
import NamespaceFilter from '@/components/shared/NamespaceFilter.vue'
import ReactiveAge from '@/components/shared/ReactiveAge.vue'
import ResourceActionMenu from '@/components/shared/ResourceActionMenu.vue'
import ResourceDataTable from '@/components/shared/ResourceDataTable.vue'
import StatusBadge from '@/components/shared/StatusBadge.vue'
import TableFilterSelect from '@/components/shared/TableFilterSelect.vue'
import { useResourceActionMenu } from '@/composables/useResourceActionMenu'
import { useResourceFilters } from '@/composables/useResourceFilters'
import { useTableColumns, type TableColumn } from '@/composables/useTableColumns'
import { useWorkloadActions, type WorkloadActionOptions } from '@/composables/useWorkloadActions'
import { useWorkloadBulkActions } from '@/composables/useWorkloadBulkActions'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { MoreVertical } from '@lucide/vue'
import Button from 'primevue/button'
import Column from 'primevue/column'
import { useTableFilterStore } from '@/stores/tableFilterStore'
import { computed, ref, toRef, watch } from 'vue'

const selection = defineModel<T[]>('selection', { default: () => [] })

const props = withDefaults(
  defineProps<{
    data: T[]
    initialColumns?: TableColumn[]
    searchFields?: string[]
    statuses?: string[]
    kind?: string
    storeKey?: string
    searchPlaceholder?: string
    emptyMessage?: string
    reportTemplate?: string
    loading?: boolean
    selectable?: boolean
    hideNamespaceFilter?: boolean
    hideStatusFilter?: boolean
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
    kind: KUBERNETES_RESOURCE_KIND.Deployment,
    searchPlaceholder: 'Search...',
    emptyMessage: 'No records found matching the filter criteria.',
    reportTemplate: 'Showing {first} to {last} of {totalRecords} items',
    loading: false,
    selectable: true,
    hideNamespaceFilter: false,
    hideStatusFilter: false,
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
const filterStore = useTableFilterStore()

// Resource filtering (search, namespace)
const resolvedStoreKey = (props.storeKey ?? props.kind ?? 'deployment').toLowerCase()

// Column visibility
const { tableColumns, visibleCols } = useTableColumns(props.initialColumns, resolvedStoreKey)

const dataRef = toRef(props, 'data')
const hideNamespaceFilterRef = toRef(props, 'hideNamespaceFilter')
const { searchQuery, selectedNamespace, filteredResources } = useResourceFilters(
  dataRef,
  props.searchFields as (keyof T)[],
  resolvedStoreKey,
  hideNamespaceFilterRef
)

// Status filtering
const storedStatus = filterStore.getFilters(resolvedStoreKey).selectedStatus
const defaultStatus: string =
  props.statuses.length > 0 ? (props.statuses[0] ?? 'All Statuses') : 'All Statuses'
const selectedStatus = ref<string>(storedStatus || defaultStatus)

watch(selectedStatus, (val) => {
  filterStore.setFilter(resolvedStoreKey, 'selectedStatus', val)
})

// Row selection persistence
const storedRowKeys = filterStore.getFilters(resolvedStoreKey).selectedRowKeys
if (storedRowKeys.length > 0) {
  const matched = props.data.filter((item) => storedRowKeys.includes(item.name))
  if (matched.length > 0) {
    selection.value = matched
  }
}

watch(
  () => props.data,
  (newData) => {
    const keys = filterStore.getFilters(resolvedStoreKey).selectedRowKeys
    if (keys.length > 0 && selection.value.length === 0) {
      const matched = newData.filter((item) => keys.includes(item.name))
      if (matched.length > 0) {
        selection.value = matched
      }
    }
  }
)

watch(
  selection,
  (val) => {
    filterStore.setFilter(
      resolvedStoreKey,
      'selectedRowKeys',
      val.map((item) => item.name)
    )
  },
  { deep: true }
)

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

// Rows per page persistence
const storedRows = filterStore.getFilters(resolvedStoreKey).rows
const rowsPerPage = ref<number>(storedRows ?? 25)

watch(rowsPerPage, (val) => {
  filterStore.setFilter(resolvedStoreKey, 'rows', val)
})

// Bulk actions wiring
const { bulkActions } = useWorkloadBulkActions(selection, {
  kind: props.kind,
  clearSelection: () => {
    selection.value = []
  }
})

// Port forward helpers
const getPortForwards = (data: T) => {
  if (!props.kind) return []
  return k8sStore.activePortForwards.filter(
    (pf) =>
      pf.kind.toLowerCase() === props.kind!.toLowerCase() &&
      (!data.namespace || !pf.namespace || pf.namespace === data.namespace) &&
      pf.name === data.name
  )
}
</script>

<template>
  <ResourceDataTable
    :data="filteredData"
    v-model:selection="selection"
    v-model:searchQuery="searchQuery"
    v-model:columns="tableColumns"
    v-model:rows="rowsPerPage"
    :searchPlaceholder="searchPlaceholder"
    :emptyMessage="emptyMessage"
    :reportTemplate="reportTemplate"
    :loading="loading"
    @refresh="emit('refresh')"
    @row-click="onRowClick"
    @row-contextmenu="onRowContextMenu"
  >
    <!-- Bulk Actions -->
    <template #bulk-actions="{ selection: currentSelection, clearSelection }">
      <slot name="bulk-actions" :selection="currentSelection" :clearSelection="clearSelection">
        <Button
          v-for="action in bulkActions"
          :key="action.label"
          :label="action.label"
          :icon="action.icon"
          :severity="action.severity"
          :variant="action.variant"
          :class="action.class"
          size="small"
          @click="action.command"
        />
      </slot>
    </template>

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
      <slot name="actions-left"></slot>
    </template>

    <!-- Selection Column -->
    <Column
      v-if="selectable"
      selectionMode="multiple"
      class="p-3 w-12 text-center"
      headerClass="w-12 text-center"
    />

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
        <div class="flex items-center gap-2">
          <slot name="name" :data="data">
            <span class="font-semibold transition-colors">{{ data.name }}</span>
          </slot>
          <ActivePortForwardBadge :portForwards="getPortForwards(data)" />
        </div>
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
          <ReactiveAge :age="data.age" />
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
