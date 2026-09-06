import test from 'node:test'
import assert from 'node:assert/strict'
import {
  validateReplicaSetForm,
  normalizeReplicaSetSpec,
  parseContainersFromPodSpec,
  formatContainersToPodSpec
} from '../../../utils/replicaSet.ts'
import type { ReplicaSet } from 'kubernetes-types/apps/v1'
import type { Container } from 'kubernetes-types/core/v1'

test('validateReplicaSetForm - valid values', () => {
  const valid = validateReplicaSetForm({
    replicas: 3,
    minReadySeconds: 10,
    serviceAccountName: 'frontend-sa',
    containers: [
      {
        name: 'frontend-container',
        workingDir: '/app',
        ports: [{ name: 'http' }]
      }
    ]
  })
  assert.equal(valid, true)
})

test('validateReplicaSetForm - allows zero replicas', () => {
  const valid = validateReplicaSetForm({
    replicas: 0
  })
  assert.equal(valid, true)
})

test('validateReplicaSetForm - invalid negative replicas', () => {
  assert.equal(validateReplicaSetForm({ replicas: -1 }), false)
})

test('validateReplicaSetForm - invalid decimal replicas', () => {
  assert.equal(validateReplicaSetForm({ replicas: 2.5 }), false)
})

test('validateReplicaSetForm - invalid negative minReadySeconds', () => {
  assert.equal(validateReplicaSetForm({ minReadySeconds: -5 }), false)
})

test('validateReplicaSetForm - invalid decimal minReadySeconds', () => {
  assert.equal(validateReplicaSetForm({ minReadySeconds: 3.14 }), false)
})

test('validateReplicaSetForm - invalid serviceAccountName', () => {
  const invalid = validateReplicaSetForm({
    serviceAccountName: 'INVALID SA',
    containers: []
  })
  assert.equal(invalid, false)
})

test('validateReplicaSetForm - invalid container fields', () => {
  assert.equal(
    validateReplicaSetForm({
      containers: [{ name: 'INVALID_CONTAINER' }]
    }),
    false
  )

  assert.equal(
    validateReplicaSetForm({
      containers: [{ name: 'valid-container', workingDir: 'invalid-rel-path' }]
    }),
    false
  )

  assert.equal(
    validateReplicaSetForm({
      containers: [{ name: '' }]
    }),
    false
  )

  assert.equal(
    validateReplicaSetForm({
      containers: [{ name: '   ' }]
    }),
    false
  )

  assert.equal(
    validateReplicaSetForm({
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

test('normalizeReplicaSetSpec - defaults for empty data', () => {
  const normalized = normalizeReplicaSetSpec(null)
  assert.equal(normalized.replicas, 1)
  assert.equal(normalized.minReadySeconds, 0)
  assert.deepEqual(normalized.selectorLabels, [])

  const normalizedEmpty = normalizeReplicaSetSpec({} as ReplicaSet)
  assert.equal(normalizedEmpty.replicas, 1)
  assert.equal(normalizedEmpty.minReadySeconds, 0)
  assert.deepEqual(normalizedEmpty.selectorLabels, [])
})

test('normalizeReplicaSetSpec - populated data', () => {
  const rs: ReplicaSet = {
    apiVersion: 'apps/v1',
    kind: 'ReplicaSet',
    metadata: {
      name: 'frontend-rs',
      namespace: 'production'
    },
    spec: {
      replicas: 5,
      minReadySeconds: 15,
      selector: {
        matchLabels: {
          app: 'frontend',
          env: 'prod'
        }
      }
    }
  }

  const normalized = normalizeReplicaSetSpec(rs)
  assert.equal(normalized.replicas, 5)
  assert.equal(normalized.minReadySeconds, 15)
  assert.deepEqual(normalized.selectorLabels, [
    { key: 'app', value: 'frontend' },
    { key: 'env', value: 'prod' }
  ])
})

test('parseContainersFromPodSpec & formatContainersToPodSpec roundtrip', () => {
  const originalContainers: Container[] = [
    {
      name: 'web-server',
      image: 'nginx:alpine',
      imagePullPolicy: 'IfNotPresent',
      workingDir: '/usr/share/nginx/html',
      command: ['nginx'],
      args: ['-g', 'daemon off;'],
      ports: [{ name: 'http', containerPort: 80, protocol: 'TCP' }],
      env: [{ name: 'PORT', value: '80' }],
      resources: {
        requests: { cpu: '100m', memory: '128Mi' },
        limits: { cpu: '500m', memory: '512Mi' }
      }
    }
  ]

  const parsed = parseContainersFromPodSpec(originalContainers)
  assert.equal(parsed.length, 1)
  assert.equal(parsed[0].name, 'web-server')
  assert.equal(parsed[0].cpuRequest, '100m')
  assert.equal(parsed[0].memoryLimit, '512Mi')

  const formatted = formatContainersToPodSpec(parsed)
  assert.equal(formatted.length, 1)
  assert.equal(formatted[0].name, 'web-server')
  assert.equal(formatted[0].image, 'nginx:alpine')
  assert.equal(formatted[0].workingDir, '/usr/share/nginx/html')
  assert.deepEqual(formatted[0].command, ['nginx'])
  assert.deepEqual(formatted[0].args, ['-g', 'daemon off;'])
  assert.equal(formatted[0].resources?.requests?.cpu, '100m')
  assert.equal(formatted[0].resources?.limits?.memory, '512Mi')
})
