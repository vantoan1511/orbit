import type { TableColumn } from '@/composables/useTableColumns'
import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface TableFilterState {
  searchQuery: string
  selectedNamespace: string[]
  isNamespaceInitialized: boolean
  selectedStatus: string
  rows: number
  columns: TableColumn[]
  selectedRowKeys: string[]
  extraFilters: Record<string, string>
}

function defaultState(): TableFilterState {
  return {
    searchQuery: '',
    selectedNamespace: [],
    isNamespaceInitialized: false,
    selectedStatus: 'All Statuses',
    rows: 25,
    columns: [],
    selectedRowKeys: [],
    extraFilters: {}
  }
}

export const useTableFilterStore = defineStore('tableFilter', () => {
  const filters = ref<Record<string, TableFilterState>>({})

  function getFilters(key: string): TableFilterState {
    if (!filters.value[key]) {
      filters.value[key] = defaultState()
    }
    return filters.value[key]
  }

  function setFilter<K extends keyof TableFilterState>(
    key: string,
    field: K,
    value: TableFilterState[K]
  ) {
    getFilters(key)[field] = value
  }

  function setExtraFilter(key: string, field: string, value: string) {
    getFilters(key).extraFilters[field] = value
  }

  function getExtraFilter(key: string, field: string, fallback: string): string {
    return getFilters(key).extraFilters[field] ?? fallback
  }

  function resetFilters(key: string) {
    filters.value[key] = defaultState()
  }

  function resetAll() {
    filters.value = {}
  }

  return {
    getFilters,
    setFilter,
    setExtraFilter,
    getExtraFilter,
    resetFilters,
    resetAll
  }
})
