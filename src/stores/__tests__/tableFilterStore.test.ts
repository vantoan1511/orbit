import assert from 'node:assert/strict'
import test, { beforeEach } from 'node:test'
import { createPinia, setActivePinia } from 'pinia'
import { tableFilterService } from '../../services/tableFilterService.ts'
import { useTableFilterStore } from '../tableFilterStore.ts'

beforeEach(() => {
  setActivePinia(createPinia())
})

test('tableFilterStore initializes with default rows and state', () => {
  const store = useTableFilterStore()
  assert.equal(store.defaultRows, 25)
  assert.equal(store.isInitialized, false)

  const podFilters = store.getFilters('pod')
  assert.equal(podFilters.searchQuery, '')
  assert.deepEqual(podFilters.selectedNamespace, [])
  assert.equal(podFilters.selectedStatus, 'All Statuses')
  assert.equal(podFilters.rows, 25)
  assert.deepEqual(podFilters.columns, [])
  assert.deepEqual(podFilters.extraFilters, {})
})

test('tableFilterStore sets filters correctly', () => {
  const store = useTableFilterStore()

  store.setFilter('deployment', 'searchQuery', 'nginx')
  assert.equal(store.getFilters('deployment').searchQuery, 'nginx')

  store.setFilter('deployment', 'selectedStatus', 'Running')
  assert.equal(store.getFilters('deployment').selectedStatus, 'Running')

  store.setFilter('deployment', 'selectedNamespace', ['default', 'kube-system'])
  assert.deepEqual(store.getFilters('deployment').selectedNamespace, ['default', 'kube-system'])

  store.setFilter('deployment', 'rows', 50)
  assert.equal(store.getFilters('deployment').rows, 50)
  assert.equal(store.defaultRows, 50)

  store.setFilter('deployment', 'columns', [{ field: 'name', header: 'Name', visible: false }])
  assert.deepEqual(store.getFilters('deployment').columns, [
    { field: 'name', header: 'Name', visible: false }
  ])
})

test('tableFilterStore setExtraFilter and getExtraFilter work as expected', () => {
  const store = useTableFilterStore()

  assert.equal(store.getExtraFilter('persistentvolume', 'storageClass', 'All'), 'All')

  store.setExtraFilter('persistentvolume', 'storageClass', 'standard')
  assert.equal(store.getExtraFilter('persistentvolume', 'storageClass', 'All'), 'standard')
})

test('tableFilterStore isolates table filters by clusterId', () => {
  const store = useTableFilterStore()

  store.setActiveClusterId('cluster-a')
  store.setFilter('pod', 'searchQuery', 'redis')
  assert.equal(store.getFilters('pod').searchQuery, 'redis')

  store.setActiveClusterId('cluster-b')
  assert.equal(store.getFilters('pod').searchQuery, '')

  store.setFilter('pod', 'searchQuery', 'postgres')
  assert.equal(store.getFilters('pod').searchQuery, 'postgres')

  // Switch back to cluster-a
  store.setActiveClusterId('cluster-a')
  assert.equal(store.getFilters('pod').searchQuery, 'redis')
})

test('tableFilterStore resetFilters restores defaults', () => {
  const store = useTableFilterStore()

  store.setFilter('service', 'searchQuery', 'api')
  store.setFilter('service', 'selectedNamespace', ['prod'])
  store.setExtraFilter('service', 'type', 'LoadBalancer')

  store.resetFilters('service')

  const reset = store.getFilters('service')
  assert.equal(reset.searchQuery, '')
  assert.deepEqual(reset.selectedNamespace, [])
  assert.deepEqual(reset.extraFilters, {})
})

test('tableFilterStore.init hydrates stored data and updates cached table filters in place', async () => {
  const originalLoadFilters = tableFilterService.loadFilters
  const mockFilters = {
    version: 1,
    defaultRows: 100,
    tables: {
      deployment: {
        rows: 100,
        columns: [{ field: 'name', header: 'Name', visible: true }]
      }
    },
    clusters: {
      default: {
        deployment: {
          searchQuery: 'hydrated-app',
          selectedNamespace: ['kube-system'],
          selectedStatus: 'Healthy'
        }
      }
    }
  }

  tableFilterService.loadFilters = async () => mockFilters

  try {
    const store = useTableFilterStore()

    // Access deployment filter BEFORE init to test in-place reactive mutation
    const preInitDeployment = store.getFilters('deployment')
    assert.equal(preInitDeployment.searchQuery, '')
    assert.equal(preInitDeployment.rows, 25)

    await store.init()

    assert.equal(store.isInitialized, true)
    assert.equal(store.defaultRows, 100)

    // Verify existing reference was mutated in place
    assert.equal(preInitDeployment.searchQuery, 'hydrated-app')
    assert.deepEqual(preInitDeployment.selectedNamespace, ['kube-system'])
    assert.equal(preInitDeployment.selectedStatus, 'Healthy')
    assert.equal(preInitDeployment.rows, 100)
    assert.deepEqual(preInitDeployment.columns, [{ field: 'name', header: 'Name', visible: true }])
  } finally {
    tableFilterService.loadFilters = originalLoadFilters
  }
})
