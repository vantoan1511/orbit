import {
  KUBERNETES_EVENT_TYPE,
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
