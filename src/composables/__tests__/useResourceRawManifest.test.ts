import assert from 'node:assert/strict'
import test from 'node:test'
import * as yaml from 'yaml'
import { ref } from 'vue'
import { OrbitEvents, type KubernetesResourceInfo } from '../../types/events.ts'
import { events } from '../../services/nativeService.ts'
import { kubernetesService } from '../../services/kubernetesService.ts'
import { useResourceRawManifest } from '../useResourceRawManifest.ts'

test('resets isYamlLoading to false on OrbitEvents.ErrorOccurred', async () => {
  let requested = false
  kubernetesService.getResourceRaw = async () => {
    requested = true
    return {} as unknown as Record<string, unknown>
  }

  const kind = ref('PersistentVolume')
  const name = ref('pv-test-error')
  const visible = ref(true)

  const manifestComposable = useResourceRawManifest({
    kind,
    name,
    visible
  })

  assert.equal(requested, true)
  assert.equal(manifestComposable.isYamlLoading.value, true)

  // Emit ErrorOccurred event matching resource
  await events.dispatch(OrbitEvents.ErrorOccurred, {
    message: 'Failed to fetch manifest for pv-test-error: 404 Not Found'
  })

  assert.equal(manifestComposable.isYamlLoading.value, false)
  assert.ok(manifestComposable.yamlError.value?.includes('pv-test-error'))

  manifestComposable.cleanup()
})

test('does not refetch on ResourceBatchUpdated when resource is deleted or unrelated', async () => {
  let fetchCount = 0
  kubernetesService.getResourceRaw = async () => {
    fetchCount++
    return {} as unknown as Record<string, unknown>
  }

  const kind = ref('PersistentVolume')
  const name = ref('pv-batch-test')
  const visible = ref(true)

  const manifestComposable = useResourceRawManifest({
    kind,
    name,
    visible
  })

  // Initial fetch was triggered by immediate watch
  assert.equal(fetchCount, 1)

  // 1. Unrelated resource in batch
  await events.dispatch(OrbitEvents.ResourceBatchUpdated, {
    kind: 'PersistentVolume',
    cluster_id: 'default',
    updates: [
      {
        action: 'MODIFIED',
        data: { name: 'pv-other-resource' } as unknown as KubernetesResourceInfo
      }
    ]
  })
  assert.equal(fetchCount, 1, 'Should not refetch for unrelated resource')

  // 2. Matching resource with DELETED action
  await events.dispatch(OrbitEvents.ResourceBatchUpdated, {
    kind: 'PersistentVolume',
    cluster_id: 'default',
    updates: [
      {
        action: 'DELETED',
        data: { name: 'pv-batch-test' } as unknown as KubernetesResourceInfo
      }
    ]
  })
  assert.equal(fetchCount, 1, 'Should not refetch for DELETED action')

  // 3. Matching resource with MODIFIED action
  await events.dispatch(OrbitEvents.ResourceBatchUpdated, {
    kind: 'PersistentVolume',
    cluster_id: 'default',
    updates: [
      {
        action: 'MODIFIED',
        data: { name: 'pv-batch-test' } as unknown as KubernetesResourceInfo
      }
    ]
  })
  assert.equal(fetchCount, 2, 'Should refetch when matching resource is MODIFIED')

  manifestComposable.cleanup()
})

test('generates displayedYaml directly via yaml.stringify from rawManifest', () => {
  const kind = ref('PersistentVolume')
  const name = ref('pv-yaml-test')
  const visible = ref(true)

  const manifestComposable = useResourceRawManifest({
    kind,
    name,
    visible
  })

  const testManifest = {
    apiVersion: 'v1',
    kind: 'PersistentVolume',
    metadata: {
      name: 'pv-yaml-test',
      labels: { env: 'production' }
    },
    spec: {
      capacity: { storage: '100Gi' },
      accessModes: ['ReadWriteOnce']
    }
  }

  manifestComposable.rawManifest.value = testManifest

  const expectedYaml = yaml.stringify(testManifest)
  assert.equal(manifestComposable.displayedYaml.value, expectedYaml)

  manifestComposable.cleanup()
})
