import test from 'node:test'
import assert from 'node:assert/strict'
import {
  KUBERNETES_STATEFULSET_UPDATE_STRATEGY,
  KUBERNETES_STATEFULSET_UPDATE_STRATEGIES,
  KUBERNETES_STATEFULSET_POD_MANAGEMENT_POLICY,
  KUBERNETES_STATEFULSET_POD_MANAGEMENT_POLICIES
} from '../../../constants/kubernetes.ts'
import {
  validateStatefulSetForm,
  isValidMaxUnavailable,
  parseVolumeClaimTemplates,
  normalizeStatefulSetSpec,
  parseContainersFromPodSpec,
  formatContainersToPodSpec
} from '../../../utils/statefulSet.ts'
import type { StatefulSet } from 'kubernetes-types/apps/v1'
import type { Container, PersistentVolumeClaim } from 'kubernetes-types/core/v1'

test('StatefulSet Constants - Update Strategies', () => {
  assert.equal(KUBERNETES_STATEFULSET_UPDATE_STRATEGY.RollingUpdate, 'RollingUpdate')
  assert.equal(KUBERNETES_STATEFULSET_UPDATE_STRATEGY.OnDelete, 'OnDelete')
  assert.deepEqual(KUBERNETES_STATEFULSET_UPDATE_STRATEGIES, ['RollingUpdate', 'OnDelete'])
})

test('StatefulSet Constants - Pod Management Policies', () => {
  assert.equal(KUBERNETES_STATEFULSET_POD_MANAGEMENT_POLICY.OrderedReady, 'OrderedReady')
  assert.equal(KUBERNETES_STATEFULSET_POD_MANAGEMENT_POLICY.Parallel, 'Parallel')
  assert.deepEqual(KUBERNETES_STATEFULSET_POD_MANAGEMENT_POLICIES, ['OrderedReady', 'Parallel'])
})

test('isValidMaxUnavailable - percentage and integer validation', () => {
  assert.equal(isValidMaxUnavailable('1'), true)
  assert.equal(isValidMaxUnavailable('0'), true)
  assert.equal(isValidMaxUnavailable('25%'), true)
  assert.equal(isValidMaxUnavailable('100%'), true)
  assert.equal(isValidMaxUnavailable(''), true)
  assert.equal(isValidMaxUnavailable(null), true)
  assert.equal(isValidMaxUnavailable(undefined), true)

  assert.equal(isValidMaxUnavailable('invalid'), false)
  assert.equal(isValidMaxUnavailable('-1'), false)
  assert.equal(isValidMaxUnavailable('150%'), false)
  assert.equal(isValidMaxUnavailable('25 %'), false)
})

test('validateStatefulSetForm - valid values', () => {
  const valid = validateStatefulSetForm({
    serviceName: 'web-service',
    serviceAccountName: 'web-sa',
    strategyType: 'RollingUpdate',
    maxUnavailable: '25%',
    containers: [
      {
        name: 'web-container',
        workingDir: '/app',
        ports: [{ name: 'http' }]
      }
    ]
  })
  assert.equal(valid, true)
})

test('validateStatefulSetForm - missing or empty serviceName', () => {
  assert.equal(validateStatefulSetForm({ serviceName: '' }), false)
  assert.equal(validateStatefulSetForm({ serviceName: '   ' }), false)
  assert.equal(validateStatefulSetForm({}), false)
})

test('validateStatefulSetForm - invalid serviceName', () => {
  const invalid = validateStatefulSetForm({
    serviceName: 'INVALID_SERVICE_NAME!',
    containers: []
  })
  assert.equal(invalid, false)
})

test('validateStatefulSetForm - invalid maxUnavailable', () => {
  assert.equal(
    validateStatefulSetForm({
      serviceName: 'web-service',
      strategyType: 'RollingUpdate',
      maxUnavailable: 'not-valid'
    }),
    false
  )
})

test('validateStatefulSetForm - invalid serviceAccountName', () => {
  const invalid = validateStatefulSetForm({
    serviceName: 'web-service',
    serviceAccountName: 'INVALID SA',
    containers: []
  })
  assert.equal(invalid, false)
})

test('validateStatefulSetForm - invalid container fields', () => {
  assert.equal(
    validateStatefulSetForm({
      serviceName: 'web-service',
      containers: [{ name: 'INVALID_CONTAINER' }]
    }),
    false
  )

  assert.equal(
    validateStatefulSetForm({
      serviceName: 'web-service',
      containers: [{ name: 'valid-container', workingDir: 'invalid-rel-path' }]
    }),
    false
  )

  assert.equal(
    validateStatefulSetForm({
      serviceName: 'web-service',
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

test('parseVolumeClaimTemplates - empty or undefined', () => {
  assert.deepEqual(parseVolumeClaimTemplates(undefined), [])
  assert.deepEqual(parseVolumeClaimTemplates([]), [])
})

test('parseVolumeClaimTemplates - parses PVC templates', () => {
  const pvcs: PersistentVolumeClaim[] = [
    {
      metadata: { name: 'data' },
      spec: {
        accessModes: ['ReadWriteOnce'],
        storageClassName: 'standard',
        volumeMode: 'Filesystem',
        resources: {
          requests: { storage: '10Gi' }
        }
      }
    }
  ]

  const parsed = parseVolumeClaimTemplates(pvcs)
  assert.equal(parsed.length, 1)
  assert.equal(parsed[0].name, 'data')
  assert.equal(parsed[0].storageClassName, 'standard')
  assert.equal(parsed[0].accessModes, 'ReadWriteOnce')
  assert.equal(parsed[0].storageCapacity, '10Gi')
  assert.equal(parsed[0].volumeMode, 'Filesystem')
})

test('normalizeStatefulSetSpec - defaults for empty data', () => {
  const normalized = normalizeStatefulSetSpec(null)
  assert.equal(normalized.replicas, 1)
  assert.equal(normalized.serviceName, '')
  assert.equal(normalized.podManagementPolicy, 'OrderedReady')
  assert.equal(normalized.strategyType, 'RollingUpdate')
  assert.equal(normalized.partition, 0)
  assert.equal(normalized.maxUnavailable, '1')
  assert.equal(normalized.minReadySeconds, 0)
  assert.equal(normalized.revisionHistoryLimit, 10)
  assert.deepEqual(normalized.selectorLabels, [])
})

test('normalizeStatefulSetSpec - populated data', () => {
  const sample: StatefulSet = {
    metadata: {
      name: 'redis',
      namespace: 'prod'
    },
    spec: {
      serviceName: 'redis-headless',
      replicas: 3,
      podManagementPolicy: 'Parallel',
      updateStrategy: {
        type: 'RollingUpdate',
        rollingUpdate: {
          partition: 1,
          maxUnavailable: '25%'
        }
      },
      minReadySeconds: 5,
      revisionHistoryLimit: 15,
      selector: {
        matchLabels: {
          app: 'redis'
        }
      },
      template: {
        spec: {
          containers: []
        }
      }
    }
  }

  const normalized = normalizeStatefulSetSpec(sample)
  assert.equal(normalized.replicas, 3)
  assert.equal(normalized.serviceName, 'redis-headless')
  assert.equal(normalized.podManagementPolicy, 'Parallel')
  assert.equal(normalized.strategyType, 'RollingUpdate')
  assert.equal(normalized.partition, 1)
  assert.equal(normalized.maxUnavailable, '25%')
  assert.equal(normalized.minReadySeconds, 5)
  assert.equal(normalized.revisionHistoryLimit, 15)
  assert.deepEqual(normalized.selectorLabels, [{ key: 'app', value: 'redis' }])
})

test('parseContainersFromPodSpec & formatContainersToPodSpec roundtrip', () => {
  const rawContainers: Container[] = [
    {
      name: 'web',
      image: 'nginx:alpine',
      imagePullPolicy: 'Always',
      workingDir: '/var/www',
      command: ['nginx'],
      args: ['-g', 'daemon off;'],
      env: [
        { name: 'PORT', value: '8080' },
        {
          name: 'DB_USER',
          valueFrom: {
            secretKeyRef: { name: 'db-secret', key: 'username' }
          }
        }
      ],
      envFrom: [{ configMapRef: { name: 'app-config' } }],
      ports: [{ name: 'http', containerPort: 8080, protocol: 'TCP' }],
      resources: {
        requests: { cpu: '100m', memory: '128Mi' },
        limits: { cpu: '500m', memory: '512Mi' }
      }
    }
  ]

  const formState = parseContainersFromPodSpec(rawContainers)
  assert.equal(formState.length, 1)
  assert.equal(formState[0].name, 'web')
  assert.equal(formState[0].image, 'nginx:alpine')
  assert.equal(formState[0].cpuRequest, '100m')
  assert.equal(formState[0].memoryLimit, '512Mi')
  assert.equal(formState[0].environments.length, 3)

  const serialized = formatContainersToPodSpec(formState)
  assert.equal(serialized.length, 1)
  assert.equal(serialized[0].name, 'web')
  assert.equal(serialized[0].image, 'nginx:alpine')
  assert.deepEqual(serialized[0].command, ['nginx'])
  assert.deepEqual(serialized[0].args, ['-g', 'daemon off;'])
  assert.equal(serialized[0].resources?.requests?.cpu, '100m')
  assert.equal(serialized[0].resources?.limits?.memory, '512Mi')
  assert.equal(serialized[0].ports?.length, 1)
  assert.equal(serialized[0].ports?.[0].containerPort, 8080)
})
