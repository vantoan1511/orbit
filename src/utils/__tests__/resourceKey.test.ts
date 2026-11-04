import assert from 'node:assert/strict'
import test from 'node:test'
import { getResourceCompositeKey, shouldShowSkeleton } from '../resourceKey.ts'

test('getResourceCompositeKey: prefers uid when present', () => {
  const item = {
    uid: 'u-12345',
    namespace: 'prod',
    name: 'web-deployment'
  }
  assert.equal(getResourceCompositeKey(item), 'u-12345')
})

test('getResourceCompositeKey: falls back to ns/name when uid is missing', () => {
  const item = {
    namespace: 'prod',
    name: 'web-deployment'
  }
  assert.equal(getResourceCompositeKey(item), 'prod/web-deployment')
})

test('getResourceCompositeKey: handles cluster-scoped resources with /name', () => {
  const item = {
    name: 'node-worker-1'
  }
  assert.equal(getResourceCompositeKey(item), '/node-worker-1')

  const itemWithEmptyNs = {
    namespace: '',
    name: 'cluster-role-1'
  }
  assert.equal(getResourceCompositeKey(itemWithEmptyNs), '/cluster-role-1')
})

test('shouldShowSkeleton: returns true only when loading and data length is 0', () => {
  assert.equal(shouldShowSkeleton(true, 0), true)
  assert.equal(shouldShowSkeleton(true, 10), false)
  assert.equal(shouldShowSkeleton(false, 0), false)
  assert.equal(shouldShowSkeleton(false, 10), false)
})
