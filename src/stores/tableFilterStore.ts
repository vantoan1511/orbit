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

export interface PersistedTableFilters {
  version: number
  defaultRows: number
  tables: Record<string, { rows?: number; columns?: TableColumn[] }>
  clusters: Record<
    string,
    Record<
      string,
      {
        searchQuery?: string
        selectedNamespace?: string[]
        isNamespaceInitialized?: boolean
        selectedStatus?: string
        extraFilters?: Record<string, string>
      }
    >
  >
}

const STORAGE_KEY = 'orbit_table_filter_preferences'
export const ALLOWED_ROW_OPTIONS = [25, 50, 100, 200] as const
export type AllowedRowOption = (typeof ALLOWED_ROW_OPTIONS)[number]

export function isAllowedRowOption(value: unknown): value is AllowedRowOption {
  return typeof value === 'number' && (ALLOWED_ROW_OPTIONS as readonly number[]).includes(value)
}

function getInitialStorage(): PersistedTableFilters {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) {
      const parsed = JSON.parse(raw) as Partial<PersistedTableFilters>
      if (parsed && typeof parsed === 'object') {
        const defaultRowsCandidate = parsed.defaultRows
        const isValidDefaultRows = isAllowedRowOption(defaultRowsCandidate)

        return {
          version: parsed.version ?? 1,
          defaultRows: isValidDefaultRows ? defaultRowsCandidate : 25,
          tables:
            parsed.tables && typeof parsed.tables === 'object' && !Array.isArray(parsed.tables)
              ? parsed.tables
              : {},
          clusters:
            parsed.clusters &&
            typeof parsed.clusters === 'object' &&
            !Array.isArray(parsed.clusters)
              ? parsed.clusters
              : {}
        }
      }
    }
  } catch (e) {
    console.error('Failed to load table filters from localStorage:', e)
  }
  return {
    version: 1,
    defaultRows: 25,
    tables: {},
    clusters: {}
  }
}

let saveTimeout: ReturnType<typeof setTimeout> | null = null
let currentPersistedRef: PersistedTableFilters | null = null

function flushSave() {
  if (saveTimeout && currentPersistedRef) {
    clearTimeout(saveTimeout)
    saveTimeout = null
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(currentPersistedRef))
    } catch (e) {
      console.error('Failed to save table filters to localStorage:', e)
    }
  }
}

if (typeof window !== 'undefined') {
  window.addEventListener('beforeunload', flushSave)
}

function scheduleSave(persisted: PersistedTableFilters) {
  currentPersistedRef = persisted
  if (saveTimeout) clearTimeout(saveTimeout)
  saveTimeout = setTimeout(() => {
    saveTimeout = null
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(persisted))
    } catch (e) {
      console.error('Failed to save table filters to localStorage:', e)
    }
  }, 250)
}

export const useTableFilterStore = defineStore('tableFilter', () => {
  const persistedData = getInitialStorage()
  currentPersistedRef = persistedData
  const defaultRows = ref<number>(persistedData.defaultRows)
  const activeClusterId = ref<string>('default')
  const filters = ref<Record<string, TableFilterState>>({})

  function setActiveClusterId(clusterId: string | null) {
    activeClusterId.value = clusterId || 'default'
  }

  function getClusterId(clusterId?: string): string {
    return clusterId || activeClusterId.value || 'default'
  }

  function getClusterTableRecord(clusterId: string, key: string) {
    if (!persistedData.clusters[clusterId]) {
      persistedData.clusters[clusterId] = {}
    }
    const clusterMap = persistedData.clusters[clusterId]!
    if (!clusterMap[key]) {
      clusterMap[key] = {}
    }
    return clusterMap[key]!
  }

  function getFilters(key: string, clusterId?: string): TableFilterState {
    const cluster = getClusterId(clusterId)
    const fullKey = `${cluster}:${key}`

    if (!filters.value[fullKey]) {
      const savedTable = persistedData.tables[key]
      const savedClusterTable = persistedData.clusters[cluster]?.[key]

      const tableRows =
        savedTable?.rows && isAllowedRowOption(savedTable.rows)
          ? savedTable.rows
          : defaultRows.value

      filters.value[fullKey] = {
        searchQuery: savedClusterTable?.searchQuery ?? '',
        selectedNamespace: savedClusterTable?.selectedNamespace
          ? [...savedClusterTable.selectedNamespace]
          : [],
        isNamespaceInitialized: savedClusterTable?.isNamespaceInitialized ?? false,
        selectedStatus: savedClusterTable?.selectedStatus ?? 'All Statuses',
        rows: tableRows,
        columns: savedTable?.columns ? [...savedTable.columns.map((c) => ({ ...c }))] : [],
        selectedRowKeys: [],
        extraFilters: { ...(savedClusterTable?.extraFilters ?? {}) }
      }
    }
    return filters.value[fullKey]
  }

  function setFilter<K extends keyof TableFilterState>(
    key: string,
    field: K,
    value: TableFilterState[K],
    clusterId?: string
  ) {
    const cluster = getClusterId(clusterId)
    const state = getFilters(key, cluster)
    state[field] = value

    if (field === 'rows') {
      const numRows = value as number
      if (isAllowedRowOption(numRows)) {
        defaultRows.value = numRows
        persistedData.defaultRows = numRows
        if (!persistedData.tables[key]) persistedData.tables[key] = {}
        persistedData.tables[key].rows = numRows
        scheduleSave(persistedData)
      }
    } else if (field === 'columns') {
      if (!persistedData.tables[key]) persistedData.tables[key] = {}
      persistedData.tables[key].columns = (value as TableColumn[]).map((c) => ({ ...c }))
      scheduleSave(persistedData)
    } else if (field === 'searchQuery') {
      const record = getClusterTableRecord(cluster, key)
      record.searchQuery = value as string
      scheduleSave(persistedData)
    } else if (field === 'selectedNamespace') {
      const record = getClusterTableRecord(cluster, key)
      record.selectedNamespace = [...(value as string[])]
      scheduleSave(persistedData)
    } else if (field === 'isNamespaceInitialized') {
      const record = getClusterTableRecord(cluster, key)
      record.isNamespaceInitialized = value as boolean
      scheduleSave(persistedData)
    } else if (field === 'selectedStatus') {
      const record = getClusterTableRecord(cluster, key)
      record.selectedStatus = value as string
      scheduleSave(persistedData)
    }
  }

  function setExtraFilter(key: string, field: string, value: string, clusterId?: string) {
    const cluster = getClusterId(clusterId)
    const state = getFilters(key, cluster)
    state.extraFilters[field] = value
    const record = getClusterTableRecord(cluster, key)
    if (!record.extraFilters) {
      record.extraFilters = {}
    }
    record.extraFilters[field] = value
    scheduleSave(persistedData)
  }

  function getExtraFilter(
    key: string,
    field: string,
    fallback: string,
    clusterId?: string
  ): string {
    const cluster = getClusterId(clusterId)
    return getFilters(key, cluster).extraFilters[field] ?? fallback
  }

  function resetFilters(key: string, clusterId?: string) {
    const cluster = getClusterId(clusterId)
    const fullKey = `${cluster}:${key}`
    const tableRows =
      persistedData.tables[key]?.rows && isAllowedRowOption(persistedData.tables[key]!.rows)
        ? persistedData.tables[key]!.rows!
        : defaultRows.value

    filters.value[fullKey] = {
      searchQuery: '',
      selectedNamespace: [],
      isNamespaceInitialized: false,
      selectedStatus: 'All Statuses',
      rows: tableRows,
      columns: persistedData.tables[key]?.columns
        ? [...persistedData.tables[key].columns!.map((c) => ({ ...c }))]
        : [],
      selectedRowKeys: [],
      extraFilters: {}
    }

    const clusterMap = persistedData.clusters[cluster]
    if (clusterMap && clusterMap[key]) {
      delete clusterMap[key]
      scheduleSave(persistedData)
    }
  }

  function resetAllSelections() {
    // Reset transient row selection keys across all cached tables
    Object.values(filters.value).forEach((state) => {
      state.selectedRowKeys = []
    })
  }

  // Alias for backward compatibility
  const resetAll = resetAllSelections

  return {
    defaultRows,
    activeClusterId,
    setActiveClusterId,
    getFilters,
    setFilter,
    setExtraFilter,
    getExtraFilter,
    resetFilters,
    resetAllSelections,
    resetAll
  }
})
