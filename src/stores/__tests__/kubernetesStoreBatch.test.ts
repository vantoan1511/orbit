import assert from 'node:assert/strict'
import test from 'node:test'
import { shallowRef } from 'vue'
import { KUBERNETES_ACTION } from '../../constants/kubernetes.ts'
import { MAX_EVENTS_RETAINED, updateResourceBatch } from '../kubernetesStore.ts'

interface TestResource {
  name: string
  namespace: string
  version?: number
}

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
