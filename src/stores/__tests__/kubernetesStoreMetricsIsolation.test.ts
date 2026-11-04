import assert from 'node:assert/strict'
import test, { beforeEach } from 'node:test'
import { createPinia, setActivePinia } from 'pinia'
import { useKubernetesStore } from '../kubernetesStore.ts'
import { app, events as nativeEvents } from '../../services/nativeService.ts'
import { VERSION } from '../../version.ts'
import { OrbitEvents } from '../../types/events.ts'
import type { NodeInfo, PodInfo } from '../../types/kubernetes.ts'

function makePod(name: string, namespace: string, nodeName: string): PodInfo {
  return {
    name,
    namespace,
    node: nodeName,
    status: 'Running',
    ready: '1/1',
    restarts: 0,
    age: '10m',
    cpuCores: 0.1,
    memoryBytes: 100 * 1024 * 1024
  }
}

function makeNode(
  name: string,
  cpuTotal = '4',
  memTotal = '16Gi',
  cpuUsed = '10%',
  memUsed = '20%'
): NodeInfo {
  return {
    name,
    status: 'Ready',
    roles: 'worker',
    version: 'v1.30.0',
    age: '10d',
    cpuTotal,
    memTotal,
    cpuUsed,
    memUsed,
    pods: '10/110'
  }
}

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

test('setNodes with changed cpuUsed only leaves pods.value reference unchanged', () => {
  const store = useKubernetesStore()
  store.setActiveClusterId('test-cluster')

  const initialNodes = [makeNode('node-a', '4', '16Gi', '10%', '20%')]
  store.setNodes(initialNodes)

  const podA = makePod('pod-a', 'default', 'node-a')
  store.setPods([podA])

  const podsRefBefore = store.pods

  // Update nodes with same totals, but changed cpuUsed/memUsed
  const updatedNodes = [makeNode('node-a', '4', '16Gi', '35%', '45%')]
  store.setNodes(updatedNodes)

  // Reference to store.pods must remain strictly unchanged
  assert.equal(store.pods, podsRefBefore)
  assert.equal(store.pods[0], podsRefBefore[0])
})

test('setNodes re-enriches only pods on nodes whose cpu/mem totals changed', () => {
  const store = useKubernetesStore()
  store.setActiveClusterId('test-cluster')

  const nodeA = makeNode('node-a', '4', '16Gi')
  const nodeB = makeNode('node-b', '8', '32Gi')
  store.setNodes([nodeA, nodeB])

  const podA = makePod('pod-a', 'default', 'node-a')
  const podB = makePod('pod-b', 'default', 'node-b')
  store.setPods([podA, podB])

  const originalPodA = store.pods[0]
  const originalPodB = store.pods[1]

  // Totals change on node-a only (e.g. upgraded CPU from 4 to 8)
  const nodeAUpdated = makeNode('node-a', '8', '16Gi')
  store.setNodes([nodeAUpdated, nodeB])

  assert.equal(store.pods.length, 2)
  // pod-a must be a newly enriched object
  assert.notEqual(store.pods[0], originalPodA)
  // pod-b on node-b must preserve exact reference equality!
  assert.equal(store.pods[1], originalPodB)
})
