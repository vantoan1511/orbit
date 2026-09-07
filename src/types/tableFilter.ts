export interface TableColumn {
  field: string
  header: string
  visible: boolean
}

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

export const ALLOWED_ROW_OPTIONS = [25, 50, 100, 200] as const
export type AllowedRowOption = (typeof ALLOWED_ROW_OPTIONS)[number]

export function isAllowedRowOption(value: unknown): value is AllowedRowOption {
  return typeof value === 'number' && (ALLOWED_ROW_OPTIONS as readonly number[]).includes(value)
}
