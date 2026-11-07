import { KUBERNETES_HPA_STATUS } from '../constants/kubernetes.ts'
import type { HpaConditionInfo } from '../types/kubernetes.ts'
import type { TagSeverity } from './severity.ts'

export interface HpaValidationResult {
  valid: boolean
  error?: string
}

export function validateHpaBounds(minReplicas: number, maxReplicas: number): HpaValidationResult {
  if (!Number.isInteger(minReplicas) || minReplicas < 1) {
    return { valid: false, error: 'Min replicas must be an integer greater than or equal to 1' }
  }
  if (!Number.isInteger(maxReplicas) || maxReplicas < 1) {
    return { valid: false, error: 'Max replicas must be an integer greater than or equal to 1' }
  }
  if (maxReplicas < minReplicas) {
    return {
      valid: false,
      error: `Max replicas (${maxReplicas}) cannot be less than Min replicas (${minReplicas})`
    }
  }
  return { valid: true }
}

export interface HpaTargetRoute {
  path: string
  query: {
    tab: string
    highlight: string
    namespace: string
  }
}

export function resolveHpaTargetRoute(
  kind: string,
  name: string,
  namespace: string
): HpaTargetRoute | null {
  switch (kind) {
    case 'Deployment':
      return {
        path: '/workloads',
        query: { tab: 'deployments', highlight: name, namespace }
      }
    case 'StatefulSet':
      return {
        path: '/workloads',
        query: { tab: 'statefulsets', highlight: name, namespace }
      }
    case 'ReplicaSet':
      return {
        path: '/workloads',
        query: { tab: 'replicasets', highlight: name, namespace }
      }
    default:
      return null
  }
}

export function getHpaStatusBadgeClass(status?: string): string {
  switch (status) {
    case KUBERNETES_HPA_STATUS.Active:
      return 'bg-(--success)'
    case KUBERNETES_HPA_STATUS.ScalingLimited:
    case KUBERNETES_HPA_STATUS.Pending:
      return 'bg-(--warning)'
    case KUBERNETES_HPA_STATUS.Failed:
      return 'bg-(--danger)'
    default:
      return 'bg-muted-color'
  }
}

export function getHpaStatusSeverity(status?: string): TagSeverity {
  switch (status) {
    case KUBERNETES_HPA_STATUS.Active:
      return 'success'
    case KUBERNETES_HPA_STATUS.ScalingLimited:
    case KUBERNETES_HPA_STATUS.Pending:
      return 'warn'
    case KUBERNETES_HPA_STATUS.Failed:
      return 'danger'
    default:
      return 'secondary'
  }
}

export interface FormattedHpaCondition {
  type: string
  status: string
  severity: TagSeverity
  reason?: string
  message?: string
  lastTransitionTime?: string
}

export function formatHpaConditions(conditions: HpaConditionInfo[]): FormattedHpaCondition[] {
  return conditions.map((cond) => {
    let severity: TagSeverity = 'secondary'
    if (cond.type === 'AbleToScale' || cond.type === 'ScalingActive') {
      severity = cond.status === 'True' ? 'success' : 'danger'
    } else if (cond.type === 'ScalingLimited') {
      severity = cond.status === 'True' ? 'warn' : 'secondary'
    }
    return {
      type: cond.type,
      status: cond.status,
      severity,
      reason: cond.reason,
      message: cond.message,
      lastTransitionTime: cond.lastTransitionTime
    }
  })
}
