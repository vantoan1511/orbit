import { KUBERNETES_RESOURCE_KIND } from '../constants/kubernetes.ts'

/**
 * Extracts available ports for port-forwarding based on the resource kind and row data.
 * - Service: maps port numbers from `portsList`
 * - Deployment: returns container ports from `ports`
 */
export function getAvailablePorts(row: unknown, resourceKind?: string): number[] {
  if (!row || typeof row !== 'object') return []

  if (resourceKind === KUBERNETES_RESOURCE_KIND.Service && 'portsList' in row) {
    const svc = row as { portsList?: Array<{ port: number }> }
    return svc.portsList?.map((p) => p.port) || []
  }

  if (resourceKind === KUBERNETES_RESOURCE_KIND.Deployment && 'ports' in row) {
    const dep = row as { ports?: number[] }
    return Array.isArray(dep.ports) ? dep.ports : []
  }

  return []
}
