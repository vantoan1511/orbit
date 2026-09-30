import assert from 'node:assert/strict'
import test, { beforeEach } from 'node:test'
import { createPinia, setActivePinia } from 'pinia'
import { storage } from '../../services/nativeService.ts'
import { BASE_LOGS_STORAGE_KEY, getLogsStorageKey, useLogsStore } from '../logsStore.ts'

beforeEach(() => {
  setActivePinia(createPinia())
})

test('getLogsStorageKey returns base key when no clusterId is provided', () => {
  assert.equal(getLogsStorageKey(), BASE_LOGS_STORAGE_KEY)
  assert.equal(getLogsStorageKey(null), BASE_LOGS_STORAGE_KEY)
  assert.equal(getLogsStorageKey(''), BASE_LOGS_STORAGE_KEY)
  assert.equal(getLogsStorageKey('   '), BASE_LOGS_STORAGE_KEY)
})

test('getLogsStorageKey isolates by clusterId when provided', () => {
  assert.equal(getLogsStorageKey('cluster-a'), 'orbit_logs_recent_logs_cluster-a')
  assert.equal(getLogsStorageKey('prod-us-east-1'), 'orbit_logs_recent_logs_prod-us-east-1')
})

test('logsStore initializes with empty recentLogs and null activeClusterId', () => {
  const store = useLogsStore()
  assert.deepEqual(store.recentLogs, [])
  assert.equal(store.activeClusterId, null)
})

test('logsStore isolates recent logs across clusters and prevents cross-cluster leak', async () => {
  const storeMap = new Map<string, string>()
  const originalGetData = storage.getData
  const originalSetData = storage.setData

  storage.getData = async (key: string) => {
    return storeMap.get(key) ?? null
  }
  storage.setData = async (key: string, data?: string | null) => {
    if (data !== undefined && data !== null) {
      storeMap.set(key, data)
    } else {
      storeMap.delete(key)
    }
  }

  try {
    const store = useLogsStore()

    // 1. Switch to cluster-a and add a log entry
    await store.setClusterId('cluster-a')
    assert.equal(store.activeClusterId, 'cluster-a')

    await store.addRecentLog({
      namespace: 'prod',
      workloadKind: 'Deployment',
      workloadName: 'frontend'
    })

    assert.equal(store.recentLogs.length, 1)
    assert.equal(store.recentLogs[0]?.workloadName, 'frontend')
    assert.equal(store.recentLogs[0]?.clusterId, 'cluster-a')

    // Verify written to cluster-a storage key
    const clusterAKey = getLogsStorageKey('cluster-a')
    assert.equal(storeMap.has(clusterAKey), true)
    const storedClusterA = JSON.parse(storeMap.get(clusterAKey)!)
    assert.equal(storedClusterA[0].workloadName, 'frontend')

    // 2. Switch to cluster-b: recentLogs must be cleared and empty because cluster-b has no logs
    await store.setClusterId('cluster-b')
    assert.equal(store.activeClusterId, 'cluster-b')
    assert.equal(store.recentLogs.length, 0) // No leak from cluster-a!

    // Add a log entry for cluster-b
    await store.addRecentLog({
      namespace: 'staging',
      workloadKind: 'StatefulSet',
      workloadName: 'database'
    })

    assert.equal(store.recentLogs.length, 1)
    assert.equal(store.recentLogs[0]?.workloadName, 'database')
    assert.equal(store.recentLogs[0]?.clusterId, 'cluster-b')

    // 3. Switch back to cluster-a: cluster-a logs must be restored without cluster-b logs
    await store.setClusterId('cluster-a')
    assert.equal(store.recentLogs.length, 1)
    assert.equal(store.recentLogs[0]?.workloadName, 'frontend')
    assert.equal(store.recentLogs[0]?.namespace, 'prod')

    // 4. Clear recent logs on cluster-a: cluster-b remains intact
    await store.clearRecentLogs()
    assert.equal(store.recentLogs.length, 0)
    assert.deepEqual(JSON.parse(storeMap.get(clusterAKey)!), [])

    // Switch back to cluster-b and confirm database log still exists
    await store.setClusterId('cluster-b')
    assert.equal(store.recentLogs.length, 1)
    assert.equal(store.recentLogs[0]?.workloadName, 'database')
  } finally {
    storage.getData = originalGetData
    storage.setData = originalSetData
  }
})
