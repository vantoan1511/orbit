<script setup lang="ts">
import { Info, RefreshCw, Search, Settings2 } from '@lucide/vue'
import Button from 'primevue/button'
import DataTable from 'primevue/datatable'
import InputText from 'primevue/inputtext'

/* eslint-disable @typescript-eslint/no-explicit-any */
withDefaults(
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
    hideConfig: false
  }
)

const emit = defineEmits<{
  (e: 'update:searchQuery', val: string): void
  (e: 'refresh'): void
  (e: 'row-click', event: any): void
}>()

const onSearchUpdate = (val: string | undefined) => {
  emit('update:searchQuery', val ?? '')
}
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
          <Button v-if="!hideConfig" severity="secondary" variant="text" size="small" class="p-1">
            <Settings2 class="w-4 h-4 text-(--text-secondary)" />
          </Button>
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
