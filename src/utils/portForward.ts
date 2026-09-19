import { KUBERNETES_RESOURCE_KIND } from '../constants/kubernetes.ts'

/**
 * Extracts available ports for port-forwarding based on the resource kind and row data.
 * - Service: maps port numbers from `portsList`
 * - Deployment, StatefulSet, ReplicaSet: returns container ports from `ports`
 * - Pod: extracts ports from container `ports` strings (e.g. "80/TCP, 443/TCP")
 */
export function getAvailablePorts(row: unknown, resourceKind?: string): number[] {
  if (!row || typeof row !== 'object') return []

  if (resourceKind === KUBERNETES_RESOURCE_KIND.Service && 'portsList' in row) {
    const svc = row as { portsList?: Array<{ port: number }> }
    return svc.portsList?.map((p) => p.port) || []
  }

  if (
    (resourceKind === KUBERNETES_RESOURCE_KIND.Deployment ||
      resourceKind === KUBERNETES_RESOURCE_KIND.StatefulSet ||
      resourceKind === KUBERNETES_RESOURCE_KIND.ReplicaSet) &&
    'ports' in row
  ) {
    const workload = row as { ports?: number[] }
    return Array.isArray(workload.ports) ? workload.ports : []
  }

  if (resourceKind === KUBERNETES_RESOURCE_KIND.Pod && 'containers' in row) {
    const pod = row as { containers?: Array<{ ports?: string }> }
    if (Array.isArray(pod.containers)) {
      const portSet = new Set<number>()
      for (const container of pod.containers) {
        if (typeof container.ports === 'string') {
          const matches = container.ports.match(/\b\d+\b/g)
          if (matches) {
            for (const m of matches) {
              const num = parseInt(m, 10)
              if (num >= 1 && num <= 65535) {
                portSet.add(num)
              }
            }
          }
        }
      }
      return Array.from(portSet).sort((a, b) => a - b)
    }
  }

  return []
}
