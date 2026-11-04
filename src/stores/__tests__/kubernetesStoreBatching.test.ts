import assert from 'node:assert/strict'
import test, { beforeEach } from 'node:test'
import { createPinia, setActivePinia } from 'pinia'
import { watch } from 'vue'
import { OrbitEvents } from '../../types/events.ts'
import { KUBERNETES_ACTION, KUBERNETES_RESOURCE_KIND } from '../../constants/kubernetes.ts'
import { events } from '../../services/nativeService.ts'
import { useKubernetesStore } from '../kubernetesStore.ts'
import type { DeploymentInfo } from '../../types/kubernetes.ts'

beforeEach(() => {
  setActivePinia(createPinia())
})

test('two ResourceBatchUpdated events in same frame trigger only one store assignment', async () => {
  const store = useKubernetesStore()
  store.setActiveClusterId('test-cluster')

  let deploymentAssignments = 0
  watch(
    () => store.deployments,
    () => {
      deploymentAssignments++
    },
    { flush: 'sync' }
  )

  const dep1: DeploymentInfo = {
    name: 'dep-1',
    namespace: 'default',
    readyReplicas: 1,
    replicas: 1,
    availableReplicas: 1,
    age: '1m',
    conditions: []
  }

  const dep2: DeploymentInfo = {
    name: 'dep-2',
    namespace: 'default',
    readyReplicas: 1,
    replicas: 1,
    availableReplicas: 1,
    age: '1m',
    conditions: []
  }

  // Dispatch two events synchronously (same frame)
  await events.dispatch(OrbitEvents.ResourceBatchUpdated, {
    kind: KUBERNETES_RESOURCE_KIND.Deployment,
    cluster_id: 'test-cluster',
    updates: [{ action: KUBERNETES_ACTION.Applied, data: dep1 }]
  })

  await events.dispatch(OrbitEvents.ResourceBatchUpdated, {
    kind: KUBERNETES_RESOURCE_KIND.Deployment,
    cluster_id: 'test-cluster',
    updates: [{ action: KUBERNETES_ACTION.Applied, data: dep2 }]
  })

  // Flush frame batcher synchronously
  store.frameBatcher.flush()

  // Deployments should have been assigned only once!
  assert.equal(deploymentAssignments, 1)
  assert.equal(store.deployments.length, 2)
  assert.equal(store.deployments[0]?.name, 'dep-1')
  assert.equal(store.deployments[1]?.name, 'dep-2')
})
