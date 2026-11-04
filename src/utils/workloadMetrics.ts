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

export type WithReplicasSortKey<T> = T & {
  replicasSortKey: number
}

/**
 * Attaches a numeric replicasSortKey to a list of workload items without mutating the original items.
 */
export function withReplicasSortKey<T extends { replicas?: Replicas | null }>(
  items: readonly T[]
): WithReplicasSortKey<T>[] {
  return items.map((item) => ({
    ...item,
    replicasSortKey: computeReplicasSortKey(item.replicas)
  }))
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
        replicasSortKey: computeReplicasSortKey(d.replicas)
      }
      cache.set(d, { restarts: r, row })
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
