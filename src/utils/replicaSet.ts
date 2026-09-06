import type { ReplicaSet } from 'kubernetes-types/apps/v1'
import { isValidK8sLabel, isValidK8sName, isValidPath } from './validators.ts'
import {
  type ContainerEnvFormItem,
  type WorkloadContainerFormState,
  type WorkloadContainerValidationItem,
  parseContainersFromPodSpec,
  formatContainersToPodSpec
} from './workloadContainer.ts'

export type { ContainerEnvFormItem }
export type ReplicaSetContainerFormState = WorkloadContainerFormState
export type ReplicaSetContainerValidationItem = WorkloadContainerValidationItem
export { parseContainersFromPodSpec, formatContainersToPodSpec }

export interface ReplicaSetFormValidationInput {
  serviceAccountName?: string
  replicas?: number
  minReadySeconds?: number
  containers?: ReplicaSetContainerValidationItem[]
}

/**
 * Validates ReplicaSet form fields against Kubernetes constraints.
 */
export function validateReplicaSetForm(input: ReplicaSetFormValidationInput): boolean {
  if (
    input.replicas !== undefined &&
    (typeof input.replicas !== 'number' ||
      isNaN(input.replicas) ||
      input.replicas < 0 ||
      !Number.isInteger(input.replicas))
  ) {
    return false
  }

  if (
    input.minReadySeconds !== undefined &&
    (typeof input.minReadySeconds !== 'number' ||
      isNaN(input.minReadySeconds) ||
      input.minReadySeconds < 0 ||
      !Number.isInteger(input.minReadySeconds))
  ) {
    return false
  }

  if (input.serviceAccountName && !isValidK8sName(input.serviceAccountName)) {
    return false
  }

  if (input.containers) {
    for (const c of input.containers) {
      if (!c.name?.trim() || !isValidK8sLabel(c.name.trim())) {
        return false
      }
      if (c.workingDir && !isValidPath(c.workingDir)) {
        return false
      }
      if (c.ports) {
        for (const p of c.ports) {
          if (p.name && !isValidK8sLabel(p.name)) {
            return false
          }
        }
      }
    }
  }

  return true
}

export interface NormalizedReplicaSetSpec {
  replicas: number
  minReadySeconds: number
  selectorLabels: Array<{ key: string; value: string }>
}

/**
 * Normalizes ReplicaSet specification parameters for form controls.
 */
export function normalizeReplicaSetSpec(data: ReplicaSet | null): NormalizedReplicaSetSpec {
  const spec = data?.spec

  const replicas = typeof spec?.replicas === 'number' ? spec.replicas : 1
  const minReadySeconds = typeof spec?.minReadySeconds === 'number' ? spec.minReadySeconds : 0

  const matchLabels = spec?.selector?.matchLabels
  const selectorLabels: Array<{ key: string; value: string }> = []
  if (matchLabels && typeof matchLabels === 'object') {
    for (const [key, value] of Object.entries(matchLabels)) {
      selectorLabels.push({ key, value: String(value ?? '') })
    }
  }

  return {
    replicas,
    minReadySeconds,
    selectorLabels
  }
}
