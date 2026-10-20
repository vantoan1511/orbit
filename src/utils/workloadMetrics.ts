import type { PodInfo } from '../types/kubernetes.ts'

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
