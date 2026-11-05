import test from 'node:test'
import assert from 'node:assert/strict'
import {
  parseCpuQuantityToCores,
  parseMemoryQuantityToBytes,
  extractNamespaceResourceQuota,
  extractNamespaceLimitRanges,
  aggregateNamespacePodMetrics
} from '../namespaceResources.ts'
import type { PolicyInfo, PodInfo, NodeInfo } from '../../types/kubernetes.ts'

test('parseCpuQuantityToCores correctly parses various CPU formats', () => {
  assert.equal(parseCpuQuantityToCores('500m'), 0.5)
  assert.equal(parseCpuQuantityToCores('1500m'), 1.5)
  assert.equal(parseCpuQuantityToCores('2'), 2)
  assert.equal(parseCpuQuantityToCores('0.25'), 0.25)
  assert.equal(parseCpuQuantityToCores('0'), 0)
  assert.equal(parseCpuQuantityToCores(''), 0)
  assert.equal(parseCpuQuantityToCores('invalid'), 0)
})

test('parseMemoryQuantityToBytes correctly parses binary and decimal memory suffixes', () => {
  assert.equal(parseMemoryQuantityToBytes('1Ki'), 1024)
  assert.equal(parseMemoryQuantityToBytes('2Mi'), 2 * 1024 * 1024)
  assert.equal(parseMemoryQuantityToBytes('1Gi'), 1024 * 1024 * 1024)
  assert.equal(parseMemoryQuantityToBytes('1K'), 1000)
  assert.equal(parseMemoryQuantityToBytes('1M'), 1000 * 1000)
  assert.equal(parseMemoryQuantityToBytes('1G'), 1000 * 1000 * 1000)
  assert.equal(parseMemoryQuantityToBytes('2048'), 2048)
  assert.equal(parseMemoryQuantityToBytes(''), 0)
  assert.equal(parseMemoryQuantityToBytes('invalid'), 0)
})

test('extractNamespaceResourceQuota parses quota from policy rules', () => {
  const policies: PolicyInfo[] = [
    {
      uid: 'quota-1',
      name: 'compute-resources',
      type: 'Resource Quota',
      scope: 'Namespaced',
      namespace: 'production',
      status: 'Enforced',
      mode: 'enforce',
      violations: 0,
      lastUpdated: '1d',
      description: 'Resource limits',
      rules: `
hard:
  requests.cpu: '2'
  requests.memory: 4Gi
  limits.cpu: '4'
  limits.memory: 8Gi
used:
  requests.cpu: 500m
  requests.memory: 1Gi
  limits.cpu: '1'
  limits.memory: 2Gi
`,
      kind: 'ResourceQuota'
    },
    {
      uid: 'quota-2',
      name: 'other-ns-quota',
      type: 'Resource Quota',
      scope: 'Namespaced',
      namespace: 'staging',
      status: 'Enforced',
      mode: 'enforce',
      violations: 0,
      lastUpdated: '1d',
      description: 'Resource limits',
      rules: '',
      kind: 'ResourceQuota'
    }
  ]

  const quota = extractNamespaceResourceQuota(policies, 'production')
  assert.ok(quota)
  assert.equal(quota.cpuLimit, '4')
  assert.equal(quota.cpuUsed, '1')
  assert.equal(quota.cpuPercent, 25)
  assert.equal(quota.memoryLimit, '8Gi')
  assert.equal(quota.memoryUsed, '2Gi')
  assert.equal(quota.memoryPercent, 25)

  // Non-existent namespace returns null
  assert.equal(extractNamespaceResourceQuota(policies, 'development'), null)
})

test('extractNamespaceLimitRanges parses Container and Pod limits', () => {
  const policies: PolicyInfo[] = [
    {
      uid: 'lr-1',
      name: 'core-limits',
      type: 'Limit Range',
      scope: 'Namespaced',
      namespace: 'production',
      status: 'Enforced',
      mode: 'enforce',
      violations: 0,
      lastUpdated: '1d',
      description: 'Limits',
      rules: `
limits:
  - type: Container
    min:
      cpu: 100m
      memory: 128Mi
    max:
      cpu: '2'
      memory: 4Gi
    default:
      cpu: 500m
      memory: 512Mi
    defaultRequest:
      cpu: 200m
      memory: 256Mi
`,
      kind: 'LimitRange'
    }
  ]

  const ranges = extractNamespaceLimitRanges(policies, 'production')
  assert.equal(ranges.length, 2)
  const cpuRange = ranges.find((r) => r.resource === 'cpu')
  assert.ok(cpuRange)
  assert.equal(cpuRange.type, 'Container')
  assert.equal(cpuRange.min, '100m')
  assert.equal(cpuRange.max, '2')
  assert.equal(cpuRange.default, '500m')
  assert.equal(cpuRange.defaultRequest, '200m')

  const memRange = ranges.find((r) => r.resource === 'memory')
  assert.ok(memRange)
  assert.equal(memRange.min, '128Mi')
  assert.equal(memRange.max, '4Gi')

  // Empty for other namespace
  assert.deepEqual(extractNamespaceLimitRanges(policies, 'default'), [])
})

test('aggregateNamespacePodMetrics aggregates pod metrics against quota limits', () => {
  const pods: PodInfo[] = [
    {
      name: 'api-1',
      namespace: 'production',
      status: 'Running',
      age: '1d',
      cpuCores: 0.5,
      memoryBytes: 512 * 1024 * 1024
    },
    {
      name: 'api-2',
      namespace: 'production',
      status: 'Running',
      age: '1d',
      cpuCores: 0.5,
      memoryBytes: 512 * 1024 * 1024
    },
    {
      name: 'staging-pod',
      namespace: 'staging',
      status: 'Running',
      age: '1d',
      cpuCores: 1.0,
      memoryBytes: 1024 * 1024 * 1024
    }
  ]

  const quota = {
    cpuRequest: '2',
    cpuLimit: '4',
    cpuUsed: '1',
    cpuPercent: 25,
    memoryRequest: '4Gi',
    memoryLimit: '4Gi',
    memoryUsed: '1Gi',
    memoryPercent: 25
  }

  const nodes: NodeInfo[] = []

  const metrics = aggregateNamespacePodMetrics(pods, 'production', nodes, quota)
  assert.equal(metrics.cpuCores, 1.0)
  assert.equal(metrics.cpuUsage, '1 cores')
  assert.equal(metrics.cpuPercent, 25) // 1 / 4 cores = 25%

  assert.equal(metrics.memoryBytes, 1024 * 1024 * 1024)
  assert.equal(metrics.memoryUsage, '1024Mi')
  assert.equal(metrics.memoryPercent, 25) // 1Gi / 4Gi = 25%
})

test('aggregateNamespacePodMetrics falls back to cluster node capacity when quota is null', () => {
  const pods: PodInfo[] = [
    {
      name: 'worker-1',
      namespace: 'batch',
      status: 'Running',
      age: '1d',
      cpuCores: 2.0,
      memoryBytes: 2 * 1024 * 1024 * 1024
    }
  ]

  const nodes: NodeInfo[] = [
    {
      name: 'node-1',
      status: 'Ready',
      role: 'worker',
      version: 'v1.31.0',
      cpuPct: 50,
      cpuUsed: '4',
      cpuTotal: '8', // 8 cores
      memPct: 50,
      memUsed: '8Gi',
      memTotal: '16Gi', // 16Gi
      podsCount: 10,
      podsLimit: 110,
      uptime: '10d',
      isCordoned: false
    }
  ]

  const metrics = aggregateNamespacePodMetrics(pods, 'batch', nodes, null)
  assert.equal(metrics.cpuCores, 2.0)
  assert.equal(metrics.cpuUsage, '2 cores')
  assert.equal(metrics.cpuPercent, 25) // 2 / 8 = 25%

  assert.equal(metrics.memoryBytes, 2 * 1024 * 1024 * 1024)
  assert.equal(metrics.memoryPercent, 13) // 2Gi / 16Gi ~ 12.5% -> 13%
})

test('aggregateNamespacePodMetrics handles empty pods safely', () => {
  const metrics = aggregateNamespacePodMetrics([], 'empty-ns', [], null)
  assert.equal(metrics.cpuCores, 0)
  assert.equal(metrics.cpuUsage, '0m')
  assert.equal(metrics.cpuPercent, 0)
  assert.equal(metrics.memoryBytes, 0)
  assert.equal(metrics.memoryUsage, '0Mi')
  assert.equal(metrics.memoryPercent, 0)
})
