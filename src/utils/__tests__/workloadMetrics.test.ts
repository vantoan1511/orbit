import assert from 'node:assert/strict'
import test from 'node:test'
import type { PodInfo } from '../../types/kubernetes.ts'
import { computeDeploymentRestarts, getTopConsumers } from '../workloadMetrics.ts'

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
