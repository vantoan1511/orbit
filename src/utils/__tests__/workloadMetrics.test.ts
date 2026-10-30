import assert from 'node:assert/strict'
import test from 'node:test'
import type { PodInfo } from '../../types/kubernetes.ts'
import {
  REPLICAS_SORT_MULTIPLIER,
  computeDeploymentRestarts,
  computeReplicasSortKey,
  getTopConsumers,
  withReplicasSortKey
} from '../workloadMetrics.ts'

function createSamplePod(
  name: string,
  namespace = 'default',
  restarts = 0,
  controlledBy?: string,
  cpuPct = 0,
  memoryPct = 0
): PodInfo {
  return {
    name,
    namespace,
    status: 'Running',
    age: '1h',
    restarts,
    controlledBy,
    cpuPct,
    memoryPct,
    images: [],
    labels: {},
    annotations: {},
    containers: []
  }
}

test('computeDeploymentRestarts sums restarts per deployment for ReplicaSet pods', () => {
  const pods: PodInfo[] = [
    createSamplePod('web-rs1-abc', 'prod', 2, 'ReplicaSet/web-rs1'),
    createSamplePod('web-rs1-def', 'prod', 3, 'ReplicaSet/web-rs1'),
    createSamplePod('api-rs2-xyz', 'prod', 1, 'ReplicaSet/api-rs2'),
    createSamplePod('daemon-pod', 'prod', 5, 'DaemonSet/daemon-ds'), // should be ignored
    createSamplePod('web-other-ns', 'staging', 4, 'ReplicaSet/web-other')
  ]

  const restartMap = computeDeploymentRestarts(pods)

  assert.equal(restartMap.get('prod/web'), 5)
  assert.equal(restartMap.get('prod/api'), 1)
  assert.equal(restartMap.get('staging/web'), 4)
  assert.equal(restartMap.get('prod/daemon'), undefined)
})

test('getTopConsumers returns top-N pods sorted descending by metric', () => {
  const pods: PodInfo[] = [
    createSamplePod('pod-low', 'default', 0, undefined, 5, 20),
    createSamplePod('pod-high', 'default', 10, undefined, 90, 10),
    createSamplePod('pod-mid', 'default', 2, undefined, 45, 80)
  ]

  const topCpu = getTopConsumers(pods, 'cpu', 2)
  assert.equal(topCpu.length, 2)
  assert.equal(topCpu[0]?.pod, 'pod-high')
  assert.equal(topCpu[0]?.pct, 90)
  assert.equal(topCpu[1]?.pod, 'pod-mid')

  const topMemory = getTopConsumers(pods, 'memory', 1)
  assert.equal(topMemory.length, 1)
  assert.equal(topMemory[0]?.pod, 'pod-mid')
  assert.equal(topMemory[0]?.pct, 80)

  const topRestarts = getTopConsumers(pods, 'restarts', 2)
  assert.equal(topRestarts.length, 2)
  assert.equal(topRestarts[0]?.pod, 'pod-high')
  assert.equal(topRestarts[0]?.value, '10')
})

test('computeReplicasSortKey orders primarily by desired replicas and breaks ties by current replicas', () => {
  const workloads = [
    { name: 'healthy-three', replicas: { current: 3, desired: 3 } },
    { name: 'scaled-zero', replicas: { current: 0, desired: 0 } },
    { name: 'pending-five', replicas: { current: 0, desired: 5 } },
    { name: 'crashing-one', replicas: { current: 0, desired: 1 } },
    { name: 'degraded-three-a', replicas: { current: 1, desired: 3 } },
    { name: 'surge-three', replicas: { current: 4, desired: 3 } },
    { name: 'healthy-one', replicas: { current: 1, desired: 1 } },
    { name: 'degraded-three-b', replicas: { current: 2, desired: 3 } }
  ]

  const mapped = workloads.map((w) => ({
    name: w.name,
    key: computeReplicasSortKey(w.replicas)
  }))

  const asc = [...mapped].sort((a, b) => a.key - b.key).map((w) => w.name)
  assert.deepEqual(asc, [
    'scaled-zero',
    'crashing-one',
    'healthy-one',
    'degraded-three-a',
    'degraded-three-b',
    'healthy-three',
    'surge-three',
    'pending-five'
  ])

  const desc = [...mapped].sort((a, b) => b.key - a.key).map((w) => w.name)
  assert.deepEqual(desc, [
    'pending-five',
    'surge-three',
    'healthy-three',
    'degraded-three-b',
    'degraded-three-a',
    'healthy-one',
    'crashing-one',
    'scaled-zero'
  ])
})

test('computeReplicasSortKey handles nullish, negative, NaN, and high replica bounds safely', () => {
  assert.equal(computeReplicasSortKey(undefined), 0)
  assert.equal(computeReplicasSortKey(null), 0)

  // Negative / NaN values clamp to 0
  assert.equal(computeReplicasSortKey({ current: -2, desired: -1 }), 0)
  assert.equal(
    computeReplicasSortKey({ current: Number.NaN, desired: 2 }),
    2 * REPLICAS_SORT_MULTIPLIER
  )

  // Kubernetes i32 max desired produces a safe integer
  const maxI32Key = computeReplicasSortKey({ current: 500, desired: 2_147_483_647 })
  assert.equal(Number.isSafeInteger(maxI32Key), true)
  assert.equal(maxI32Key, 2_147_483_647 * REPLICAS_SORT_MULTIPLIER + 500)
})

test('withReplicasSortKey enriches items without mutating original workload records', () => {
  const original = Object.freeze([
    Object.freeze({ id: 'dep-1', replicas: { current: 1, desired: 2 } }),
    Object.freeze({ id: 'dep-2', replicas: null })
  ])

  const enriched = withReplicasSortKey(original)

  assert.notEqual(enriched, original)
  assert.equal(enriched.length, 2)
  assert.equal(enriched[0].id, 'dep-1')
  assert.equal(enriched[0].replicasSortKey, 2 * REPLICAS_SORT_MULTIPLIER + 1)
  assert.equal(enriched[1].id, 'dep-2')
  assert.equal(enriched[1].replicasSortKey, 0)
  assert.equal('replicasSortKey' in original[0], false)
})
