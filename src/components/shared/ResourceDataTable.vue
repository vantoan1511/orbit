<script setup lang="ts">
import { Info, RefreshCw, Settings2, X } from '@lucide/vue'
import Button from 'primevue/button'
import Checkbox from 'primevue/checkbox'
import DataTable from 'primevue/datatable'
import InputText from 'primevue/inputtext'
import Popover from 'primevue/popover'
import TableFilterSelect from '@/components/shared/TableFilterSelect.vue'
import { computed, ref, watch } from 'vue'

import ResourceTableSkeleton from '@/components/shared/ResourceTableSkeleton.vue'
import { type TableColumn } from '@/composables/useTableColumns'

/* eslint-disable @typescript-eslint/no-explicit-any */
const selection = defineModel<any[]>('selection')

const props = withDefaults(
  defineProps<{
    data: any[]
    searchQuery?: string
    searchPlaceholder?: string
    rows?: number
    rowsPerPageOptions?: number[]
    emptyMessage?: string
    reportTemplate?: string
    loading?: boolean
    hideSearch?: boolean
    hideActions?: boolean
    hideRefresh?: boolean
    hideConfig?: boolean
    hideRowsPerPage?: boolean
    columns?: TableColumn[]
  }>(),
  {
    searchQuery: '',
    searchPlaceholder: 'Search...',
    rows: 25,
    rowsPerPageOptions: () => [25, 50, 100, 200],
    emptyMessage: 'No records found matching the filter criteria.',
    reportTemplate: 'Showing {first} to {last} of {totalRecords} items',
    loading: false,
    hideSearch: false,
    hideActions: false,
    hideRefresh: false,
    hideConfig: false,
    hideRowsPerPage: false,
    columns: () => []
  }
)

const emit = defineEmits<{
  (e: 'update:searchQuery', val: string): void
  (e: 'update:columns', val: TableColumn[]): void
  (e: 'update:rows', val: number): void
  (e: 'refresh'): void
  (e: 'row-click', event: any): void
  (e: 'row-contextmenu', event: { originalEvent: Event; data: any; index?: number }): void
}>()

const rowsPerPage = ref(props.rowsPerPageOptions.includes(props.rows) ? props.rows : 25)

watch(rowsPerPage, (newVal) => {
  emit('update:rows', newVal)
})

watch(
  () => props.rows,
  (newRows) => {
    if (newRows && props.rowsPerPageOptions.includes(newRows)) {
      rowsPerPage.value = newRows
    }
  }
)

const onSearchUpdate = (val: string | undefined) => {
  emit('update:searchQuery', val ?? '')
}

const configPopover = ref()
const toggleConfig = (event: Event) => {
  configPopover.value?.toggle(event)
}

const onToggleColumn = (field: string, visible: any) => {
  if (!props.columns) return
  const updated = props.columns.map((col) =>
    col.field === field ? { ...col, visible: !!visible } : col
  )
  emit('update:columns', updated)
}

const allSelected = computed({
  get: () => props.columns?.every((col) => col.visible) ?? false,
  set: (val) => {
    if (!props.columns) return
    const updated = props.columns.map((col) => ({ ...col, visible: val }))
    emit('update:columns', updated)
  }
})

const isIndeterminate = computed(() => {
  if (!props.columns || props.columns.length === 0) return false
  const visibleCount = props.columns.filter((c) => c.visible).length
  return visibleCount > 0 && visibleCount < props.columns.length
})
</script>

<template>
  <div class="flex flex-col gap-4">
    <!-- Bulk Actions Toolbar -->
    <div
      v-if="selection && selection.length > 0"
      class="flex items-center justify-between gap-4 bg-(--bg-hover)/40 border border-(--border) rounded-lg px-3 py-2 min-h-[42px]"
    >
      <div class="flex items-center gap-3">
        <span class="text-xs font-semibold text-primary"> {{ selection.length }} selected </span>
        <Button
          v-tooltip.top="'Clear Selection'"
          severity="secondary"
          variant="text"
          size="small"
          class="p-1! h-6! w-6!"
          @click="selection = []"
        >
          <template #icon>
            <X class="w-3.5 h-3.5 text-muted-color" />
          </template>
        </Button>
      </div>

      <div class="flex items-center gap-2">
        <slot
          name="bulk-actions"
          :selection="selection"
          :clearSelection="() => (selection = [])"
        ></slot>
      </div>
    </div>

    <!-- Filter Toolbar -->
    <div v-else class="flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div class="flex items-center gap-3 flex-wrap">
        <!-- Search -->
        <div class="relative min-w-64" v-if="!hideSearch">
          <IconField>
            <InputIcon class="pi pi-search" />
            <InputText
              :model-value="searchQuery"
              :placeholder="searchPlaceholder"
              @update:model-value="onSearchUpdate"
              variant="filled"
              size="small"
              fluid
            />
          </IconField>
        </div>

        <slot name="filters"></slot>
      </div>

      <!-- Toggles and Actions -->
      <div class="flex items-center gap-4 self-end md:self-auto" v-if="!hideActions">
        <slot name="actions-left"></slot>

        <div v-if="!hideRowsPerPage" class="flex items-center gap-2">
          <span class="text-xs font-medium text-muted-color">Rows:</span>
          <TableFilterSelect v-model="rowsPerPage" :options="rowsPerPageOptions" class="min-w-20" />
        </div>

        <div class="flex items-center gap-1">
          <Button
            v-if="!hideRefresh"
            v-tooltip.top="'Refresh'"
            severity="secondary"
            variant="text"
            size="small"
            class="p-1! w-7! h-7!"
            @click="emit('refresh')"
            :loading="loading"
          >
            <template #icon>
              <RefreshCw class="w-4 h-4 text-muted-color" />
            </template>
          </Button>
          <Button
            v-if="!hideConfig"
            v-tooltip.top="'Configure Columns'"
            severity="secondary"
            variant="text"
            size="small"
            class="p-1! w-7! h-7!"
            @click="toggleConfig"
          >
            <template #icon>
              <Settings2 class="w-4 h-4 text-muted-color" />
            </template>
          </Button>
          <Popover ref="configPopover">
            <div class="flex flex-col gap-2 p-3 min-w-48 text-primary">
              <div class="font-semibold text-sm border-b border-surface pb-1.5 text-muted-color">
                Configure Columns
              </div>
              <div class="flex flex-col gap-1.5 pt-1">
                <div class="flex items-center gap-2 py-0.5 hover:bg-surface-200/20 rounded px-1">
                  <Checkbox
                    inputId="col-all"
                    :modelValue="allSelected"
                    @update:modelValue="allSelected = $event"
                    :binary="true"
                    :indeterminate="isIndeterminate"
                    size="small"
                  />
                  <label
                    for="col-all"
                    class="text-sm cursor-pointer select-none font-semibold text-primary w-full"
                  >
                    All
                  </label>
                </div>
                <div
                  v-for="col in columns"
                  :key="col.field"
                  class="flex items-center gap-2 py-0.5 rounded px-1"
                >
                  <Checkbox
                    :inputId="`col-${col.field}`"
                    :modelValue="col.visible"
                    @update:modelValue="onToggleColumn(col.field, $event)"
                    :binary="true"
                    size="small"
                  />
                  <label
                    :for="`col-${col.field}`"
                    class="text-sm cursor-pointer select-none font-medium text-muted-color w-full"
                  >
                    {{ col.header }}
                  </label>
                </div>
              </div>
            </div>
          </Popover>
        </div>
      </div>
    </div>

    <!-- Table Container -->
    <div class="border border-(--border) rounded-lg overflow-hidden bg-(--bg-card)">
      <!-- Loading Skeleton or Data Table -->
      <slot name="loading" v-if="loading">
        <ResourceTableSkeleton :rows="rowsPerPage" :columns="columns?.length || 6" />
      </slot>

      <!-- Data Table -->
      <DataTable
        v-else
        :value="data"
        v-model:selection="selection"
        dataKey="name"
        paginator
        :rowHover="true"
        v-model:rows="rowsPerPage"
        :rowsPerPageOptions="rowsPerPageOptions"
        class="p-datatable-sm cursor-pointer"
        tableClass="w-full text-left text-xs border-collapse"
        paginatorTemplate="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
        :currentPageReportTemplate="reportTemplate"
        :pt="{
          pcPaginator: {
            pcRowPerPageDropdown: {
              root: {
                class: 'p-variant-filled p-select-sm min-w-20'
              }
            }
          }
        }"
        @row-click="emit('row-click', $event)"
        @row-contextmenu="emit('row-contextmenu', $event)"
      >
        <template #empty>
          <slot name="empty">
            <div class="text-center py-10 text-muted-color flex flex-col items-center gap-2">
              <Info class="w-8 h-8 text-muted-color/50" />
              <span>{{ emptyMessage }}</span>
            </div>
          </slot>
        </template>

        <!-- Pass columns down -->
        <slot></slot>
      </DataTable>
    </div>

    <!-- Details slideout drawer -->
    <slot name="drawer"></slot>
  </div>
</template>
