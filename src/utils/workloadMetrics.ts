import type { DeploymentCondition } from 'kubernetes-types/apps/v1'
import type { DeploymentInfo, PodInfo, Replicas } from '../types/kubernetes.ts'

export const REPLICAS_SORT_MULTIPLIER = 1_000_000

/**
 * Computes a deterministic numeric sort key for workload replica counts.
 * Orders primarily by desired replicas, breaking ties by current replicas.
 */
export function computeReplicasSortKey(replicas?: Replicas | null): number {
  if (!replicas) return 0
  const desired = Number.isFinite(replicas.desired) ? Math.max(0, Math.trunc(replicas.desired)) : 0
  const current = Number.isFinite(replicas.current) ? Math.max(0, Math.trunc(replicas.current)) : 0
  return desired * REPLICAS_SORT_MULTIPLIER + Math.min(current, REPLICAS_SORT_MULTIPLIER - 1)
}

export interface ImageTagItem {
  full: string
  short: string
}

/**
 * Precomputes full and short image names for container image tags.
 */
export function formatImageTags(images?: readonly string[]): ImageTagItem[] {
  if (!images || images.length === 0) return []
  return images.map((img) => ({
    full: img,
    short: img.split('/').pop() || img
  }))
}

export interface OrderedDeploymentCondition {
  type: string
  status: string
  reason?: string
  message?: string
  severity: 'danger' | 'success' | 'info' | 'secondary'
  tooltip: string
}

const CONDITION_ORDER: Record<string, number> = {
  Available: 1,
  Progressing: 2,
  ReplicaFailure: 3
}

export function getConditionSeverity(type: string): 'danger' | 'success' | 'info' | 'secondary' {
  switch (type) {
    case 'ReplicaFailure':
      return 'danger'
    case 'Available':
      return 'success'
    case 'Progressing':
      return 'info'
    default:
      return 'secondary'
  }
}

/**
 * Precomputes and orders active (status: 'True') deployment conditions.
 */
export function formatOrderedDeploymentConditions(
  conditions?: readonly DeploymentCondition[]
): OrderedDeploymentCondition[] {
  if (!conditions || conditions.length === 0) return []
  return [...conditions]
    .filter((c) => c.status === 'True')
    .sort((a, b) => {
      const orderA = CONDITION_ORDER[a.type] ?? 99
      const orderB = CONDITION_ORDER[b.type] ?? 99
      if (orderA !== orderB) return orderA - orderB
      return a.type.localeCompare(b.type)
    })
    .map((c) => ({
      type: c.type,
      status: c.status,
      reason: c.reason,
      message: c.message,
      severity: getConditionSeverity(c.type),
      tooltip: c.message || c.reason || c.type
    }))
}

export type WithReplicasSortKey<T> = T & {
  replicasSortKey: number
  imageTags: ImageTagItem[]
}

const replicasSortKeyCache = new WeakMap<object, WithReplicasSortKey<unknown>>()

/**
 * Attaches numeric replicasSortKey and precomputed imageTags to workload items without mutating the original items.
 * Memoised by object identity via a WeakMap.
 */
export function withReplicasSortKey<T extends { replicas?: Replicas | null; images?: string[] }>(
  items: readonly T[]
): WithReplicasSortKey<T>[] {
  return items.map((item) => {
    if (typeof item === 'object' && item !== null) {
      const cached = replicasSortKeyCache.get(item)
      if (cached) return cached as unknown as WithReplicasSortKey<T>
    }
    const decorated: WithReplicasSortKey<T> = {
      ...item,
      replicasSortKey: computeReplicasSortKey(item.replicas),
      imageTags: formatImageTags(item.images)
    }
    if (typeof item === 'object' && item !== null) {
      replicasSortKeyCache.set(item, decorated)
    }
    return decorated
  })
}

const REPLICA_SET_PREFIX = 'ReplicaSet/'

/**
 * Aggregates pod restart counts by parent Deployment name.
 */
export function computeDeploymentRestarts(pods: PodInfo[]): Map<string, number> {
  const map = new Map<string, number>()
  for (const pod of pods) {
    const restarts = pod.restarts
    if (restarts == null || restarts === 0 || !pod.controlledBy || !pod.namespace) continue
    if (pod.controlledBy.startsWith(REPLICA_SET_PREFIX)) {
      const rsName = pod.controlledBy.slice(REPLICA_SET_PREFIX.length)
      const lastDash = rsName.lastIndexOf('-')
      if (lastDash > 0) {
        const depName = rsName.slice(0, lastDash)
        const key = `${pod.namespace}/${depName}`
        map.set(key, (map.get(key) ?? 0) + restarts)
      }
    }
  }
  return map
}

export interface DeploymentRow extends DeploymentInfo {
  restarts: number
  replicasSortKey: number
  orderedConditions: OrderedDeploymentCondition[]
  imageTags: ImageTagItem[]
}

/**
 * Creates a memoised decorator for Deployment rows.
 * Reuses previous row object when source deployment object & restart count are unchanged.
 */
export function createDeploymentRowDecorator(): (
  deployments: DeploymentInfo[],
  restarts: Map<string, number>
) => DeploymentRow[] {
  const cache = new WeakMap<DeploymentInfo, { restarts: number; row: DeploymentRow }>()

  return (deployments: DeploymentInfo[], restarts: Map<string, number>): DeploymentRow[] => {
    return deployments.map((d) => {
      const key = `${d.namespace ?? ''}/${d.name}`
      const r = restarts.get(key) ?? 0
      const cached = cache.get(d)
      if (cached && cached.restarts === r) {
        return cached.row
      }
      const row: DeploymentRow = {
        ...d,
        restarts: r,
        replicasSortKey: computeReplicasSortKey(d.replicas),
        orderedConditions: formatOrderedDeploymentConditions(d.conditions),
        imageTags: formatImageTags(d.images)
      }
      cache.set(d, { restarts: r, row })
      return row
    })
  }
}

export interface PodRow extends PodInfo {
  nodeShort: string
  hasCpuPct: boolean
  cpuWidthStyle: string
  hasMemoryPct: boolean
  memoryWidthStyle: string
}

/**
 * Creates a memoised decorator for Pod rows.
 * Precomputes nodeShort, CPU/memory progress bar styles, and memoises results via WeakMap.
 */
export function createPodRowDecorator(): (pods: PodInfo[]) => PodRow[] {
  const cache = new WeakMap<PodInfo, PodRow>()

  return (pods: PodInfo[]): PodRow[] => {
    return pods.map((p) => {
      const cached = cache.get(p)
      if (cached) return cached

      const nodeShort = p.node ? (p.node.split('.')[0] ?? 'N/A') : 'N/A'
      const hasCpuPct = typeof p.cpuPct === 'number' && Boolean(p.cpu) && p.cpu !== '-'
      const cpuPctClamped = hasCpuPct ? Math.min(Math.max(p.cpuPct ?? 0, 0), 100) : 0
      const cpuWidthStyle = `${cpuPctClamped}%`

      const hasMemoryPct = typeof p.memoryPct === 'number' && Boolean(p.memory) && p.memory !== '-'
      const memoryPctClamped = hasMemoryPct ? Math.min(Math.max(p.memoryPct ?? 0, 0), 100) : 0
      const memoryWidthStyle = `${memoryPctClamped}%`

      const row: PodRow = {
        ...p,
        nodeShort,
        hasCpuPct,
        cpuWidthStyle,
        hasMemoryPct,
        memoryWidthStyle
      }
      cache.set(p, row)
      return row
    })
  }
}

const fingerprintCache = new WeakMap<PodInfo[], string>()

/**
 * Computes a restart fingerprint for pods controlled by a ReplicaSet.
 * Memoised by pods array reference identity.
 */
export function restartFingerprint(pods: PodInfo[]): string {
  const cached = fingerprintCache.get(pods)
  if (cached !== undefined) return cached

  const parts: string[] = []
  for (const p of pods) {
    if (p.restarts && p.controlledBy?.startsWith(REPLICA_SET_PREFIX)) {
      parts.push(`${p.namespace ?? ''}/${p.name}:${p.restarts}`)
    }
  }
  const fp = parts.join('|')
  fingerprintCache.set(pods, fp)
  return fp
}

export type MetricType = 'cpu' | 'memory' | 'restarts'

export interface ConsumerItem {
  pod: string
  namespace: string
  value: string
  pct: number
  color: string
}

export const METRIC_CONFIGS: Record<
  MetricType,
  {
    sortKey: keyof PodInfo
    getValue: (p: PodInfo) => string
    getPct: (p: PodInfo) => number
    color: string
  }
> = {
  cpu: {
    sortKey: 'cpuPct',
    getValue: (p) => p.cpu || '0m',
    getPct: (p) => Math.min(Math.max(Number((p.cpuPct || 0).toFixed(2)), 0), 100),
    color: 'bg-(--accent)'
  },
  memory: {
    sortKey: 'memoryPct',
    getValue: (p) => p.memory || '0Mi',
    getPct: (p) => Math.min(Math.max(Number((p.memoryPct || 0).toFixed(2)), 0), 100),
    color: 'bg-violet-500'
  },
  restarts: {
    sortKey: 'restarts',
    getValue: (p) => String(p.restarts || 0),
    getPct: () => 0,
    color: 'bg-amber-500'
  }
}

/**
 * Returns the top N pods sorted descending by the given metric.
 */
export function getTopConsumers(pods: PodInfo[], metric: MetricType, limit = 5): ConsumerItem[] {
  const config = METRIC_CONFIGS[metric]
  if (!config) return []

  const sorted = [...pods].sort(
    (a, b) => ((b[config.sortKey] as number) || 0) - ((a[config.sortKey] as number) || 0)
  )

  return sorted.slice(0, limit).map((p) => ({
    pod: p.name,
    namespace: p.namespace,
    value: config.getValue(p),
    pct: config.getPct(p),
    color: config.color
  }))
}
