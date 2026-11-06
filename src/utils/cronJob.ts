import type { CronJob } from 'kubernetes-types/batch/v1'
import { isValidCron, isValidK8sLabel, isValidK8sName, isValidPath } from './validators.ts'
import {
  type WorkloadContainerFormState,
  type WorkloadContainerValidationItem,
  parseContainersFromPodSpec,
  formatContainersToPodSpec
} from './workloadContainer.ts'

export type CronJobContainerFormState = WorkloadContainerFormState
export type CronJobContainerValidationItem = WorkloadContainerValidationItem
export { parseContainersFromPodSpec, formatContainersToPodSpec }

export const KUBERNETES_CONCURRENCY_POLICIES: string[] = ['Allow', 'Forbid', 'Replace']
export type KubernetesConcurrencyPolicy = 'Allow' | 'Forbid' | 'Replace'

export interface CronJobFormValidationInput {
  schedule?: string
  concurrencyPolicy?: string
  startingDeadlineSeconds?: number | null
  successfulJobsHistoryLimit?: number | null
  failedJobsHistoryLimit?: number | null
  parallelism?: number | null
  completions?: number | null
  backoffLimit?: number | null
  activeDeadlineSeconds?: number | null
  ttlSecondsAfterFinished?: number | null
  serviceAccountName?: string
  containers?: WorkloadContainerValidationItem[]
}

export interface NormalizedCronJobSpec {
  schedule: string
  concurrencyPolicy: string
  suspend: boolean
  startingDeadlineSeconds: number | null
  successfulJobsHistoryLimit: number
  failedJobsHistoryLimit: number
  parallelism: number
  completions: number | null
  backoffLimit: number
  activeDeadlineSeconds: number | null
  ttlSecondsAfterFinished: number | null
}

/**
 * Validates CronJob form fields against Kubernetes constraints.
 */
export function validateCronJobForm(input: CronJobFormValidationInput): boolean {
  if (input.schedule !== undefined) {
    if (!isValidCron(input.schedule)) {
      return false
    }
  }

  if (input.concurrencyPolicy !== undefined) {
    if (
      !KUBERNETES_CONCURRENCY_POLICIES.includes(
        input.concurrencyPolicy as KubernetesConcurrencyPolicy
      )
    ) {
      return false
    }
  }

  if (
    input.startingDeadlineSeconds !== undefined &&
    input.startingDeadlineSeconds !== null &&
    (typeof input.startingDeadlineSeconds !== 'number' ||
      isNaN(input.startingDeadlineSeconds) ||
      input.startingDeadlineSeconds < 1 ||
      !Number.isInteger(input.startingDeadlineSeconds))
  ) {
    return false
  }

  if (
    input.successfulJobsHistoryLimit !== undefined &&
    input.successfulJobsHistoryLimit !== null &&
    (typeof input.successfulJobsHistoryLimit !== 'number' ||
      isNaN(input.successfulJobsHistoryLimit) ||
      input.successfulJobsHistoryLimit < 0 ||
      !Number.isInteger(input.successfulJobsHistoryLimit))
  ) {
    return false
  }

  if (
    input.failedJobsHistoryLimit !== undefined &&
    input.failedJobsHistoryLimit !== null &&
    (typeof input.failedJobsHistoryLimit !== 'number' ||
      isNaN(input.failedJobsHistoryLimit) ||
      input.failedJobsHistoryLimit < 0 ||
      !Number.isInteger(input.failedJobsHistoryLimit))
  ) {
    return false
  }

  if (
    input.parallelism !== undefined &&
    input.parallelism !== null &&
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

  if (input.serviceAccountName && !isValidK8sName(input.serviceAccountName)) {
    return false
  }

  if (input.containers && input.containers.length > 0) {
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
 * Normalizes CronJob specification parameters for form controls.
 */
export function normalizeCronJobSpec(data: CronJob | null): NormalizedCronJobSpec {
  const spec = data?.spec
  const jobSpec = spec?.jobTemplate?.spec

  const schedule = spec?.schedule || '*/5 * * * *'
  const concurrencyPolicy = spec?.concurrencyPolicy || 'Allow'
  const suspend = Boolean(spec?.suspend)
  const startingDeadlineSeconds =
    typeof spec?.startingDeadlineSeconds === 'number' ? spec.startingDeadlineSeconds : null
  const successfulJobsHistoryLimit =
    typeof spec?.successfulJobsHistoryLimit === 'number' ? spec.successfulJobsHistoryLimit : 3
  const failedJobsHistoryLimit =
    typeof spec?.failedJobsHistoryLimit === 'number' ? spec.failedJobsHistoryLimit : 1

  const parallelism = typeof jobSpec?.parallelism === 'number' ? jobSpec.parallelism : 1
  const completions = typeof jobSpec?.completions === 'number' ? jobSpec.completions : null
  const backoffLimit = typeof jobSpec?.backoffLimit === 'number' ? jobSpec.backoffLimit : 6
  const activeDeadlineSeconds =
    typeof jobSpec?.activeDeadlineSeconds === 'number' ? jobSpec.activeDeadlineSeconds : null
  const ttlSecondsAfterFinished =
    typeof jobSpec?.ttlSecondsAfterFinished === 'number' ? jobSpec.ttlSecondsAfterFinished : null

  return {
    schedule,
    concurrencyPolicy,
    suspend,
    startingDeadlineSeconds,
    successfulJobsHistoryLimit,
    failedJobsHistoryLimit,
    parallelism,
    completions,
    backoffLimit,
    activeDeadlineSeconds,
    ttlSecondsAfterFinished
  }
}
