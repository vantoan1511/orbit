import {
  KUBERNETES_EVENT_TYPE,
  KUBERNETES_NAMESPACE_STATUS,
  KUBERNETES_NODE_STATUS,
  KUBERNETES_POD_STATUS,
  KUBERNETES_RESOURCE_KIND,
  KUBERNETES_SERVICE_TYPE
} from '@/constants/kubernetes'

export type TagSeverity = 'success' | 'info' | 'warn' | 'danger' | 'secondary' | 'contrast'

/**
 * Returns the Tag severity for a Kubernetes Service type.
 */
export function getServiceTypeSeverity(type?: string): TagSeverity {
  switch (type) {
    case KUBERNETES_SERVICE_TYPE.LoadBalancer:
      return 'info'
    case KUBERNETES_SERVICE_TYPE.ClusterIP:
      return 'success'
    case KUBERNETES_SERVICE_TYPE.NodePort:
      return 'warn'
    case KUBERNETES_SERVICE_TYPE.ExternalName:
      return 'contrast'
    default:
      return 'secondary'
  }
}

/**
 * Returns the Tag severity for a Kubernetes Event type.
 */
export function getEventTypeSeverity(type?: string): TagSeverity {
  switch (type) {
    case KUBERNETES_EVENT_TYPE.Warning:
      return 'warn'
    case KUBERNETES_EVENT_TYPE.Error:
      return 'danger'
    case KUBERNETES_EVENT_TYPE.Normal:
      return 'success'
    default:
      return 'secondary'
  }
}

/**
 * Returns the Tag severity for a Kubernetes Workload kind.
 */
export function getWorkloadKindSeverity(kind?: string): TagSeverity {
  switch (kind) {
    case KUBERNETES_RESOURCE_KIND.Deployment:
    case KUBERNETES_RESOURCE_KIND.StatefulSet:
    case KUBERNETES_RESOURCE_KIND.DaemonSet:
    case KUBERNETES_RESOURCE_KIND.ReplicaSet:
      return 'info'
    case KUBERNETES_RESOURCE_KIND.Job:
    case KUBERNETES_RESOURCE_KIND.CronJob:
      return 'warn'
    default:
      return 'secondary'
  }
}

/**
 * Returns background color class for a Pod status dot/badge.
 */
export function getPodStatusBadgeClass(status?: string): string {
  switch (status) {
    case KUBERNETES_POD_STATUS.Running:
    case KUBERNETES_POD_STATUS.Completed:
    case KUBERNETES_POD_STATUS.Succeeded:
      return 'bg-emerald-500'
    case KUBERNETES_POD_STATUS.Pending:
    case KUBERNETES_POD_STATUS.ContainerCreating:
      return 'bg-amber-500'
    case KUBERNETES_POD_STATUS.CrashLoopBackOff:
    case KUBERNETES_POD_STATUS.Error:
    case KUBERNETES_POD_STATUS.Failed:
      return 'bg-rose-500'
    default:
      return 'bg-muted-color'
  }
}

/**
 * Returns background color class for a Node status dot/badge.
 */
export function getNodeStatusBadgeClass(status?: string): string {
  switch (status) {
    case KUBERNETES_NODE_STATUS.Ready:
      return 'bg-emerald-500'
    case KUBERNETES_NODE_STATUS.NotReady:
      return 'bg-rose-500'
    default:
      return 'bg-amber-500'
  }
}

/**
 * Returns background color class for a Namespace status dot/badge.
 */
export function getNamespaceStatusBadgeClass(status?: string): string {
  switch (status) {
    case KUBERNETES_NAMESPACE_STATUS.Active:
      return 'bg-emerald-500'
    case KUBERNETES_NAMESPACE_STATUS.Terminating:
      return 'bg-amber-500'
    default:
      return 'bg-gray-400'
  }
}
