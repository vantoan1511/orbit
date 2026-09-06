import test from 'node:test'
import assert from 'node:assert/strict'
import {
  KUBERNETES_DAEMONSET_UPDATE_STRATEGY,
  KUBERNETES_DAEMONSET_UPDATE_STRATEGIES,
  KUBERNETES_DNS_POLICIES,
  KUBERNETES_IMAGE_PULL_POLICIES
} from '../../../constants/kubernetes.ts'
import {
  validateDaemonSetForm,
  isValidIntOrPercent,
  isValidMaxUnavailable,
  isValidMaxSurge,
  normalizeDaemonSetSpec,
  parseContainersFromPodSpec,
  formatContainersToPodSpec
} from '../../../utils/daemonSet.ts'
import type { DaemonSet } from 'kubernetes-types/apps/v1'
import type { Container } from 'kubernetes-types/core/v1'

test('DaemonSet Constants - Update Strategies and Policies', () => {
  assert.equal(KUBERNETES_DAEMONSET_UPDATE_STRATEGY.RollingUpdate, 'RollingUpdate')
  assert.equal(KUBERNETES_DAEMONSET_UPDATE_STRATEGY.OnDelete, 'OnDelete')
  assert.deepEqual(KUBERNETES_DAEMONSET_UPDATE_STRATEGIES, ['RollingUpdate', 'OnDelete'])
  assert.ok(KUBERNETES_DNS_POLICIES.includes('ClusterFirst'))
  assert.ok(KUBERNETES_IMAGE_PULL_POLICIES.includes('IfNotPresent'))
})

test('isValidIntOrPercent - validates correctly', () => {
  assert.equal(isValidIntOrPercent('50%'), true)
  assert.equal(isValidIntOrPercent('5'), true)
  assert.equal(isValidIntOrPercent('150%'), false)
  assert.equal(isValidIntOrPercent('abc'), false)
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

test('isValidMaxSurge - percentage and integer validation', () => {
  assert.equal(isValidMaxSurge('0'), true)
  assert.equal(isValidMaxSurge('1'), true)
  assert.equal(isValidMaxSurge('25%'), true)
  assert.equal(isValidMaxSurge('100%'), true)
  assert.equal(isValidMaxSurge(''), true)
  assert.equal(isValidMaxSurge(null), true)
  assert.equal(isValidMaxSurge(undefined), true)

  assert.equal(isValidMaxSurge('invalid'), false)
  assert.equal(isValidMaxSurge('-1'), false)
  assert.equal(isValidMaxSurge('150%'), false)
  assert.equal(isValidMaxSurge('10 %'), false)
})

test('validateDaemonSetForm - valid values', () => {
  const valid = validateDaemonSetForm({
    serviceAccountName: 'fluentd-sa',
    strategyType: 'RollingUpdate',
    maxUnavailable: '25%',
    maxSurge: '0',
    containers: [
      {
        name: 'fluentd-container',
        workingDir: '/app',
        ports: [{ name: 'http' }]
      }
    ]
  })
  assert.equal(valid, true)
})

test('validateDaemonSetForm - invalid maxUnavailable', () => {
  assert.equal(
    validateDaemonSetForm({
      strategyType: 'RollingUpdate',
      maxUnavailable: 'not-valid'
    }),
    false
  )
})

test('validateDaemonSetForm - invalid maxSurge', () => {
  assert.equal(
    validateDaemonSetForm({
      strategyType: 'RollingUpdate',
      maxSurge: '200%'
    }),
    false
  )
})

test('validateDaemonSetForm - invalid serviceAccountName', () => {
  const invalid = validateDaemonSetForm({
    serviceAccountName: 'INVALID SA',
    containers: []
  })
  assert.equal(invalid, false)
})

test('validateDaemonSetForm - invalid container fields', () => {
  assert.equal(
    validateDaemonSetForm({
      containers: [{ name: 'INVALID_CONTAINER' }]
    }),
    false
  )

  assert.equal(
    validateDaemonSetForm({
      containers: [{ name: 'valid-container', workingDir: 'invalid-rel-path' }]
    }),
    false
  )

  assert.equal(
    validateDaemonSetForm({
      containers: [{ name: '' }]
    }),
    false
  )

  assert.equal(
    validateDaemonSetForm({
      containers: [{ name: '   ' }]
    }),
    false
  )

  assert.equal(
    validateDaemonSetForm({
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

test('normalizeDaemonSetSpec - defaults for empty data', () => {
  const normalized = normalizeDaemonSetSpec(null)
  assert.equal(normalized.strategyType, 'RollingUpdate')
  assert.equal(normalized.maxUnavailable, '1')
  assert.equal(normalized.maxSurge, '0')
  assert.equal(normalized.minReadySeconds, 0)
  assert.equal(normalized.revisionHistoryLimit, 10)
  assert.deepEqual(normalized.selectorLabels, [])
})

test('normalizeDaemonSetSpec - populated data', () => {
  const ds: DaemonSet = {
    metadata: { name: 'fluentd' },
    spec: {
      updateStrategy: {
        type: 'RollingUpdate',
        rollingUpdate: {
          maxUnavailable: '10%',
          maxSurge: 1
        }
      },
      minReadySeconds: 15,
      revisionHistoryLimit: 5,
      selector: {
        matchLabels: { app: 'fluentd', env: 'prod' }
      },
      template: {
        spec: {
          containers: []
        }
      }
    }
  }

  const normalized = normalizeDaemonSetSpec(ds)
  assert.equal(normalized.strategyType, 'RollingUpdate')
  assert.equal(normalized.maxUnavailable, '10%')
  assert.equal(normalized.maxSurge, '1')
  assert.equal(normalized.minReadySeconds, 15)
  assert.equal(normalized.revisionHistoryLimit, 5)
  assert.equal(normalized.selectorLabels.length, 2)
  assert.equal(normalized.selectorLabels.find((s) => s.key === 'app')?.value, 'fluentd')
  assert.equal(normalized.selectorLabels.find((s) => s.key === 'env')?.value, 'prod')
})

test('parseContainersFromPodSpec & formatContainersToPodSpec roundtrip', () => {
  const rawContainers: Container[] = [
    {
      name: 'fluentd',
      image: 'fluent/fluentd:v1.16',
      imagePullPolicy: 'IfNotPresent',
      workingDir: '/home/fluent',
      command: ['fluentd'],
      args: ['-c', '/etc/fluent.conf'],
      env: [
        { name: 'ENV_LITERAL', value: 'production' },
        {
          name: 'CONFIG_KEY',
          valueFrom: {
            configMapKeyRef: { name: 'app-config', key: 'api_url' }
          }
        },
        {
          name: 'SECRET_KEY',
          valueFrom: {
            secretKeyRef: { name: 'app-secret', key: 'api_key' }
          }
        },
        {
          name: 'POD_IP',
          valueFrom: {
            fieldRef: { fieldPath: 'status.podIP' }
          }
        },
        {
          name: 'CPU_LIMIT',
          valueFrom: {
            resourceFieldRef: { resource: 'limits.cpu' }
          }
        }
      ],
      envFrom: [
        { prefix: 'CFG_', configMapRef: { name: 'global-config' } },
        { prefix: 'SEC_', secretRef: { name: 'global-secret' } }
      ],
      ports: [{ name: 'metrics', containerPort: 24231, protocol: 'TCP' }],
      resources: {
        requests: { cpu: '100m', memory: '128Mi' },
        limits: { cpu: '500m', memory: '512Mi' }
      }
    }
  ]

  const parsed = parseContainersFromPodSpec(rawContainers)
  assert.equal(parsed.length, 1)
  assert.equal(parsed[0].name, 'fluentd')
  assert.equal(parsed[0].environments.length, 7)
  assert.equal(parsed[0].ports.length, 1)
  assert.equal(parsed[0].cpuRequest, '100m')
  assert.equal(parsed[0].memoryLimit, '512Mi')

  const formatted = formatContainersToPodSpec(parsed)
  assert.equal(formatted.length, 1)
  assert.equal(formatted[0].name, 'fluentd')
  assert.equal(formatted[0].image, 'fluent/fluentd:v1.16')
  assert.equal(formatted[0].workingDir, '/home/fluent')
  assert.deepEqual(formatted[0].command, ['fluentd'])
  assert.deepEqual(formatted[0].args, ['-c', '/etc/fluent.conf'])
  assert.equal(formatted[0].env?.length, 5)
  assert.equal(formatted[0].envFrom?.length, 2)
  assert.equal(formatted[0].resources?.requests?.cpu, '100m')
  assert.equal(formatted[0].resources?.limits?.memory, '512Mi')
})
