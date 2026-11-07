import assert from 'node:assert/strict'
import test from 'node:test'
import { sortTableRows, sortTableRowsStable, paginateTableRows } from '../tablePagination.ts'

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

test('sortTableRowsStable: unsorted keeps previous positions, appends new rows', () => {
  // prev [a,b,c], new data [c,a,d,b] -> [a,b,c,d]
  const previousOrder = new Map<string, number>([
    ['a', 0],
    ['b', 1],
    ['c', 2]
  ])

  const newData = [
    { id: 'c', val: 3 },
    { id: 'a', val: 1 },
    { id: 'd', val: 4 },
    { id: 'b', val: 2 }
  ]

  const result = sortTableRowsStable(newData, null, null, (r) => r.id, previousOrder)
  assert.deepEqual(
    result.rows.map((r) => r.id),
    ['a', 'b', 'c', 'd']
  )
  assert.equal(result.order.get('a'), 0)
  assert.equal(result.order.get('b'), 1)
  assert.equal(result.order.get('c'), 2)
  assert.equal(result.order.get('d'), 3)
})

test('sortTableRowsStable: ties keep previous order', () => {
  // 3 pods with same status 'Running', previous order p1, p2, p3
  const previousOrder = new Map<string, number>([
    ['p1', 0],
    ['p2', 1],
    ['p3', 2]
  ])

  // New data shuffled order [p3, p1, p2]
  const shuffledData = [
    { id: 'p3', status: 'Running' },
    { id: 'p1', status: 'Running' },
    { id: 'p2', status: 'Running' }
  ]

  const result = sortTableRowsStable(shuffledData, 'status', 1, (r) => r.id, previousOrder)

  // Because all status values are identical, previous relative order [p1, p2, p3] must be preserved
  assert.deepEqual(
    result.rows.map((r) => r.id),
    ['p1', 'p2', 'p3']
  )
})

test('sortTableRowsStable: distinct values still sort correctly and does not mutate input', () => {
  const original = [
    { id: '1', score: 10 },
    { id: '2', score: 50 },
    { id: '3', score: 20 }
  ]
  const copy = [...original]

  const resultAsc = sortTableRowsStable(original, 'score', 1, (r) => r.id)
  assert.deepEqual(
    resultAsc.rows.map((r) => r.id),
    ['1', '3', '2']
  )

  const resultDesc = sortTableRowsStable(original, 'score', -1, (r) => r.id)
  assert.deepEqual(
    resultDesc.rows.map((r) => r.id),
    ['2', '3', '1']
  )

  // Input array not mutated
  assert.deepEqual(original, copy)
})
