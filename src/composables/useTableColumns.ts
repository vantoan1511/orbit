import { useTableFilterStore } from '@/stores/tableFilterStore'
import { computed, ref, watch } from 'vue'

export interface TableColumn {
  field: string
  header: string
  visible: boolean
}

export function useTableColumns(initialColumns: TableColumn[], storeKey?: string) {
  const filterStore = storeKey ? useTableFilterStore() : null
  const storedColumns = storeKey && filterStore ? filterStore.getFilters(storeKey).columns : null

  // If we have stored columns, merge visibility into initialColumns (in case column definitions were added/updated)
  let initial = initialColumns
  if (storedColumns && storedColumns.length > 0) {
    const visibilityMap: Record<string, boolean> = Object.fromEntries(
      storedColumns.map((col) => [col.field, col.visible])
    )
    initial = initialColumns.map((col) => ({
      ...col,
      visible: visibilityMap[col.field] !== undefined ? visibilityMap[col.field]! : col.visible
    }))
  }

  const tableColumns = ref<TableColumn[]>(initial)

  if (storeKey && filterStore) {
    watch(
      tableColumns,
      (val) => {
        filterStore.setFilter(storeKey, 'columns', [...val.map((c) => ({ ...c }))])
      },
      { deep: true }
    )
  }

  const visibleCols = computed(() =>
    Object.fromEntries(tableColumns.value.map((col) => [col.field, col.visible]))
  )

  return { tableColumns, visibleCols }
}
