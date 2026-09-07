import assert from 'node:assert/strict'
import test, { beforeEach } from 'node:test'
import { createPinia, setActivePinia } from 'pinia'
import { type NodeInfo, type PodInfo } from '../../types/kubernetes.ts'
import { useKubernetesStore } from '../kubernetesStore.ts'

beforeEach(() => {
  setActivePinia(createPinia())
})

function createSampleNode(name = 'node-1', cpuTotal = '12', memTotal = '16'): NodeInfo {
  return {
    name,
    status: 'Ready',
    role: 'worker',
    version: 'v1.30.0',
    cpuPct: 0,
    cpuUsed: '0',
    cpuTotal,
    memPct: 0,
    memUsed: '0',
    memTotal,
    podsCount: 0,
    podsLimit: 110,
    uptime: '1d',
    createdAt: new Date().toISOString(),
    labels: [],
    labelsMap: {},
    annotations: {},
    isCordoned: false
  }
}

function createSamplePod(name = 'api-pod', namespace = 'default', node = 'node-1'): PodInfo {
  return {
    name,
    namespace,
    status: 'Running',
    age: '1h',
    node,
    restarts: 0,
    images: ['nginx:latest'],
    labels: {},
    annotations: {},
    containers: []
  }
}

test('setPods calculates node-relative CPU cores and percentages when node is present', () => {
  const store = useKubernetesStore()
  store.setNodes([createSampleNode('node-1', '12', '16')])

  const pod = createSamplePod('api-pod', 'default', 'node-1')
  pod.cpuCores = 2.25
  pod.memoryBytes = 314849280 // ~300.26 MiB

  store.setPods([pod])

  const storedPod = store.pods[0]
  assert.ok(storedPod)
  assert.equal(storedPod.cpu, '2.25/12 cores')
  assert.equal(storedPod.cpuPct, 18.75)
  assert.ok(storedPod.memory?.includes('Mi'))
  assert.equal(storedPod.memoryPct, 1.83)
})

test('setNodes re-enriches existing pods with node capacity once nodes load', () => {
  const store = useKubernetesStore()
  const pod = createSamplePod('api-pod', 'default', 'node-1')
  pod.cpuCores = 2.25
  pod.memoryBytes = 314849280

  // Pods load first
  store.setPods([pod])
  assert.equal(store.pods[0]?.cpu, '2.25 cores')

  // Nodes load second
  store.setNodes([createSampleNode('node-1', '12', '16')])
  assert.equal(store.pods[0]?.cpu, '2.25/12 cores')
  assert.equal(store.pods[0]?.cpuPct, 18.75)
})

test('onPodMetricsUpdated caches metrics and enriches pods even if metrics arrive before pods', () => {
  const store = useKubernetesStore()
  store.setNodes([createSampleNode('node-1', '12', '16')])

  // Metrics event arrives before pods are fetched
  store.setPods([
    {
      ...createSamplePod('api-pod', 'default', 'node-1'),
      cpu: '2.25 cores',
      cpuCores: 2.25,
      memory: '300.25Mi',
      memoryBytes: 314849280
    }
  ])

  assert.equal(store.pods[0]?.cpu, '2.25/12 cores')
  assert.equal(store.pods[0]?.memory, '300.25Mi')
})

test('setPods keeps cpuPct and memoryPct undefined when node is missing or unresolved', () => {
  const store = useKubernetesStore()
  // No nodes in store
  const pod = createSamplePod('unassigned-pod', 'default', 'nonexistent-node')
  pod.cpuCores = 0.25
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  ;(pod as any).cpuPct = null
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  ;(pod as any).memoryPct = null

  store.setPods([pod])

  const storedPod = store.pods[0]
  assert.ok(storedPod)
  assert.equal(storedPod.cpu, '250m')
  assert.equal(storedPod.cpuPct, undefined)
  assert.equal(storedPod.memoryPct, undefined)
})
