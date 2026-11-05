import type { NodeInfo, PodInfo, PolicyInfo } from '@/types/kubernetes'
import { formatCpuCores, formatMemoryMiB } from './metrics.ts'
import * as yaml from 'yaml'

export interface ResourceQuotaInfo {
  cpuRequest: string
  cpuLimit: string
  cpuUsed: string
  cpuPercent: number
  memoryRequest: string
  memoryLimit: string
  memoryUsed: string
  memoryPercent: number
}

export interface LimitRangeInfo {
  type: string
  resource: string
  min: string
  max: string
  default: string
  defaultRequest?: string
}

export interface AggregatedNamespaceMetrics {
  cpuUsage: string
  cpuCores: number
  cpuPercent: number
  memoryUsage: string
  memoryBytes: number
  memoryPercent: number
}

/**
 * Parses Kubernetes CPU quantity string to decimal cores.
 * e.g. "500m" -> 0.5, "1500m" -> 1.5, "2" -> 2.0, "250u" -> 0.00025
 */
export function parseCpuQuantityToCores(q: string | undefined | null): number {
  if (!q) return 0
  const trimmed = String(q).trim()
  if (!trimmed) return 0

  if (trimmed.endsWith('m')) {
    const val = Number.parseFloat(trimmed.slice(0, -1))
    return Number.isFinite(val) ? val / 1000 : 0
  }
  if (trimmed.endsWith('u')) {
    const val = Number.parseFloat(trimmed.slice(0, -1))
    return Number.isFinite(val) ? val / 1_000_000 : 0
  }
  if (trimmed.endsWith('n')) {
    const val = Number.parseFloat(trimmed.slice(0, -1))
    return Number.isFinite(val) ? val / 1_000_000_000 : 0
  }

  const val = Number.parseFloat(trimmed)
  return Number.isFinite(val) ? val : 0
}

/**
 * Parses Kubernetes Memory quantity string to raw bytes.
 * e.g. "1Ki" -> 1024, "2Mi" -> 2097152, "1Gi" -> 1073741824
 * e.g. "1K" -> 1000, "1M" -> 1000000, "1G" -> 1000000000
 */
export function parseMemoryQuantityToBytes(q: string | undefined | null): number {
  if (!q) return 0
  const trimmed = String(q).trim()
  if (!trimmed) return 0

  const binaryMultipliers: Record<string, number> = {
    ki: 1024,
    mi: 1024 ** 2,
    gi: 1024 ** 3,
    ti: 1024 ** 4,
    pi: 1024 ** 5,
    ei: 1024 ** 6
  }

  const decimalMultipliers: Record<string, number> = {
    k: 1000,
    m: 1000 ** 2,
    g: 1000 ** 3,
    t: 1000 ** 4,
    p: 1000 ** 5,
    e: 1000 ** 6
  }

  const binaryMatch = trimmed.match(/^([0-9.]+)\s*([A-Za-z]i)$/)
  if (binaryMatch && binaryMatch[1] && binaryMatch[2]) {
    const num = Number.parseFloat(binaryMatch[1])
    const unit = binaryMatch[2].toLowerCase()
    const mult = binaryMultipliers[unit] ?? 1
    return Number.isFinite(num) ? num * mult : 0
  }

  const decimalMatch = trimmed.match(/^([0-9.]+)\s*([A-Za-z])$/)
  if (decimalMatch && decimalMatch[1] && decimalMatch[2]) {
    const num = Number.parseFloat(decimalMatch[1])
    const unit = decimalMatch[2].toLowerCase()
    const mult = decimalMultipliers[unit] ?? 1
    return Number.isFinite(num) ? num * mult : 0
  }

  const num = Number.parseFloat(trimmed)
  return Number.isFinite(num) ? num : 0
}

interface RawQuotaYaml {
  hard?: Record<string, string | number>
  used?: Record<string, string | number>
  spec?: { hard?: Record<string, string | number> }
  status?: {
    hard?: Record<string, string | number>
    used?: Record<string, string | number>
  }
}

/**
 * Extracts and computes ResourceQuotaInfo for a namespace from policies.
 */
export function extractNamespaceResourceQuota(
  policies: PolicyInfo[],
  namespaceName: string
): ResourceQuotaInfo | null {
  const quotaPolicy = policies.find(
    (p) =>
      (p.kind === 'ResourceQuota' || p.type.toLowerCase().includes('quota')) &&
      p.namespace === namespaceName
  )

  if (!quotaPolicy || !quotaPolicy.rules) {
    return null
  }

  let doc: RawQuotaYaml | null = null
  try {
    doc = yaml.parse(quotaPolicy.rules) as RawQuotaYaml | null
  } catch {
    return null
  }

  if (!doc) return null

  const hard = doc.hard || doc.status?.hard || doc.spec?.hard || {}
  const used = doc.used || doc.status?.used || {}

  const getQuotaVal = (map: Record<string, string | number>, ...keys: string[]): string => {
    for (const k of keys) {
      if (map[k] !== undefined && map[k] !== null) {
        return String(map[k])
      }
    }
    return '-'
  }

  const cpuLimit = getQuotaVal(hard, 'limits.cpu', 'cpu')
  const cpuRequest = getQuotaVal(hard, 'requests.cpu', 'cpu')
  const cpuUsed = getQuotaVal(used, 'limits.cpu', 'cpu', 'requests.cpu')

  const memoryLimit = getQuotaVal(hard, 'limits.memory', 'memory')
  const memoryRequest = getQuotaVal(hard, 'requests.memory', 'memory')
  const memoryUsed = getQuotaVal(used, 'limits.memory', 'memory', 'requests.memory')

  const cpuLimitCores = parseCpuQuantityToCores(cpuLimit)
  const cpuUsedCores = parseCpuQuantityToCores(cpuUsed)
  const cpuPercent =
    cpuLimitCores > 0 ? Math.min(100, Math.round((cpuUsedCores / cpuLimitCores) * 100)) : 0

  const memLimitBytes = parseMemoryQuantityToBytes(memoryLimit)
  const memUsedBytes = parseMemoryQuantityToBytes(memoryUsed)
  const memoryPercent =
    memLimitBytes > 0 ? Math.min(100, Math.round((memUsedBytes / memLimitBytes) * 100)) : 0

  return {
    cpuRequest,
    cpuLimit,
    cpuUsed,
    cpuPercent,
    memoryRequest,
    memoryLimit,
    memoryUsed,
    memoryPercent
  }
}

interface RawLimitRangeItem {
  type?: string
  min?: Record<string, string | number>
  max?: Record<string, string | number>
  default?: Record<string, string | number>
  defaultRequest?: Record<string, string | number>
}

interface RawLimitRangeYaml {
  limits?: RawLimitRangeItem[]
}

/**
 * Extracts LimitRange items for a namespace from policies.
 */
export function extractNamespaceLimitRanges(
  policies: PolicyInfo[],
  namespaceName: string
): LimitRangeInfo[] {
  const limitRangePolicies = policies.filter(
    (p) =>
      (p.kind === 'LimitRange' || p.type.toLowerCase().includes('limit')) &&
      p.namespace === namespaceName
  )

  const results: LimitRangeInfo[] = []

  for (const policy of limitRangePolicies) {
    if (!policy.rules) continue
    try {
      const doc = yaml.parse(policy.rules) as RawLimitRangeYaml | null
      const items = doc?.limits || []
      for (const item of items) {
        const itemType = item.type || 'Container'
        const resources = new Set<string>()

        if (item.min) Object.keys(item.min).forEach((k) => resources.add(k))
        if (item.max) Object.keys(item.max).forEach((k) => resources.add(k))
        if (item.default) Object.keys(item.default).forEach((k) => resources.add(k))
        if (item.defaultRequest) Object.keys(item.defaultRequest).forEach((k) => resources.add(k))

        for (const res of resources) {
          results.push({
            type: itemType,
            resource: res,
            min: item.min?.[res] !== undefined ? String(item.min[res]) : '-',
            max: item.max?.[res] !== undefined ? String(item.max[res]) : '-',
            default: item.default?.[res] !== undefined ? String(item.default[res]) : '-',
            defaultRequest:
              item.defaultRequest?.[res] !== undefined ? String(item.defaultRequest[res]) : '-'
          })
        }
      }
    } catch {
      // Ignore parse errors on malformed rules
    }
  }

  return results
}

/**
 * Aggregates live CPU and Memory metrics for all pods belonging to a namespace,
 * calculating saturation percentage against ResourceQuota limits if present,
 * or cluster node capacity as a fallback.
 */
export function aggregateNamespacePodMetrics(
  pods: PodInfo[],
  namespaceName: string,
  nodes: NodeInfo[],
  quota?: ResourceQuotaInfo | null
): AggregatedNamespaceMetrics {
  const nsPods = pods.filter((p) => p.namespace === namespaceName)

  let totalCpuCores = 0
  let totalMemoryBytes = 0

  for (const pod of nsPods) {
    if (typeof pod.cpuCores === 'number') {
      totalCpuCores += pod.cpuCores
    } else if (pod.cpu) {
      totalCpuCores += parseCpuQuantityToCores(pod.cpu)
    }

    if (typeof pod.memoryBytes === 'number') {
      totalMemoryBytes += pod.memoryBytes
    } else if (pod.memory) {
      totalMemoryBytes += parseMemoryQuantityToBytes(pod.memory)
    }
  }

  // Calculate CPU saturation percentage
  let cpuPercent = 0
  const quotaCpuLimitCores = quota?.cpuLimit ? parseCpuQuantityToCores(quota.cpuLimit) : 0
  if (quotaCpuLimitCores > 0) {
    cpuPercent = Math.min(100, Math.round((totalCpuCores / quotaCpuLimitCores) * 100))
  } else if (nodes.length > 0) {
    const clusterCpuCores = nodes.reduce(
      (sum, n) => sum + parseCpuQuantityToCores(n.cpuTotal || n.cpuUsed),
      0
    )
    if (clusterCpuCores > 0) {
      cpuPercent = Math.min(100, Math.round((totalCpuCores / clusterCpuCores) * 100))
    }
  }

  // Calculate Memory saturation percentage
  let memoryPercent = 0
  const quotaMemoryLimitBytes = quota?.memoryLimit
    ? parseMemoryQuantityToBytes(quota.memoryLimit)
    : 0
  if (quotaMemoryLimitBytes > 0) {
    memoryPercent = Math.min(100, Math.round((totalMemoryBytes / quotaMemoryLimitBytes) * 100))
  } else if (nodes.length > 0) {
    const clusterMemoryBytes = nodes.reduce(
      (sum, n) => sum + parseMemoryQuantityToBytes(n.memTotal || n.memUsed),
      0
    )
    if (clusterMemoryBytes > 0) {
      memoryPercent = Math.min(100, Math.round((totalMemoryBytes / clusterMemoryBytes) * 100))
    }
  }

  const cpuUsage = formatCpuCores(totalCpuCores)
  const memoryMib = totalMemoryBytes / (1024 * 1024)
  const memoryUsage = formatMemoryMiB(memoryMib)

  return {
    cpuUsage,
    cpuCores: totalCpuCores,
    cpuPercent,
    memoryUsage,
    memoryBytes: totalMemoryBytes,
    memoryPercent
  }
}
