import assert from 'node:assert/strict'
import test, { beforeEach } from 'node:test'
import { createPinia, setActivePinia } from 'pinia'
import { useKubernetesStore } from '../kubernetesStore.ts'
import { useLogsStore } from '../logsStore.ts'
import type { PodInfo, ActivePortForward } from '@/types/kubernetes.ts'

beforeEach(() => {
  setActivePinia(createPinia())
})

test('setActiveClusterId thoroughly clears all resources including pods and port forwards', async () => {
  const store = useKubernetesStore()
  const logsStore = useLogsStore()

  // 1. Seed initial data simulating an active cluster with workloads and pods
  store.setActiveClusterId('cluster-1')

  const samplePod: PodInfo = {
    name: 'nginx-pod-1',
    namespace: 'default',
    status: 'Running',
    ready: '1/1',
    restarts: 0,
    age: '5m',
    cpu: '10m',
    memory: '20Mi'
  }
  store.setPods([samplePod])
  assert.equal(store.pods.length, 1)

  const samplePortForward: ActivePortForward = {
    id: 'pf-1',
    clusterId: 'cluster-1',
    resourceType: 'pod',
    namespace: 'default',
    resourceName: 'nginx-pod-1',
    targetPort: 80,
    localPort: 8080,
    status: 'Active',
    createdAt: Date.now()
  }
  store.activePortForwards.push(samplePortForward)
  assert.equal(store.activePortForwards.length, 1)

  store.cpuHistory = [10, 20, 30, 40, 50, 60, 70]
  store.memHistory = [15, 25, 35, 45, 55, 65, 75]
  store.lastUpdatedAt = new Date()

  // 2. Switch to cluster-2
  store.setActiveClusterId('cluster-2')

  // Verify activeClusterId changed
  assert.equal(store.activeClusterId, 'cluster-2')

  // Verify pods are cleared (preventing leak from cluster-1)
  assert.equal(store.pods.length, 0)

  // Verify active port forwards are cleared
  assert.equal(store.activePortForwards.length, 0)

  // Verify CPU & memory histories are reset
  assert.deepEqual(store.cpuHistory, [0, 0, 0, 0, 0, 0, 0])
  assert.deepEqual(store.memHistory, [0, 0, 0, 0, 0, 0, 0])

  // Verify lastUpdatedAt is reset
  assert.equal(store.lastUpdatedAt, null)

  // Verify namespacesLoading is reset to true
  assert.equal(store.namespacesLoading, true)

  // Verify logsStore is updated with the new clusterId
  assert.equal(logsStore.activeClusterId, 'cluster-2')
})
