export const VIRTUAL_SCROLL_ROW_THRESHOLD = 50
export const VIRTUAL_SCROLL_ITEM_SIZE = 44
export const VIRTUAL_SCROLL_HEIGHT = '560px'

export interface VirtualScrollerConfig {
  itemSize: number
  numToleratedItems: number
}

export const DEFAULT_VIRTUAL_SCROLLER_OPTIONS: Readonly<VirtualScrollerConfig> = Object.freeze({
  itemSize: VIRTUAL_SCROLL_ITEM_SIZE,
  numToleratedItems: 10
})

/**
 * Determines whether virtual scrolling should be activated based on rows-per-page.
 * Virtual scrolling activates when page rows exceed the threshold (default: 50, e.g. 100 or 200).
 */
export function shouldEnableVirtualScroll(
  pageRowCount: number,
  threshold: number = VIRTUAL_SCROLL_ROW_THRESHOLD
): boolean {
  return pageRowCount > threshold
}

/**
 * Pure table row sorting helper supporting strings and numbers without mutating the input array.
 */
export function sortTableRows<T extends Record<string, unknown>>(
  rows: readonly T[],
  sortField?: string | null,
  sortOrder?: number | null
): T[] {
  const result = [...rows]
  if (!sortField || !sortOrder) {
    return result
  }

  const order = sortOrder === -1 ? -1 : 1

  return result.sort((a, b) => {
    const valA = a[sortField]
    const valB = b[sortField]

    if (valA === valB) return 0
    if (valA === null || valA === undefined) return 1
    if (valB === null || valB === undefined) return -1

    if (typeof valA === 'number' && typeof valB === 'number') {
      return (valA - valB) * order
    }

    if (typeof valA === 'string' && typeof valB === 'string') {
      return valA.localeCompare(valB) * order
    }

    if (valA > valB) return order
    if (valA < valB) return -order
    return 0
  })
}

/**
 * Pure table row pagination helper.
 * Clamps the `first` index when total records change and slices the active page items.
 */
export function paginateTableRows<T>(
  rows: readonly T[],
  first: number,
  rowsPerPage: number
): { pageRows: T[]; clampedFirst: number } {
  const total = rows.length
  if (total === 0 || rowsPerPage <= 0) {
    return { pageRows: [], clampedFirst: 0 }
  }

  const validRowsPerPage = Math.max(1, rowsPerPage)
  const maxFirst = Math.max(0, Math.floor((total - 1) / validRowsPerPage) * validRowsPerPage)
  const safeFirst = Number.isFinite(first) ? Math.max(0, first) : 0
  const clampedFirst = Math.min(safeFirst, maxFirst)

  const pageRows = rows.slice(clampedFirst, clampedFirst + validRowsPerPage)
  return { pageRows, clampedFirst }
}
