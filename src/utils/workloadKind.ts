import { KUBERNETES_RESOURCE_KIND } from '../constants/kubernetes.ts'
import type { WorkloadInfo } from '../types/kubernetes.ts'

/**
 * Resolves the Kubernetes resource kind for a workload.
 * Prioritizes the explicit `kind` field returned by the backend engine,
 * falling back to structural and heuristic inference for backward compatibility.
 */
export const getWorkloadKind = (w: WorkloadInfo): string => {
  if ('kind' in w && w.kind) return w.kind
  if ('schedule' in w) return KUBERNETES_RESOURCE_KIND.CronJob
  if ('completions' in w) return KUBERNETES_RESOURCE_KIND.Job
  if ('strategy' in w) return KUBERNETES_RESOURCE_KIND.Deployment
  if ('replicas' in w && w.replicas) {
    const reps = w.replicas
    if ('ready' in reps && 'upToDate' in reps) return KUBERNETES_RESOURCE_KIND.DaemonSet
    if (w.name && w.name.includes('stateful')) return KUBERNETES_RESOURCE_KIND.StatefulSet
    return KUBERNETES_RESOURCE_KIND.ReplicaSet
  }
  return 'Workload'
}
