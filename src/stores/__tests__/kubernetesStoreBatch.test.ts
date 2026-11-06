import assert from 'node:assert/strict'
import test from 'node:test'
import { shallowRef } from 'vue'
import { KUBERNETES_ACTION } from '../../constants/kubernetes.ts'
import {
  MAX_EVENTS_RETAINED,
  mergeNodeWatchUpdate,
  resourcesEqual,
  updateResourceBatch
} from '../kubernetesStoreBatch.ts'

interface TestResource {
  name: string
  namespace: string
  version?: number
  metadata?: {
    labels?: Record<string, string>
    annotations?: string[]
  }
}

test('resourcesEqual handles primitives, nested objects, ordered arrays, and undefined vs missing keys', () => {
  // Primitives
  assert.equal(resourcesEqual(1, 1), true)
  assert.equal(resourcesEqual('a', 'a'), true)
  assert.equal(resourcesEqual(true, true), true)
  assert.equal(resourcesEqual(null, null), true)
  assert.equal(resourcesEqual(null, undefined), false)
  assert.equal(resourcesEqual(1, '1'), false)

  // Arrays: ordered
  assert.equal(resourcesEqual([1, 2, 3], [1, 2, 3]), true)
  assert.equal(resourcesEqual([1, 2, 3], [1, 3, 2]), false)
  assert.equal(resourcesEqual([1, 2], [1, 2, 3]), false)

  // Nested objects
  assert.equal(
    resourcesEqual({ a: 1, b: { c: [1, 2], d: 'hello' } }, { a: 1, b: { c: [1, 2], d: 'hello' } }),
    true
  )
  assert.equal(
    resourcesEqual({ a: 1, b: { c: [1, 2], d: 'hello' } }, { a: 1, b: { c: [1, 3], d: 'hello' } }),
    false
  )

  // Undefined vs missing key treated equal
  assert.equal(resourcesEqual({ a: 1, b: undefined }, { a: 1 }), true)
  assert.equal(resourcesEqual({ a: 1 }, { a: 1, b: undefined }), true)
  assert.equal(resourcesEqual({ a: 1, b: undefined }, { a: 1, b: undefined }), true)
  assert.equal(resourcesEqual({ a: 1, b: 2 }, { a: 1 }), false)
})

test('updateResourceBatch: identical Applied payload keeps list reference', () => {
  const item1: TestResource = { name: 'pod-1', namespace: 'default', version: 1 }
  const item2: TestResource = { name: 'pod-2', namespace: 'default', version: 1 }
  const listRef = shallowRef<TestResource[]>([item1, item2])
  const before = listRef.value

  const getKey = (item: TestResource) => `${item.namespace}/${item.name}`

  updateResourceBatch(
    listRef,
    [
      {
        action: KUBERNETES_ACTION.Applied,
        data: { name: 'pod-1', namespace: 'default', version: 1 }
      }
    ],
    getKey
  )

  // List reference must be completely untouched
  assert.equal(listRef.value, before)
  assert.equal(listRef.value[0], item1)
  assert.equal(listRef.value[1], item2)
})

test('updateResourceBatch: one changed item replaces only that index', () => {
  const item1: TestResource = { name: 'pod-1', namespace: 'default', version: 1 }
  const item2: TestResource = { name: 'pod-2', namespace: 'default', version: 1 }
  const listRef = shallowRef<TestResource[]>([item1, item2])

  const getKey = (item: TestResource) => `${item.namespace}/${item.name}`

  updateResourceBatch(
    listRef,
    [
      {
        action: KUBERNETES_ACTION.Applied,
        data: { name: 'pod-1', namespace: 'default', version: 2 }
      }
    ],
    getKey
  )

  assert.notEqual(listRef.value, [item1, item2])
  assert.equal(listRef.value.length, 2)
  assert.notEqual(listRef.value[0], item1)
  assert.equal(listRef.value[0]?.version, 2)
  // item2 must preserve exact reference equality!
  assert.equal(listRef.value[1], item2)
})

interface TestEvent {
  uid: string
  message: string
}

test('updateResourceBatch: applies updates and inserts in O(1) key lookups', () => {
  const listRef = shallowRef<TestResource[]>([
    { name: 'pod-1', namespace: 'default', version: 1 },
    { name: 'pod-2', namespace: 'default', version: 1 }
  ])

  const getKey = (item: TestResource) => `${item.namespace}/${item.name}`

  updateResourceBatch(
    listRef,
    [
      {
        action: KUBERNETES_ACTION.Applied,
        data: { name: 'pod-1', namespace: 'default', version: 2 }
      },
      {
        action: KUBERNETES_ACTION.Applied,
        data: { name: 'pod-3', namespace: 'default', version: 1 }
      }
    ],
    getKey
  )

  assert.equal(listRef.value.length, 3)
  assert.equal(listRef.value[0]?.version, 2)
  assert.equal(listRef.value[1]?.name, 'pod-2')
  assert.equal(listRef.value[2]?.name, 'pod-3')
})

test('updateResourceBatch: handles deletions correctly without index corruption', () => {
  const listRef = shallowRef<TestResource[]>([
    { name: 'pod-1', namespace: 'default' },
    { name: 'pod-2', namespace: 'default' },
    { name: 'pod-3', namespace: 'default' }
  ])

  const getKey = (item: TestResource) => `${item.namespace}/${item.name}`

  updateResourceBatch(
    listRef,
    [
      {
        action: KUBERNETES_ACTION.Deleted,
        data: { name: 'pod-2', namespace: 'default' }
      }
    ],
    getKey
  )

  assert.equal(listRef.value.length, 2)
  assert.equal(listRef.value[0]?.name, 'pod-1')
  assert.equal(listRef.value[1]?.name, 'pod-3')
})

test('updateResourceBatch: supports custom merge function', () => {
  const listRef = shallowRef<TestResource[]>([{ name: 'pod-1', namespace: 'default', version: 10 }])

  const getKey = (item: TestResource) => `${item.namespace}/${item.name}`
  const merge = (existing: TestResource, incoming: TestResource) => ({
    ...incoming,
    version: (existing.version ?? 0) + (incoming.version ?? 0)
  })

  updateResourceBatch(
    listRef,
    [
      {
        action: KUBERNETES_ACTION.Applied,
        data: { name: 'pod-1', namespace: 'default', version: 5 }
      }
    ],
    getKey,
    merge
  )

  assert.equal(listRef.value[0]?.version, 15)
})

test('updateResourceBatch: enforces maxCapacity by trimming oldest records', () => {
  const listRef = shallowRef<TestEvent[]>([
    { uid: 'ev-1', message: 'First' },
    { uid: 'ev-2', message: 'Second' }
  ])

  const getKey = (item: TestEvent) => item.uid

  // Cap at 3 items
  updateResourceBatch(
    listRef,
    [
      { action: KUBERNETES_ACTION.Applied, data: { uid: 'ev-3', message: 'Third' } },
      { action: KUBERNETES_ACTION.Applied, data: { uid: 'ev-4', message: 'Fourth' } }
    ],
    getKey,
    undefined,
    3
  )

  assert.equal(listRef.value.length, 3)
  assert.equal(listRef.value[0]?.uid, 'ev-2')
  assert.equal(listRef.value[1]?.uid, 'ev-3')
  assert.equal(listRef.value[2]?.uid, 'ev-4')
  assert.equal(MAX_EVENTS_RETAINED, 1000)
})

test('updateResourceBatch: handles deletion followed by re-application in the same batch', () => {
  const listRef = shallowRef<TestResource[]>([
    { name: 'pod-1', namespace: 'default', version: 1 },
    { name: 'pod-2', namespace: 'default', version: 1 }
  ])

  const getKey = (item: TestResource) => `${item.namespace}/${item.name}`

  // Sequence: delete pod-1, then immediately re-apply pod-1 with new version
  updateResourceBatch(
    listRef,
    [
      { action: KUBERNETES_ACTION.Deleted, data: { name: 'pod-1', namespace: 'default' } },
      {
        action: KUBERNETES_ACTION.Applied,
        data: { name: 'pod-1', namespace: 'default', version: 99 }
      }
    ],
    getKey
  )

  assert.equal(listRef.value.length, 2)
  assert.equal(listRef.value[0]?.name, 'pod-1')
  assert.equal(listRef.value[0]?.version, 99)
  assert.equal(listRef.value[1]?.name, 'pod-2')
})

test('updateResourceBatch: handles insertion followed by deletion in the same batch', () => {
  const listRef = shallowRef<TestResource[]>([{ name: 'pod-1', namespace: 'default', version: 1 }])

  const getKey = (item: TestResource) => `${item.namespace}/${item.name}`

  // Sequence: create pod-2, then immediately delete pod-2 in the same batch
  updateResourceBatch(
    listRef,
    [
      {
        action: KUBERNETES_ACTION.Applied,
        data: { name: 'pod-2', namespace: 'default', version: 1 }
      },
      { action: KUBERNETES_ACTION.Deleted, data: { name: 'pod-2', namespace: 'default' } }
    ],
    getKey
  )

  assert.equal(listRef.value.length, 1)
  assert.equal(listRef.value[0]?.name, 'pod-1')
})

test('mergeNodeWatchUpdate preserves existing CPU, memory, and pod metrics when watcher payload has zero metrics', () => {
  const existing = {
    name: 'node-1',
    status: 'Ready',
    role: 'worker',
    version: 'v1.30.0',
    cpuPct: 45.5,
    cpuUsed: '1.82',
    cpuTotal: '4',
    memPct: 62.1,
    memUsed: '9.94',
    memTotal: '16',
    podsCount: 18,
    podsLimit: 110,
    uptime: '5d',
    labels: ['env=prod'],
    labelsMap: { env: 'prod' },
    annotations: {},
    isCordoned: false,
    addresses: [],
    conditions: [],
    taints: [],
    imagesCount: 20
  } as unknown as import('../../types/kubernetes.ts').NodeInfo

  const incoming = {
    name: 'node-1',
    status: 'Ready',
    role: 'worker',
    version: 'v1.30.0',
    cpuPct: 0,
    cpuUsed: '0',
    cpuTotal: '4',
    memPct: 0,
    memUsed: '0',
    memTotal: '16',
    podsCount: 0,
    podsLimit: 110,
    uptime: '5d',
    labels: ['env=prod'],
    labelsMap: { env: 'prod' },
    annotations: {},
    isCordoned: true,
    addresses: [],
    conditions: [],
    taints: [],
    imagesCount: 20
  } as unknown as import('../../types/kubernetes.ts').NodeInfo

  const merged = mergeNodeWatchUpdate(existing, incoming)
  assert.equal(merged.isCordoned, true)
  assert.equal(merged.cpuPct, 45.5)
  assert.equal(merged.cpuUsed, '1.82')
  assert.equal(merged.memPct, 62.1)
  assert.equal(merged.memUsed, '9.94')
  assert.equal(merged.podsCount, 18)
})
