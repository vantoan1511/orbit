import type { Job } from 'kubernetes-types/batch/v1'
import { KUBERNETES_JOB_COMPLETION_MODE } from '../constants/kubernetes.ts'
import { isValidK8sLabel, isValidK8sName, isValidPath } from './validators.ts'
import {
  type ContainerEnvFormItem,
  type WorkloadContainerFormState,
  type WorkloadContainerValidationItem,
  parseContainersFromPodSpec,
  formatContainersToPodSpec
} from './workloadContainer.ts'

export type { ContainerEnvFormItem }
export type JobContainerFormState = WorkloadContainerFormState
export type JobContainerValidationItem = WorkloadContainerValidationItem
export { parseContainersFromPodSpec, formatContainersToPodSpec }

export interface JobFormValidationInput {
  parallelism?: number
  completions?: number
  completionMode?: string
  backoffLimit?: number
  activeDeadlineSeconds?: number
  ttlSecondsAfterFinished?: number
  backoffLimitPerIndex?: number
  maxFailedIndexes?: number
  serviceAccountName?: string
  containers?: JobContainerValidationItem[]
}

/**
 * Validates Job form fields against Kubernetes constraints.
 */
export function validateJobForm(input: JobFormValidationInput): boolean {
  if (input.completionMode === KUBERNETES_JOB_COMPLETION_MODE.Indexed) {
    if (
      input.completions === undefined ||
      input.completions === null ||
      typeof input.completions !== 'number' ||
      isNaN(input.completions) ||
      input.completions < 1 ||
      !Number.isInteger(input.completions)
    ) {
      return false
    }
  }

  if (
    input.parallelism !== undefined &&
    (typeof input.parallelism !== 'number' ||
      isNaN(input.parallelism) ||
      input.parallelism < 0 ||
      !Number.isInteger(input.parallelism))
  ) {
    return false
  }

  if (
    input.completions !== undefined &&
    input.completions !== null &&
    (typeof input.completions !== 'number' ||
      isNaN(input.completions) ||
      input.completions < 1 ||
      !Number.isInteger(input.completions))
  ) {
    return false
  }

  if (
    input.backoffLimit !== undefined &&
    input.backoffLimit !== null &&
    (typeof input.backoffLimit !== 'number' ||
      isNaN(input.backoffLimit) ||
      input.backoffLimit < 0 ||
      !Number.isInteger(input.backoffLimit))
  ) {
    return false
  }

  if (
    input.activeDeadlineSeconds !== undefined &&
    input.activeDeadlineSeconds !== null &&
    (typeof input.activeDeadlineSeconds !== 'number' ||
      isNaN(input.activeDeadlineSeconds) ||
      input.activeDeadlineSeconds < 1 ||
      !Number.isInteger(input.activeDeadlineSeconds))
  ) {
    return false
  }

  if (
    input.ttlSecondsAfterFinished !== undefined &&
    input.ttlSecondsAfterFinished !== null &&
    (typeof input.ttlSecondsAfterFinished !== 'number' ||
      isNaN(input.ttlSecondsAfterFinished) ||
      input.ttlSecondsAfterFinished < 0 ||
      !Number.isInteger(input.ttlSecondsAfterFinished))
  ) {
    return false
  }

  if (
    input.backoffLimitPerIndex !== undefined &&
    input.backoffLimitPerIndex !== null &&
    (typeof input.backoffLimitPerIndex !== 'number' ||
      isNaN(input.backoffLimitPerIndex) ||
      input.backoffLimitPerIndex < 0 ||
      !Number.isInteger(input.backoffLimitPerIndex))
  ) {
    return false
  }

  if (
    input.maxFailedIndexes !== undefined &&
    input.maxFailedIndexes !== null &&
    (typeof input.maxFailedIndexes !== 'number' ||
      isNaN(input.maxFailedIndexes) ||
      input.maxFailedIndexes < 0 ||
      !Number.isInteger(input.maxFailedIndexes))
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

export interface NormalizedJobSpec {
  parallelism: number
  completions: number | null
  backoffLimit: number
  activeDeadlineSeconds: number | null
  ttlSecondsAfterFinished: number | null
  suspend: boolean
  completionMode: string
  backoffLimitPerIndex: number | null
  maxFailedIndexes: number | null
  podReplacementPolicy: string
  manualSelector: boolean
  selectorLabels: Array<{ key: string; value: string }>
}

/**
 * Normalizes Job specification parameters for form controls.
 */
export function normalizeJobSpec(data: Job | null): NormalizedJobSpec {
  const spec = data?.spec

  const parallelism = typeof spec?.parallelism === 'number' ? spec.parallelism : 1
  const completions = typeof spec?.completions === 'number' ? spec.completions : null
  const backoffLimit = typeof spec?.backoffLimit === 'number' ? spec.backoffLimit : 6
  const activeDeadlineSeconds =
    typeof spec?.activeDeadlineSeconds === 'number' ? spec.activeDeadlineSeconds : null
  const ttlSecondsAfterFinished =
    typeof spec?.ttlSecondsAfterFinished === 'number' ? spec.ttlSecondsAfterFinished : null
  const suspend = Boolean(spec?.suspend)
  const completionMode = spec?.completionMode || KUBERNETES_JOB_COMPLETION_MODE.NonIndexed
  const backoffLimitPerIndex =
    typeof spec?.backoffLimitPerIndex === 'number' ? spec.backoffLimitPerIndex : null
  const maxFailedIndexes = typeof spec?.maxFailedIndexes === 'number' ? spec.maxFailedIndexes : null
  const podReplacementPolicy = spec?.podReplacementPolicy || ''
  const manualSelector = Boolean(spec?.manualSelector)

  const matchLabels = spec?.selector?.matchLabels
  const selectorLabels: Array<{ key: string; value: string }> = []
  if (matchLabels && typeof matchLabels === 'object') {
    for (const [key, value] of Object.entries(matchLabels)) {
      selectorLabels.push({ key, value: String(value ?? '') })
    }
  }

  return {
    parallelism,
    completions,
    backoffLimit,
    activeDeadlineSeconds,
    ttlSecondsAfterFinished,
    suspend,
    completionMode,
    backoffLimitPerIndex,
    maxFailedIndexes,
    podReplacementPolicy,
    manualSelector,
    selectorLabels
  }
}
