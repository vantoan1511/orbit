import assert from 'node:assert/strict'
import test from 'node:test'
import {
  VIRTUAL_SCROLL_ROW_THRESHOLD,
  VIRTUAL_SCROLL_ITEM_SIZE,
  VIRTUAL_SCROLL_HEIGHT,
  DEFAULT_VIRTUAL_SCROLLER_OPTIONS,
  shouldEnableVirtualScroll,
  sortTableRows,
  paginateTableRows
} from '../tableVirtualPagination.ts'

test('shouldEnableVirtualScroll returns false for <= 50 rows and true for > 50 rows (100, 200)', () => {
  assert.equal(VIRTUAL_SCROLL_ROW_THRESHOLD, 50)
  assert.equal(VIRTUAL_SCROLL_ITEM_SIZE, 44)
  assert.equal(VIRTUAL_SCROLL_HEIGHT, '560px')
  assert.equal(DEFAULT_VIRTUAL_SCROLLER_OPTIONS.itemSize, 44)
  assert.equal(DEFAULT_VIRTUAL_SCROLLER_OPTIONS.numToleratedItems, 10)

  assert.equal(shouldEnableVirtualScroll(10), false)
  assert.equal(shouldEnableVirtualScroll(25), false)
  assert.equal(shouldEnableVirtualScroll(50), false)
  assert.equal(shouldEnableVirtualScroll(51), true)
  assert.equal(shouldEnableVirtualScroll(100), true)
  assert.equal(shouldEnableVirtualScroll(200), true)

  // Custom threshold
  assert.equal(shouldEnableVirtualScroll(20, 20), false)
  assert.equal(shouldEnableVirtualScroll(21, 20), true)
})

test('sortTableRows sorts ascending and descending by numeric and string fields without mutating input array', () => {
  const original = [
    { name: 'charlie', count: 3, age: 30 },
    { name: 'alice', count: 10, age: 25 },
    { name: 'bob', count: 1, age: 40 }
  ]
  const copy = [...original]

  // No sort field
  const noSort = sortTableRows(original, null, 1)
  assert.deepEqual(noSort, original)
  assert.notEqual(noSort, original) // returns copy

  // Sort ascending by string
  const sortedNameAsc = sortTableRows(original, 'name', 1)
  assert.deepEqual(
    sortedNameAsc.map((r) => r.name),
    ['alice', 'bob', 'charlie']
  )

  // Sort descending by string
  const sortedNameDesc = sortTableRows(original, 'name', -1)
  assert.deepEqual(
    sortedNameDesc.map((r) => r.name),
    ['charlie', 'bob', 'alice']
  )

  // Sort ascending by number
  const sortedCountAsc = sortTableRows(original, 'count', 1)
  assert.deepEqual(
    sortedCountAsc.map((r) => r.count),
    [1, 3, 10]
  )

  // Sort descending by number
  const sortedCountDesc = sortTableRows(original, 'count', -1)
  assert.deepEqual(
    sortedCountDesc.map((r) => r.count),
    [10, 3, 1]
  )

  // Original array must NOT be mutated
  assert.deepEqual(original, copy)

  // Handling nullish values
  const withNulls = [
    { name: 'b', val: null },
    { name: 'a', val: 5 },
    { name: 'c', val: undefined }
  ]
  const sortedNullsAsc = sortTableRows(withNulls, 'val', 1)
  assert.equal(sortedNullsAsc[0].name, 'a')
})

test('paginateTableRows slices current page and clamps out-of-range first offset when total records shrink', () => {
  const items = Array.from({ length: 95 }, (_, i) => ({ id: i + 1 }))

  // First page, 25 per page
  const page1 = paginateTableRows(items, 0, 25)
  assert.equal(page1.clampedFirst, 0)
  assert.equal(page1.pageRows.length, 25)
  assert.equal(page1.pageRows[0].id, 1)
  assert.equal(page1.pageRows[24].id, 25)

  // Last page (first = 75, 20 items left)
  const page4 = paginateTableRows(items, 75, 25)
  assert.equal(page4.clampedFirst, 75)
  assert.equal(page4.pageRows.length, 20)
  assert.equal(page4.pageRows[0].id, 76)
  assert.equal(page4.pageRows[19].id, 95)

  // Out of range (first = 150, but total is 95, rowsPerPage = 25 -> max first is 75)
  const clamped = paginateTableRows(items, 150, 25)
  assert.equal(clamped.clampedFirst, 75)
  assert.equal(clamped.pageRows.length, 20)
  assert.equal(clamped.pageRows[0].id, 76)

  // Empty items
  const empty = paginateTableRows([], 50, 25)
  assert.equal(empty.clampedFirst, 0)
  assert.deepEqual(empty.pageRows, [])

  // Negative first
  const neg = paginateTableRows(items, -10, 25)
  assert.equal(neg.clampedFirst, 0)
  assert.equal(neg.pageRows.length, 25)
})
