import assert from 'node:assert/strict'
import test, { beforeEach } from 'node:test'
import { createPinia, setActivePinia } from 'pinia'
import { useKubernetesStore } from '../kubernetesStore.ts'
import { app, events as nativeEvents } from '../../services/nativeService.ts'
import { VERSION } from '../../version.ts'
import { OrbitEvents } from '../../types/events.ts'
import type { PodInfo } from '../../types/kubernetes.ts'

beforeEach(() => {
  setActivePinia(createPinia())
})

test('app.getConfig returns current VERSION rather than outdated static version', async () => {
  const config = await app.getConfig()
  assert.equal(config.version, VERSION)
})

test('setActiveClusterId clears cached pod metrics avoiding cross-cluster metrics leak', async () => {
  const store = useKubernetesStore()

  // 1. Set active cluster to cluster-1
  store.setActiveClusterId('cluster-1')

  // 2. Dispatch pod metrics event for cluster-1
  await nativeEvents.dispatch(OrbitEvents.PodMetricsUpdated, {
    cluster_id: 'cluster-1',
    metrics: [
      {
        name: 'coredns-1',
        namespace: 'kube-system',
        cpu: '15m',
        memory: '30Mi',
        cpuCores: 0.015,
        memoryBytes: 31457280
      }
    ]
  })

  // Verify that a pod added in cluster-1 is enriched with cached metrics
  const podCluster1: PodInfo = {
    name: 'coredns-1',
    namespace: 'kube-system',
    status: 'Running',
    ready: '1/1',
    restarts: 0,
    age: '1d'
  }
  store.setPods([podCluster1])
  assert.equal(store.pods[0]?.cpu, '15m')
  assert.equal(store.pods[0]?.memory, '30Mi')

  // 3. Switch to cluster-2
  store.setActiveClusterId('cluster-2')

  // 4. In cluster-2, add a pod with the same name and namespace but no metrics yet
  const podCluster2: PodInfo = {
    name: 'coredns-1',
    namespace: 'kube-system',
    status: 'Running',
    ready: '1/1',
    restarts: 0,
    age: '2m'
  }
  store.setPods([podCluster2])

  // It should NOT be enriched with cluster-1's cached metrics!
  assert.equal(store.pods[0]?.cpu, undefined)
  assert.equal(store.pods[0]?.memory, undefined)
})
