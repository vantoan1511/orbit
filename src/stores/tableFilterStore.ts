import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  createDefaultFilters,
  TABLE_FILTER_STORAGE_KEY,
  tableFilterService
} from '../services/tableFilterService.ts'
import type {
  AllowedRowOption,
  PersistedTableFilters,
  TableColumn,
  TableFilterState
} from '../types/tableFilter.ts'
import { ALLOWED_ROW_OPTIONS, isAllowedRowOption } from '../types/tableFilter.ts'

export { ALLOWED_ROW_OPTIONS, isAllowedRowOption, TABLE_FILTER_STORAGE_KEY }
export type { AllowedRowOption, PersistedTableFilters, TableColumn, TableFilterState }

let saveTimeout: ReturnType<typeof setTimeout> | null = null
let currentPersistedRef: PersistedTableFilters | null = null

async function flushSave() {
  if (currentPersistedRef) {
    if (saveTimeout) {
      clearTimeout(saveTimeout)
      saveTimeout = null
    }
    await tableFilterService.saveFilters(currentPersistedRef)
  }
}

if (typeof window !== 'undefined') {
  window.addEventListener('beforeunload', () => {
    void flushSave()
  })
}

function scheduleSave(persisted: PersistedTableFilters, immediate = false) {
  currentPersistedRef = persisted
  if (saveTimeout) {
    clearTimeout(saveTimeout)
    saveTimeout = null
  }
  if (immediate) {
    void tableFilterService.saveFilters(persisted)
    return
  }
  saveTimeout = setTimeout(async () => {
    saveTimeout = null
    await tableFilterService.saveFilters(persisted)
  }, 250)
}

export const useTableFilterStore = defineStore('tableFilter', () => {
  let persistedData: PersistedTableFilters = createDefaultFilters()
  currentPersistedRef = persistedData

  const isInitialized = ref<boolean>(false)
  const defaultRows = ref<number>(persistedData.defaultRows)
  const activeClusterId = ref<string>('default')
  const filters = ref<Record<string, TableFilterState>>({})

  function applyPersistedData(loaded: PersistedTableFilters) {
    persistedData = loaded
    currentPersistedRef = loaded
    defaultRows.value = loaded.defaultRows

    // Sync any existing in-memory cached tables with loaded data
    for (const [fullKey, state] of Object.entries(filters.value)) {
      const [cluster, ...keyParts] = fullKey.split(':')
      const key = keyParts.join(':')
      if (cluster && key) {
        const savedTable = loaded.tables[key]
        const savedClusterTable = loaded.clusters[cluster]?.[key]

        if (savedTable?.rows && isAllowedRowOption(savedTable.rows)) {
          state.rows = savedTable.rows
        }
        if (savedTable?.columns && savedTable.columns.length > 0) {
          state.columns = savedTable.columns.map((c) => ({ ...c }))
        }
        if (savedClusterTable) {
          if (savedClusterTable.searchQuery !== undefined) {
            state.searchQuery = savedClusterTable.searchQuery
          }
          if (savedClusterTable.selectedNamespace !== undefined) {
            state.selectedNamespace = [...savedClusterTable.selectedNamespace]
          }
          if (savedClusterTable.isNamespaceInitialized !== undefined) {
            state.isNamespaceInitialized = savedClusterTable.isNamespaceInitialized
          }
          if (savedClusterTable.selectedStatus !== undefined) {
            state.selectedStatus = savedClusterTable.selectedStatus
          }
          if (savedClusterTable.extraFilters !== undefined) {
            state.extraFilters = { ...savedClusterTable.extraFilters }
          }
        }
      }
    }
  }

  let initPromise: Promise<PersistedTableFilters> | null = null

  async function init(): Promise<PersistedTableFilters> {
    if (isInitialized.value) return persistedData
    if (initPromise) return initPromise

    initPromise = (async () => {
      try {
        const loaded = await tableFilterService.loadFilters()
        applyPersistedData(loaded)
        isInitialized.value = true
        return loaded
      } catch (e) {
        console.warn('Failed to initialize table filters:', e)
        isInitialized.value = true
        return persistedData
      } finally {
        initPromise = null
      }
    })()

    return initPromise
  }

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
        scheduleSave(persistedData, true)
      }
    } else if (field === 'columns') {
      if (!persistedData.tables[key]) persistedData.tables[key] = {}
      persistedData.tables[key].columns = (value as TableColumn[]).map((c) => ({ ...c }))
      scheduleSave(persistedData, true)
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
      scheduleSave(persistedData, true)
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

  // Automatically trigger hydration on store creation in browser runtime
  if (typeof window !== 'undefined') {
    void init()
  }

  return {
    isInitialized,
    init,
    flushSave,
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
