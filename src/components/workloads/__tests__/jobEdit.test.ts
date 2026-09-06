import test from 'node:test'
import assert from 'node:assert/strict'
import {
  validateJobForm,
  normalizeJobSpec,
  parseContainersFromPodSpec,
  formatContainersToPodSpec
} from '../../../utils/job.ts'
import {
  KUBERNETES_JOB_COMPLETION_MODES,
  KUBERNETES_JOB_COMPLETION_MODE,
  KUBERNETES_POD_REPLACEMENT_POLICIES,
  KUBERNETES_POD_REPLACEMENT_POLICY
} from '../../../constants/kubernetes.ts'
import type { Job } from 'kubernetes-types/batch/v1'
import type { Container } from 'kubernetes-types/core/v1'

test('Job Constants - Completion Modes and Replacement Policies', () => {
  assert.equal(KUBERNETES_JOB_COMPLETION_MODE.NonIndexed, 'NonIndexed')
  assert.equal(KUBERNETES_JOB_COMPLETION_MODE.Indexed, 'Indexed')
  assert.deepEqual(KUBERNETES_JOB_COMPLETION_MODES, ['NonIndexed', 'Indexed'])

  assert.equal(KUBERNETES_POD_REPLACEMENT_POLICY.TerminatingOrFailed, 'TerminatingOrFailed')
  assert.equal(KUBERNETES_POD_REPLACEMENT_POLICY.Failed, 'Failed')
  assert.deepEqual(KUBERNETES_POD_REPLACEMENT_POLICIES, ['TerminatingOrFailed', 'Failed'])
})

test('validateJobForm - valid values', () => {
  const valid = validateJobForm({
    parallelism: 2,
    completions: 4,
    backoffLimit: 6,
    activeDeadlineSeconds: 300,
    ttlSecondsAfterFinished: 3600,
    serviceAccountName: 'job-runner-sa',
    containers: [
      {
        name: 'batch-worker',
        workingDir: '/app',
        ports: [{ name: 'metrics' }]
      }
    ]
  })
  assert.equal(valid, true)
})

test('validateJobForm - allows zero parallelism', () => {
  assert.equal(validateJobForm({ parallelism: 0 }), true)
})

test('validateJobForm - invalid negative parallelism', () => {
  assert.equal(validateJobForm({ parallelism: -1 }), false)
})

test('validateJobForm - invalid decimal parallelism', () => {
  assert.equal(validateJobForm({ parallelism: 2.5 }), false)
})

test('validateJobForm - allows undefined or null completions for NonIndexed mode', () => {
  assert.equal(validateJobForm({ completions: undefined }), true)
  assert.equal(validateJobForm({ completions: null as unknown as undefined }), true)
  assert.equal(
    validateJobForm({
      completionMode: KUBERNETES_JOB_COMPLETION_MODE.NonIndexed,
      completions: undefined
    }),
    true
  )
})

test('validateJobForm - enforces completions for Indexed mode', () => {
  assert.equal(
    validateJobForm({
      completionMode: KUBERNETES_JOB_COMPLETION_MODE.Indexed,
      completions: 3
    }),
    true
  )
  assert.equal(
    validateJobForm({
      completionMode: KUBERNETES_JOB_COMPLETION_MODE.Indexed,
      completions: undefined
    }),
    false
  )
  assert.equal(
    validateJobForm({
      completionMode: KUBERNETES_JOB_COMPLETION_MODE.Indexed,
      completions: null as unknown as undefined
    }),
    false
  )
  assert.equal(
    validateJobForm({
      completionMode: KUBERNETES_JOB_COMPLETION_MODE.Indexed,
      completions: 0
    }),
    false
  )
  assert.equal(
    validateJobForm({
      completionMode: KUBERNETES_JOB_COMPLETION_MODE.Indexed,
      completions: 1.5
    }),
    false
  )
})

test('validateJobForm - invalid zero or negative completions', () => {
  assert.equal(validateJobForm({ completions: 0 }), false)
  assert.equal(validateJobForm({ completions: -2 }), false)
})

test('validateJobForm - invalid decimal completions', () => {
  assert.equal(validateJobForm({ completions: 2.5 }), false)
})

test('validateJobForm - invalid negative backoffLimit', () => {
  assert.equal(validateJobForm({ backoffLimit: -1 }), false)
})

test('validateJobForm - invalid decimal backoffLimit', () => {
  assert.equal(validateJobForm({ backoffLimit: 4.2 }), false)
})

test('validateJobForm - invalid zero or negative activeDeadlineSeconds', () => {
  assert.equal(validateJobForm({ activeDeadlineSeconds: 0 }), false)
  assert.equal(validateJobForm({ activeDeadlineSeconds: -10 }), false)
})

test('validateJobForm - invalid decimal activeDeadlineSeconds', () => {
  assert.equal(validateJobForm({ activeDeadlineSeconds: 12.5 }), false)
})

test('validateJobForm - allows zero ttlSecondsAfterFinished', () => {
  assert.equal(validateJobForm({ ttlSecondsAfterFinished: 0 }), true)
})

test('validateJobForm - invalid negative ttlSecondsAfterFinished', () => {
  assert.equal(validateJobForm({ ttlSecondsAfterFinished: -1 }), false)
})

test('validateJobForm - invalid decimal ttlSecondsAfterFinished', () => {
  assert.equal(validateJobForm({ ttlSecondsAfterFinished: 5.5 }), false)
})

test('validateJobForm - invalid negative backoffLimitPerIndex', () => {
  assert.equal(validateJobForm({ backoffLimitPerIndex: -1 }), false)
})

test('validateJobForm - invalid negative maxFailedIndexes', () => {
  assert.equal(validateJobForm({ maxFailedIndexes: -1 }), false)
})

test('validateJobForm - invalid serviceAccountName', () => {
  assert.equal(validateJobForm({ serviceAccountName: 'INVALID SA' }), false)
})

test('validateJobForm - invalid container fields', () => {
  assert.equal(
    validateJobForm({
      containers: [{ name: 'INVALID_CONTAINER' }]
    }),
    false
  )

  assert.equal(
    validateJobForm({
      containers: [{ name: 'valid-container', workingDir: 'invalid-rel-path' }]
    }),
    false
  )

  assert.equal(
    validateJobForm({
      containers: [{ name: '' }]
    }),
    false
  )

  assert.equal(
    validateJobForm({
      containers: [{ name: '   ' }]
    }),
    false
  )

  assert.equal(
    validateJobForm({
      containers: [
        {
          name: 'valid-container',
          workingDir: '/app',
          ports: [{ name: 'INVALID_PORT_NAME' }]
        }
      ]
    }),
    false
  )
})

test('normalizeJobSpec - defaults for empty data', () => {
  const norm = normalizeJobSpec(null)
  assert.equal(norm.parallelism, 1)
  assert.equal(norm.completions, null)
  assert.equal(norm.backoffLimit, 6)
  assert.equal(norm.activeDeadlineSeconds, null)
  assert.equal(norm.ttlSecondsAfterFinished, null)
  assert.equal(norm.suspend, false)
  assert.equal(norm.completionMode, KUBERNETES_JOB_COMPLETION_MODE.NonIndexed)
  assert.equal(norm.backoffLimitPerIndex, null)
  assert.equal(norm.maxFailedIndexes, null)
  assert.equal(norm.podReplacementPolicy, '')
  assert.deepEqual(norm.selectorLabels, [])
})

test('normalizeJobSpec - preserves unset completions for work queue jobs', () => {
  const workQueueJob: Job = {
    apiVersion: 'batch/v1',
    kind: 'Job',
    metadata: { name: 'queue-processor', namespace: 'default' },
    spec: {
      parallelism: 5,
      template: { spec: { containers: [] } }
    }
  }
  const norm = normalizeJobSpec(workQueueJob)
  assert.equal(norm.parallelism, 5)
  assert.equal(norm.completions, null)
  assert.equal(norm.completionMode, KUBERNETES_JOB_COMPLETION_MODE.NonIndexed)
})

test('normalizeJobSpec - populated data', () => {
  const rawJob: Job = {
    apiVersion: 'batch/v1',
    kind: 'Job',
    metadata: {
      name: 'batch-worker-job',
      namespace: 'production'
    },
    spec: {
      parallelism: 3,
      completions: 10,
      backoffLimit: 4,
      activeDeadlineSeconds: 1200,
      ttlSecondsAfterFinished: 600,
      suspend: true,
      completionMode: 'Indexed',
      backoffLimitPerIndex: 2,
      maxFailedIndexes: 1,
      podReplacementPolicy: 'Failed',
      selector: {
        matchLabels: {
          'batch.kubernetes.io/job-name': 'batch-worker-job',
          tier: 'backend'
        }
      },
      template: {
        spec: {
          restartPolicy: 'OnFailure',
          containers: []
        }
      }
    }
  }

  const norm = normalizeJobSpec(rawJob)
  assert.equal(norm.parallelism, 3)
  assert.equal(norm.completions, 10)
  assert.equal(norm.backoffLimit, 4)
  assert.equal(norm.activeDeadlineSeconds, 1200)
  assert.equal(norm.ttlSecondsAfterFinished, 600)
  assert.equal(norm.suspend, true)
  assert.equal(norm.completionMode, 'Indexed')
  assert.equal(norm.backoffLimitPerIndex, 2)
  assert.equal(norm.maxFailedIndexes, 1)
  assert.equal(norm.podReplacementPolicy, 'Failed')
  assert.equal(norm.selectorLabels.length, 2)
  assert.deepEqual(norm.selectorLabels[0], {
    key: 'batch.kubernetes.io/job-name',
    value: 'batch-worker-job'
  })
})

test('parseContainersFromPodSpec & formatContainersToPodSpec roundtrip', () => {
  const rawContainers: Container[] = [
    {
      name: 'worker',
      image: 'busybox:latest',
      command: ['sh', '-c'],
      args: ['echo hello; sleep 10'],
      workingDir: '/workspace',
      env: [{ name: 'JOB_ENV', value: 'production' }],
      ports: [{ containerPort: 9090, protocol: 'TCP', name: 'http-metrics' }],
      resources: {
        requests: { cpu: '100m', memory: '128Mi' },
        limits: { cpu: '500m', memory: '512Mi' }
      }
    }
  ]

  const formState = parseContainersFromPodSpec(rawContainers)
  assert.equal(formState.length, 1)
  assert.equal(formState[0]?.name, 'worker')
  assert.equal(formState[0]?.cpuRequest, '100m')
  assert.equal(formState[0]?.memoryLimit, '512Mi')
  assert.equal(formState[0]?.command.length, 2)
  assert.equal(formState[0]?.args.length, 1)

  const roundtripped = formatContainersToPodSpec(formState)
  assert.equal(roundtripped.length, 1)
  assert.equal(roundtripped[0]?.name, 'worker')
  assert.equal(roundtripped[0]?.image, 'busybox:latest')
  assert.deepEqual(roundtripped[0]?.command, ['sh', '-c'])
  assert.deepEqual(roundtripped[0]?.args, ['echo hello; sleep 10'])
  assert.equal(roundtripped[0]?.workingDir, '/workspace')
  assert.deepEqual(roundtripped[0]?.env, [{ name: 'JOB_ENV', value: 'production' }])
  assert.deepEqual(roundtripped[0]?.ports, [
    { containerPort: 9090, protocol: 'TCP', name: 'http-metrics' }
  ])
  assert.deepEqual(roundtripped[0]?.resources, {
    requests: { cpu: '100m', memory: '128Mi' },
    limits: { cpu: '500m', memory: '512Mi' }
  })
})
