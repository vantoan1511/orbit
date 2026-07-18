import { computed, ref } from 'vue'

export interface TableColumn {
  field: string
  header: string
  visible: boolean
}

export function useTableColumns(initialColumns: TableColumn[]) {
  const tableColumns = ref<TableColumn[]>(initialColumns)
  const visibleCols = computed(() =>
    Object.fromEntries(tableColumns.value.map((col) => [col.field, col.visible]))
  )
  return { tableColumns, visibleCols }
}
