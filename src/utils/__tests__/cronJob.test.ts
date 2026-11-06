import test from 'node:test'
import assert from 'node:assert/strict'
import type { CronJob } from 'kubernetes-types/batch/v1'
import type { Container } from 'kubernetes-types/core/v1'
import { validateCronJobForm, normalizeCronJobSpec } from '../cronJob.ts'

test('validateCronJobForm accepts valid schedule, concurrencyPolicy, limits, and containers', () => {
  const valid = validateCronJobForm({
    schedule: '*/5 * * * *',
    concurrencyPolicy: 'Allow',
    startingDeadlineSeconds: 100,
    successfulJobsHistoryLimit: 3,
    failedJobsHistoryLimit: 1,
    parallelism: 1,
    completions: 1,
    backoffLimit: 6,
    activeDeadlineSeconds: 60,
    ttlSecondsAfterFinished: 300,
    serviceAccountName: 'my-sa',
    containers: [
      {
        name: 'worker',
        image: 'busybox:latest',
        imagePullPolicy: 'IfNotPresent',
        commands: 'echo hello',
        args: '',
        workingDir: '/workspace',
        cpuRequest: '',
        cpuLimit: '',
        memoryRequest: '',
        memoryLimit: '',
        ports: [],
        env: []
      }
    ]
  })
  assert.equal(valid, true)
})

test('validateCronJobForm accepts valid macro schedule like @daily', () => {
  const valid = validateCronJobForm({
    schedule: '@daily',
    concurrencyPolicy: 'Forbid'
  })
  assert.equal(valid, true)
})

test('validateCronJobForm rejects invalid cron expression', () => {
  const valid = validateCronJobForm({
    schedule: 'invalid cron expression',
    concurrencyPolicy: 'Allow'
  })
  assert.equal(valid, false)
})

test('validateCronJobForm rejects invalid concurrencyPolicy', () => {
  const valid = validateCronJobForm({
    schedule: '0 * * * *',
    concurrencyPolicy: 'InvalidPolicy'
  })
  assert.equal(valid, false)
})

test('validateCronJobForm rejects negative history limits', () => {
  assert.equal(
    validateCronJobForm({
      schedule: '0 * * * *',
      successfulJobsHistoryLimit: -1
    }),
    false
  )
  assert.equal(
    validateCronJobForm({
      schedule: '0 * * * *',
      failedJobsHistoryLimit: -2
    }),
    false
  )
})

test('validateCronJobForm rejects non-integer or zero/negative startingDeadlineSeconds', () => {
  assert.equal(
    validateCronJobForm({
      schedule: '0 * * * *',
      startingDeadlineSeconds: 0
    }),
    false
  )
  assert.equal(
    validateCronJobForm({
      schedule: '0 * * * *',
      startingDeadlineSeconds: -10
    }),
    false
  )
  assert.equal(
    validateCronJobForm({
      schedule: '0 * * * *',
      startingDeadlineSeconds: 1.5
    }),
    false
  )
})

test('validateCronJobForm rejects invalid serviceAccountName or invalid container name', () => {
  assert.equal(
    validateCronJobForm({
      schedule: '0 * * * *',
      serviceAccountName: 'INVALID_SA_NAME!'
    }),
    false
  )

  assert.equal(
    validateCronJobForm({
      schedule: '0 * * * *',
      containers: [
        {
          name: 'INVALID_CONTAINER_NAME!'
        }
      ]
    }),
    false
  )
})

test('normalizeCronJobSpec normalizes missing/partial CronJob with defaults', () => {
  const normalized = normalizeCronJobSpec(null)
  assert.deepEqual(normalized, {
    schedule: '*/5 * * * *',
    concurrencyPolicy: 'Allow',
    suspend: false,
    startingDeadlineSeconds: null,
    successfulJobsHistoryLimit: 3,
    failedJobsHistoryLimit: 1,
    parallelism: 1,
    completions: null,
    backoffLimit: 6,
    activeDeadlineSeconds: null,
    ttlSecondsAfterFinished: null
  })
})

test('normalizeCronJobSpec preserves explicit CronJob spec and jobTemplate values', () => {
  const cj: CronJob = {
    metadata: { name: 'backup', namespace: 'prod' },
    spec: {
      schedule: '0 2 * * *',
      concurrencyPolicy: 'Forbid',
      suspend: true,
      startingDeadlineSeconds: 200,
      successfulJobsHistoryLimit: 5,
      failedJobsHistoryLimit: 2,
      jobTemplate: {
        spec: {
          parallelism: 2,
          completions: 4,
          backoffLimit: 3,
          activeDeadlineSeconds: 120,
          ttlSecondsAfterFinished: 600,
          template: {
            spec: {
              containers: [{ name: 'db-backup', image: 'postgres:15' }]
            }
          }
        }
      }
    }
  }

  const normalized = normalizeCronJobSpec(cj)
  assert.deepEqual(normalized, {
    schedule: '0 2 * * *',
    concurrencyPolicy: 'Forbid',
    suspend: true,
    startingDeadlineSeconds: 200,
    successfulJobsHistoryLimit: 5,
    failedJobsHistoryLimit: 2,
    parallelism: 2,
    completions: 4,
    backoffLimit: 3,
    activeDeadlineSeconds: 120,
    ttlSecondsAfterFinished: 600
  })
})

test('roundtrip serialization parses and formats containers cleanly without loss', async () => {
  const { parseContainersFromPodSpec, formatContainersToPodSpec } = await import('../cronJob.ts')
  const initialContainers: Container[] = [
    {
      name: 'sync-worker',
      image: 'busybox:latest',
      imagePullPolicy: 'Always',
      command: ['/bin/sh'],
      args: ['-c', 'echo running'],
      workingDir: '/workspace',
      env: [{ name: 'ENV_MODE', value: 'production' }],
      ports: [{ name: 'metrics', containerPort: 9090, protocol: 'TCP' }],
      resources: {
        requests: { cpu: '100m', memory: '128Mi' },
        limits: { cpu: '500m', memory: '512Mi' }
      }
    }
  ]

  const parsed = parseContainersFromPodSpec(initialContainers)
  assert.equal(parsed.length, 1)
  assert.equal(parsed[0].name, 'sync-worker')
  assert.equal(parsed[0].cpuRequest, '100m')
  assert.equal(parsed[0].memoryLimit, '512Mi')

  const formatted = formatContainersToPodSpec(parsed)
  assert.equal(formatted.length, 1)
  assert.equal(formatted[0].name, 'sync-worker')
  assert.equal(formatted[0].image, 'busybox:latest')
  assert.equal(formatted[0].imagePullPolicy, 'Always')
  assert.deepEqual(formatted[0].command, ['/bin/sh'])
  assert.deepEqual(formatted[0].args, ['-c', 'echo running'])
  assert.equal(formatted[0].workingDir, '/workspace')
  assert.equal(formatted[0].resources?.requests?.cpu, '100m')
  assert.equal(formatted[0].resources?.limits?.memory, '512Mi')
})
