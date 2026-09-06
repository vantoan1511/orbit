import type { DaemonSet } from 'kubernetes-types/apps/v1'
import { KUBERNETES_DAEMONSET_UPDATE_STRATEGY } from '../constants/kubernetes.ts'
import { isValidK8sLabel, isValidK8sName, isValidPath } from './validators.ts'
import {
  type ContainerEnvFormItem,
  type WorkloadContainerFormState,
  type WorkloadContainerValidationItem,
  parseContainersFromPodSpec,
  formatContainersToPodSpec
} from './workloadContainer.ts'

export type { ContainerEnvFormItem }
export type DaemonSetContainerFormState = WorkloadContainerFormState
export type DaemonSetContainerValidationItem = WorkloadContainerValidationItem
export { parseContainersFromPodSpec, formatContainersToPodSpec }

export interface DaemonSetFormValidationInput {
  serviceAccountName?: string
  strategyType?: string
  maxUnavailable?: string
  maxSurge?: string
  containers?: DaemonSetContainerValidationItem[]
}

/**
 * Validates Kubernetes IntOrString percentage (0-100%) or non-negative integer.
 */
export function isValidIntOrPercent(value?: string | number | null): boolean {
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
 * Validates Kubernetes maxUnavailable string (percentage 0-100% or non-negative integer).
 */
export function isValidMaxUnavailable(value?: string | number | null): boolean {
  return isValidIntOrPercent(value)
}

/**
 * Validates Kubernetes maxSurge string (percentage 0-100% or non-negative integer).
 */
export function isValidMaxSurge(value?: string | number | null): boolean {
  return isValidIntOrPercent(value)
}

/**
 * Validates DaemonSet form fields against Kubernetes constraints.
 */
export function validateDaemonSetForm(input: DaemonSetFormValidationInput): boolean {
  if (input.serviceAccountName && !isValidK8sName(input.serviceAccountName)) {
    return false
  }

  if (input.strategyType === KUBERNETES_DAEMONSET_UPDATE_STRATEGY.RollingUpdate) {
    if (input.maxUnavailable && !isValidMaxUnavailable(input.maxUnavailable)) {
      return false
    }
    if (input.maxSurge && !isValidMaxSurge(input.maxSurge)) {
      return false
    }
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

export interface NormalizedDaemonSetSpec {
  strategyType: string
  maxUnavailable: string
  maxSurge: string
  minReadySeconds: number
  revisionHistoryLimit: number
  selectorLabels: Array<{ key: string; value: string }>
}

/**
 * Normalizes DaemonSet specification parameters for form controls.
 */
export function normalizeDaemonSetSpec(data: DaemonSet | null): NormalizedDaemonSetSpec {
  const spec = data?.spec

  const strat = spec?.updateStrategy
  const strategyType = strat?.type || KUBERNETES_DAEMONSET_UPDATE_STRATEGY.RollingUpdate
  const rolling = strat?.rollingUpdate

  const maxUnavailable =
    rolling?.maxUnavailable !== undefined && rolling?.maxUnavailable !== null
      ? String(rolling.maxUnavailable)
      : '1'

  const maxSurge =
    rolling?.maxSurge !== undefined && rolling?.maxSurge !== null ? String(rolling.maxSurge) : '0'

  const minReadySeconds = typeof spec?.minReadySeconds === 'number' ? spec.minReadySeconds : 0
  const revisionHistoryLimit =
    typeof spec?.revisionHistoryLimit === 'number' ? spec.revisionHistoryLimit : 10

  const matchLabels = spec?.selector?.matchLabels || {}
  const selectorLabels = Object.entries(matchLabels).map(([key, value]) => ({
    key,
    value: String(value ?? '')
  }))

  return {
    strategyType,
    maxUnavailable,
    maxSurge,
    minReadySeconds,
    revisionHistoryLimit,
    selectorLabels
  }
}
