<script setup lang="ts">
import { Info, RefreshCw, Search, Settings2 } from '@lucide/vue'
import Button from 'primevue/button'
import Checkbox from 'primevue/checkbox'
import DataTable from 'primevue/datatable'
import InputText from 'primevue/inputtext'
import Popover from 'primevue/popover'
import { computed, ref } from 'vue'

/* eslint-disable @typescript-eslint/no-explicit-any */
export interface TableColumn {
  field: string
  header: string
  visible: boolean
}

const props = withDefaults(
  defineProps<{
    data: any[]
    searchQuery?: string
    searchPlaceholder?: string
    rows?: number
    emptyMessage?: string
    reportTemplate?: string
    loading?: boolean
    hideSearch?: boolean
    hideActions?: boolean
    hideRefresh?: boolean
    hideConfig?: boolean
    columns?: TableColumn[]
  }>(),
  {
    searchQuery: '',
    searchPlaceholder: 'Search...',
    rows: 10,
    emptyMessage: 'No records found matching the filter criteria.',
    reportTemplate: 'Showing {first} to {last} of {totalRecords} items',
    loading: false,
    hideSearch: false,
    hideActions: false,
    hideRefresh: false,
    hideConfig: false,
    columns: () => []
  }
)

const emit = defineEmits<{
  (e: 'update:searchQuery', val: string): void
  (e: 'update:columns', val: TableColumn[]): void
  (e: 'refresh'): void
  (e: 'row-click', event: any): void
}>()

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
  <div
    class="bg-(--bg-card) border border-(--border) rounded-xl p-6 shadow-sm flex flex-col gap-6 transition-all duration-200"
  >
    <!-- Filter Toolbar -->
    <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div class="flex items-center gap-3 flex-wrap">
        <!-- Search -->
        <div class="relative min-w-64" v-if="!hideSearch">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-(--text-muted)" />
          <InputText
            :model-value="searchQuery"
            @update:model-value="onSearchUpdate"
            :placeholder="searchPlaceholder"
            class="pl-9 pr-4 py-2 w-full text-xs bg-(--bg-hover)/30 border-(--border) text-(--text-primary) rounded-lg focus:border-violet-500 focus:ring-1 focus:ring-violet-500"
          />
        </div>

        <slot name="filters"></slot>
      </div>

      <!-- Toggles and Actions -->
      <div class="flex items-center gap-4 self-end md:self-auto" v-if="!hideActions">
        <slot name="actions-left"></slot>

        <div class="flex items-center gap-1">
          <Button
            v-if="!hideRefresh"
            severity="secondary"
            variant="text"
            size="small"
            class="p-1"
            @click="emit('refresh')"
            :loading="loading"
          >
            <RefreshCw class="w-4 h-4 text-(--text-secondary)" />
          </Button>
          <Button
            v-if="!hideConfig"
            severity="secondary"
            variant="text"
            size="small"
            class="p-1"
            @click="toggleConfig"
          >
            <Settings2 class="w-4 h-4 text-(--text-secondary)" />
          </Button>
          <Popover ref="configPopover">
            <div class="flex flex-col gap-2 p-3 min-w-48 bg-(--bg-card) text-(--text-primary)">
              <div
                class="font-semibold text-xs border-b border-(--border) pb-1.5 text-(--text-secondary)"
              >
                Configure Columns
              </div>
              <div class="flex flex-col gap-1.5 pt-1">
                <div
                  class="flex items-center gap-2 py-0.5 hover:bg-(--bg-hover)/20 rounded px-1 border-b border-(--border)/50"
                >
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
                    class="text-xs cursor-pointer select-none font-semibold text-(--text-primary) w-full"
                  >
                    All
                  </label>
                </div>
                <div
                  v-for="col in columns"
                  :key="col.field"
                  class="flex items-center gap-2 py-0.5 hover:bg-(--bg-hover)/20 rounded px-1"
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
                    class="text-xs cursor-pointer select-none font-medium text-(--text-secondary) w-full"
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

    <!-- Data Table -->
    <DataTable
      :value="data"
      paginator
      :rows="rows"
      class="p-datatable-sm border border-(--border) rounded-lg overflow-hidden cursor-pointer"
      tableClass="w-full text-left text-xs border-collapse"
      paginatorTemplate="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
      :currentPageReportTemplate="reportTemplate"
      @row-click="emit('row-click', $event)"
    >
      <template #empty>
        <slot name="empty">
          <div class="text-center py-10 text-(--text-muted) flex flex-col items-center gap-2">
            <Info class="w-8 h-8 text-(--text-muted)/50" />
            <span>{{ emptyMessage }}</span>
          </div>
        </slot>
      </template>

      <!-- Pass columns down -->
      <slot></slot>
    </DataTable>

    <!-- Details slideout drawer -->
    <slot name="drawer"></slot>
  </div>
</template>
