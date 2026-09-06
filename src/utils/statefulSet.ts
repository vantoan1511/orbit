import type { StatefulSet } from 'kubernetes-types/apps/v1'
import type { PersistentVolumeClaim } from 'kubernetes-types/core/v1'
import {
  KUBERNETES_STATEFULSET_POD_MANAGEMENT_POLICY,
  KUBERNETES_STATEFULSET_UPDATE_STRATEGY
} from '../constants/kubernetes.ts'
import { isValidK8sLabel, isValidK8sName, isValidPath } from './validators.ts'
import {
  type ContainerEnvFormItem,
  type WorkloadContainerFormState,
  type WorkloadContainerValidationItem,
  parseContainersFromPodSpec,
  formatContainersToPodSpec
} from './workloadContainer.ts'

export type { ContainerEnvFormItem }
export type StatefulSetContainerFormState = WorkloadContainerFormState
export type StatefulSetContainerValidationItem = WorkloadContainerValidationItem
export { parseContainersFromPodSpec, formatContainersToPodSpec }

export interface FormattedVolumeClaim {
  name: string
  storageClassName: string
  accessModes: string
  storageCapacity: string
  volumeMode: string
}

export interface StatefulSetFormValidationInput {
  serviceName?: string
  serviceAccountName?: string
  strategyType?: string
  maxUnavailable?: string
  containers?: StatefulSetContainerValidationItem[]
}

/**
 * Validates Kubernetes maxUnavailable string (percentage 0-100% or non-negative integer).
 */
export function isValidMaxUnavailable(value?: string | number | null): boolean {
  if (value === undefined || value === null) return true
  const str = String(value).trim()
  if (!str) return true

  if (/^\d+%$/.test(str)) {
    const pct = parseInt(str.slice(0, -1), 10)
    return pct >= 0 && pct <= 100
  }

  if (/^\d+$/.test(str)) {
    return true
  }

  return false
}

/**
 * Validates StatefulSet form fields against Kubernetes constraints.
 */
export function validateStatefulSetForm(input: StatefulSetFormValidationInput): boolean {
  if (!input.serviceName?.trim() || !isValidK8sName(input.serviceName.trim())) {
    return false
  }

  if (input.serviceAccountName && !isValidK8sName(input.serviceAccountName)) {
    return false
  }

  if (
    input.strategyType === KUBERNETES_STATEFULSET_UPDATE_STRATEGY.RollingUpdate &&
    input.maxUnavailable &&
    !isValidMaxUnavailable(input.maxUnavailable)
  ) {
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

/**
 * Parses VolumeClaimTemplates from StatefulSet spec for table rendering.
 */
export function parseVolumeClaimTemplates(
  volumeClaimTemplates?: PersistentVolumeClaim[]
): FormattedVolumeClaim[] {
  if (!volumeClaimTemplates || !Array.isArray(volumeClaimTemplates)) return []

  return volumeClaimTemplates.map((pvc: PersistentVolumeClaim) => {
    const name = pvc.metadata?.name || 'Unnamed PVC'
    const spec = pvc.spec || {}
    const storageClassName = spec.storageClassName || 'Default'
    const accessModes = (spec.accessModes || []).join(', ') || 'ReadWriteOnce'
    const storageCapacity = spec.resources?.requests?.storage || 'N/A'
    const volumeMode = spec.volumeMode || 'Filesystem'

    return {
      name,
      storageClassName,
      accessModes,
      storageCapacity,
      volumeMode
    }
  })
}

export interface NormalizedStatefulSetSpec {
  replicas: number
  serviceName: string
  podManagementPolicy: string
  strategyType: string
  partition: number
  maxUnavailable: string
  minReadySeconds: number
  revisionHistoryLimit: number
  selectorLabels: Array<{ key: string; value: string }>
}

/**
 * Normalizes StatefulSet specification parameters for form controls.
 */
export function normalizeStatefulSetSpec(data: StatefulSet | null): NormalizedStatefulSetSpec {
  const spec = data?.spec

  const replicas = typeof spec?.replicas === 'number' ? spec.replicas : 1
  const serviceName = spec?.serviceName || ''
  const podManagementPolicy =
    spec?.podManagementPolicy || KUBERNETES_STATEFULSET_POD_MANAGEMENT_POLICY.OrderedReady

  const strat = spec?.updateStrategy
  const strategyType = strat?.type || KUBERNETES_STATEFULSET_UPDATE_STRATEGY.RollingUpdate
  const rolling = strat?.rollingUpdate

  const partition = typeof rolling?.partition === 'number' ? rolling.partition : 0
  const maxUnavailable =
    rolling?.maxUnavailable !== undefined && rolling?.maxUnavailable !== null
      ? String(rolling.maxUnavailable)
      : '1'

  const minReadySeconds = typeof spec?.minReadySeconds === 'number' ? spec.minReadySeconds : 0
  const revisionHistoryLimit =
    typeof spec?.revisionHistoryLimit === 'number' ? spec.revisionHistoryLimit : 10

  const matchLabels = spec?.selector?.matchLabels || {}
  const selectorLabels = Object.entries(matchLabels).map(([key, value]) => ({
    key,
    value: String(value ?? '')
  }))

  return {
    replicas,
    serviceName,
    podManagementPolicy,
    strategyType,
    partition,
    maxUnavailable,
    minReadySeconds,
    revisionHistoryLimit,
    selectorLabels
  }
}
