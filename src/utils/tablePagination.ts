function compareFieldValues(valA: unknown, valB: unknown, order: number): number {
  if (valA === valB) return 0
  if (valA === null || valA === undefined) return 1
  if (valB === null || valB === undefined) return -1

  if (typeof valA === 'number' && typeof valB === 'number') {
    return (valA - valB) * order
  }

  if (typeof valA === 'string' && typeof valB === 'string') {
    return valA.localeCompare(valB) * order
  }

  if (typeof valA === 'boolean' && typeof valB === 'boolean') {
    return (Number(valA) - Number(valB)) * order
  }

  const strA = String(valA)
  const strB = String(valB)
  return strA.localeCompare(strB) * order
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

  return result.sort((a, b) => compareFieldValues(a[sortField], b[sortField], order))
}

/**
 * Stable table row sorting helper.
 * Breaks ties and unsorted states using previous row order indices and row keys,
 * ensuring rows do not jump during high-frequency burst flushes.
 */
export function sortTableRowsStable<T extends Record<string, unknown>>(
  rows: readonly T[],
  sortField: string | null | undefined,
  sortOrder: number | null | undefined,
  getKey: (row: T) => string,
  previousOrder?: ReadonlyMap<string, number>
): { rows: T[]; order: Map<string, number> } {
  const decorated = rows.map((row, index) => {
    const key = getKey(row)
    const prevIndex = previousOrder?.get(key)
    return { row, key, originalIndex: index, prevIndex }
  })

  const hasSort = Boolean(sortField && sortOrder)
  const orderFactor = sortOrder === -1 ? -1 : 1

  decorated.sort((a, b) => {
    if (hasSort && sortField) {
      const cmp = compareFieldValues(a.row[sortField], b.row[sortField], orderFactor)
      if (cmp !== 0) return cmp
    }

    // Tie-breaker: previous order
    if (a.prevIndex !== undefined && b.prevIndex !== undefined) {
      if (a.prevIndex !== b.prevIndex) {
        return a.prevIndex - b.prevIndex
      }
    } else if (a.prevIndex !== undefined) {
      return -1
    } else if (b.prevIndex !== undefined) {
      return 1
    }

    // Secondary tie-breaker: fallback to stable original index, then key
    if (a.originalIndex !== b.originalIndex) {
      return a.originalIndex - b.originalIndex
    }

    return a.key.localeCompare(b.key)
  })

  const sortedRows = decorated.map((d) => d.row)
  const order = new Map<string, number>()
  for (const [i, item] of decorated.entries()) {
    order.set(item.key, i)
  }

  return { rows: sortedRows, order }
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
