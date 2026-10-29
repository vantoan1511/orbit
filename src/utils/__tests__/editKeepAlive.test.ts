import assert from 'node:assert/strict'
import test from 'node:test'
import type { AppTab } from '../../types/tabs.ts'
import { createEditHostRegistry, editTabCacheName, getEditCacheNames } from '../editKeepAlive.ts'

test('editTabCacheName is stable across calls for the same tab id', () => {
  const name1 = editTabCacheName('/workloads/Deployment/default/nginx/edit')
  const name2 = editTabCacheName('/workloads/Deployment/default/nginx/edit')
  assert.equal(name1, name2)
  assert.ok(name1.length > 0)
})

test('getEditCacheNames returns two distinct names for two edit tabs', () => {
  const tabs: AppTab[] = [
    {
      id: '/workloads/Deployment/ns-a/web/edit',
      title: 'Edit web',
      route: '/workloads/Deployment/ns-a/web/edit',
      path: '/workloads/Deployment/ns-a/web/edit'
    },
    {
      id: '/workloads/Deployment/ns-b/api/edit',
      title: 'Edit api',
      route: '/workloads/Deployment/ns-b/api/edit',
      path: '/workloads/Deployment/ns-b/api/edit'
    }
  ]

  const cacheNames = getEditCacheNames(tabs)
  assert.equal(cacheNames.length, 2)
  assert.equal(cacheNames[0], editTabCacheName(tabs[0].id))
  assert.equal(cacheNames[1], editTabCacheName(tabs[1].id))
  assert.notEqual(cacheNames[0], cacheNames[1])
})

test('getEditCacheNames excludes non-edit tabs', () => {
  const tabs: AppTab[] = [
    {
      id: '/workloads',
      title: 'Workloads',
      route: '/workloads',
      path: '/workloads'
    },
    {
      id: '/logs?pod=xyz',
      title: 'Logs',
      route: '/logs?pod=xyz',
      path: '/logs'
    },
    {
      id: '/workloads/Deployment/default/nginx/edit',
      title: 'Edit nginx',
      route: '/workloads/Deployment/default/nginx/edit',
      path: '/workloads/Deployment/default/nginx/edit'
    }
  ]

  const cacheNames = getEditCacheNames(tabs)
  assert.equal(cacheNames.length, 1)
  assert.equal(cacheNames[0], editTabCacheName(tabs[2].id))
})

test('removing a tab from the list removes its name (eviction driver)', () => {
  const tab1: AppTab = {
    id: '/workloads/Deployment/default/web/edit',
    title: 'Edit web',
    route: '/workloads/Deployment/default/web/edit',
    path: '/workloads/Deployment/default/web/edit'
  }
  const tab2: AppTab = {
    id: '/workloads/Deployment/default/api/edit',
    title: 'Edit api',
    route: '/workloads/Deployment/default/api/edit',
    path: '/workloads/Deployment/default/api/edit'
  }

  const initial = getEditCacheNames([tab1, tab2])
  assert.deepEqual(initial, [editTabCacheName(tab1.id), editTabCacheName(tab2.id)])

  // Remove tab1 (simulates user closing tab1)
  const afterClose = getEditCacheNames([tab2])
  assert.deepEqual(afterClose, [editTabCacheName(tab2.id)])
  assert.ok(!afterClose.includes(editTabCacheName(tab1.id)))
})

test('createEditHostRegistry manages per-tab wrapper component lifecycle', () => {
  const registry = createEditHostRegistry()
  const DummyBase = { name: 'DummyBase' }

  const tabId1 = '/workloads/Deployment/default/web/edit'
  const tabId2 = '/workloads/Deployment/default/api/edit'

  const host1a = registry.getOrCreateEditHost(tabId1, DummyBase)
  const host1b = registry.getOrCreateEditHost(tabId1, DummyBase)
  const host2 = registry.getOrCreateEditHost(tabId2, DummyBase)

  // Same component for same tab id
  assert.equal(host1a, host1b)
  // Different component for different tab id
  assert.notEqual(host1a, host2)
  // Names match editTabCacheName
  assert.equal(host1a.name, editTabCacheName(tabId1))
  assert.equal(host2.name, editTabCacheName(tabId2))

  // Pruning
  assert.equal(registry.hasEditHost(tabId1), true)
  assert.equal(registry.hasEditHost(tabId2), true)

  registry.pruneEditHosts([tabId2])
  assert.equal(registry.hasEditHost(tabId1), false)
  assert.equal(registry.hasEditHost(tabId2), true)
})
