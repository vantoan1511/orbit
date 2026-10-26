import assert from 'node:assert/strict'
import test, { beforeEach } from 'node:test'
import { createPinia, setActivePinia } from 'pinia'
import { useKubernetesStore } from '../../stores/kubernetesStore.ts'
import { kubernetesService } from '../../services/kubernetesService.ts'
import {
  isAddClusterModalVisible,
  openAddClusterModal,
  closeAddClusterModal,
  useCluster
} from '../useCluster.ts'

beforeEach(() => {
  setActivePinia(createPinia())
})

test('add cluster modal visibility controls toggle state', () => {
  closeAddClusterModal()
  assert.equal(isAddClusterModalVisible.value, false)

  openAddClusterModal()
  assert.equal(isAddClusterModalVisible.value, true)

  closeAddClusterModal()
  assert.equal(isAddClusterModalVisible.value, false)
})

test('handleAddCluster opens the add cluster modal', () => {
  closeAddClusterModal()
  const { handleAddCluster } = useCluster()
  handleAddCluster()
  assert.equal(isAddClusterModalVisible.value, true)
  closeAddClusterModal()
})

test('activeCluster and lastUpdatedAt evaluate cleanly with active Pinia', () => {
  const store = useKubernetesStore()
  const { activeCluster, lastUpdatedAt } = useCluster()

  assert.equal(activeCluster.value, null)
  assert.equal(lastUpdatedAt.value, null)

  store.setClusters([
    {
      id: 'cluster-alpha',
      name: 'Alpha Cluster',
      status: 'Connected',
      server: 'https://alpha.k8s.local:6443',
      version: 'v1.30.0',
      nodes: 3
    }
  ])
  store.setActiveClusterId('cluster-alpha')

  assert.ok(activeCluster.value)
  assert.equal(activeCluster.value?.id, 'cluster-alpha')
  assert.equal(activeCluster.value?.name, 'Alpha Cluster')
})

test('reconnectActiveCluster dispatches switchCluster with active cluster id', async () => {
  const store = useKubernetesStore()
  store.setActiveClusterId('cluster-xyz')
  let switchedId: string | null = null
  const origSwitch = kubernetesService.switchCluster
  kubernetesService.switchCluster = async (id: string) => {
    switchedId = id
  }
  try {
    const { reconnectActiveCluster } = useCluster()
    await reconnectActiveCluster()
    assert.equal(switchedId, 'cluster-xyz')
  } finally {
    kubernetesService.switchCluster = origSwitch
  }
})
