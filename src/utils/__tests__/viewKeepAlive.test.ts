import assert from 'node:assert/strict'
import test from 'node:test'
import type { AppTab } from '../../types/tabs.ts'
import {
  createViewHostRegistry,
  getViewCacheNames,
  resolveViewKey,
  viewCacheKey,
  viewTabCacheName
} from '../viewKeepAlive.ts'

test('viewCacheKey strips query and hash to return path-only', () => {
  assert.equal(viewCacheKey('/workloads'), '/workloads')
  assert.equal(viewCacheKey('/workloads?tab=pods'), '/workloads')
  assert.equal(viewCacheKey('/workloads#section'), '/workloads')
  assert.equal(viewCacheKey('/workloads?tab=pods#hash'), '/workloads')
})

test('viewTabCacheName is stable and sanitized for the same key', () => {
  const name1 = viewTabCacheName('/workloads')
  const name2 = viewTabCacheName('/workloads')
  assert.equal(name1, name2)
  assert.equal(name1, 'ViewTab__workloads')
})

test('getViewCacheNames returns distinct names for unique paths across tabs', () => {
  const tabs: AppTab[] = [
    { id: '1', title: 'Workloads', route: '/workloads', path: '/workloads' },
    { id: '2', title: 'Workloads Pods', route: '/workloads?tab=pods', path: '/workloads' },
    { id: '3', title: 'Network', route: '/network', path: '/network' }
  ]

  const cacheNames = getViewCacheNames(tabs)
  assert.equal(cacheNames.length, 2)
  assert.ok(cacheNames.includes(viewTabCacheName('/workloads')))
  assert.ok(cacheNames.includes(viewTabCacheName('/network')))
})

test('getViewCacheNames excludes edit routes', () => {
  const tabs: AppTab[] = [
    { id: '1', title: 'Workloads', route: '/workloads', path: '/workloads' },
    {
      id: '/workloads/Deployment/default/nginx/edit',
      title: 'Edit nginx',
      route: '/workloads/Deployment/default/nginx/edit',
      path: '/workloads/Deployment/default/nginx/edit'
    }
  ]

  const cacheNames = getViewCacheNames(tabs)
  assert.equal(cacheNames.length, 1)
  assert.equal(cacheNames[0], viewTabCacheName('/workloads'))
})

test('createViewHostRegistry manages host component lifecycle', () => {
  const registry = createViewHostRegistry()
  const DummyBase = { name: 'DummyBase' }

  const key1 = '/workloads'
  const key2 = '/network'

  const host1a = registry.getOrCreateHost(key1, DummyBase)
  const host1b = registry.getOrCreateHost(key1, DummyBase)
  const host2 = registry.getOrCreateHost(key2, DummyBase)

  // Same component for same key
  assert.equal(host1a, host1b)
  assert.notEqual(host1a, host2)

  // Host name matches viewTabCacheName
  assert.equal(host1a.name, viewTabCacheName(key1))
  assert.equal(host2.name, viewTabCacheName(key2))

  // Pruning
  assert.equal(registry.hasHost(key1), true)
  assert.equal(registry.hasHost(key2), true)

  registry.prune([key2])
  assert.equal(registry.hasHost(key1), false)
  assert.equal(registry.hasHost(key2), true)

  // Clear all
  registry.clearAll()
  assert.equal(registry.hasHost(key2), false)
})

test('resolveViewKey returns fullPath for edit routes and path otherwise', () => {
  const editPath = '/workloads/Deployment/default/nginx/edit'
  const editFullPath = '/workloads/Deployment/default/nginx/edit?from=list'
  assert.equal(resolveViewKey(editPath, editFullPath), editFullPath)

  const normalPath = '/workloads'
  const normalFullPath = '/workloads?tab=deployments'
  assert.equal(resolveViewKey(normalPath, normalFullPath), normalPath)
})
