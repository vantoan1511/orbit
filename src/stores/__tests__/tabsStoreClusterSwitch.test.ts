import assert from 'node:assert/strict'
import test, { beforeEach } from 'node:test'
import { createPinia, setActivePinia } from 'pinia'
import { storage } from '../../services/nativeService.ts'
import { getTabsStorageKey } from '../../services/tabStorageService.ts'
import { useTabsStore } from '../tabsStore.ts'

beforeEach(() => {
  setActivePinia(createPinia())
})

test('tabsStore.init resets tabs when switching to a cluster with no persisted state', async () => {
  const storeMap = new Map<string, string>()
  const originalGetData = storage.getData
  const originalSetData = storage.setData

  storage.getData = async (key: string) => storeMap.get(key) ?? null
  storage.setData = async (key: string, data?: string | null) => {
    if (data !== undefined && data !== null) {
      storeMap.set(key, data)
    } else {
      storeMap.delete(key)
    }
  }

  try {
    const store = useTabsStore()

    // 1. Initialize on cluster-a and open some tabs
    await store.init('cluster-a')
    store.openTab({
      route: '/workloads?tab=deployments',
      title: 'Deployments'
    })
    store.openTab({
      route: '/logs?namespace=default&workload=api',
      title: 'Logs: api'
    })

    assert.equal(store.tabs.length, 2)
    assert.equal(store.activeTabId, '/logs?namespace=default&workload=api')

    // 2. Switch to cluster-b where no tabs have ever been saved (storage returns null)
    await store.init('cluster-b')

    // CRITICAL: tabs must be empty and activeTabId must be null! No leak from cluster-a!
    assert.equal(store.tabs.length, 0)
    assert.equal(store.activeTabId, null)

    // Wait 200ms to verify no scheduled saveTimer from cluster-a fires and writes to cluster-b key
    await new Promise((resolve) => setTimeout(resolve, 200))
    const clusterBKey = getTabsStorageKey('cluster-b')
    assert.equal(storeMap.has(clusterBKey), false)
  } finally {
    storage.getData = originalGetData
    storage.setData = originalSetData
  }
})
