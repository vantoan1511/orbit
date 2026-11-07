import { describe, it } from 'node:test'
import assert from 'node:assert'
import {
  validateHpaBounds,
  resolveHpaTargetRoute,
  getHpaStatusBadgeClass,
  getHpaStatusSeverity,
  formatHpaConditions
} from '../hpa.ts'
import type { HpaConditionInfo } from '../../types/kubernetes.ts'

describe('HPA Utilities', () => {
  describe('validateHpaBounds', () => {
    it('validates minReplicas >= 1 and maxReplicas >= minReplicas', () => {
      assert.deepStrictEqual(validateHpaBounds(1, 10), { valid: true })
      assert.deepStrictEqual(validateHpaBounds(5, 5), { valid: true })

      assert.strictEqual(validateHpaBounds(0, 10).valid, false)
      assert.strictEqual(validateHpaBounds(-1, 10).valid, false)
      assert.strictEqual(validateHpaBounds(5, 2).valid, false)
      assert.strictEqual(validateHpaBounds(1.5, 5).valid, false)
    })
  })

  describe('resolveHpaTargetRoute', () => {
    it('maps Deployment, StatefulSet, ReplicaSet to workloads tabs', () => {
      assert.deepStrictEqual(resolveHpaTargetRoute('Deployment', 'web', 'prod'), {
        path: '/workloads',
        query: { tab: 'deployments', highlight: 'web', namespace: 'prod' }
      })

      assert.deepStrictEqual(resolveHpaTargetRoute('StatefulSet', 'db', 'data'), {
        path: '/workloads',
        query: { tab: 'statefulsets', highlight: 'db', namespace: 'data' }
      })

      assert.deepStrictEqual(resolveHpaTargetRoute('ReplicaSet', 'rs-1', 'default'), {
        path: '/workloads',
        query: { tab: 'replicasets', highlight: 'rs-1', namespace: 'default' }
      })

      assert.strictEqual(resolveHpaTargetRoute('UnknownKind', 'test', 'default'), null)
    })
  })

  describe('getHpaStatusBadgeClass and getHpaStatusSeverity', () => {
    it('returns appropriate semantic colors and severities for HPA statuses', () => {
      assert.strictEqual(getHpaStatusBadgeClass('Active'), 'bg-(--success)')
      assert.strictEqual(getHpaStatusSeverity('Active'), 'success')

      assert.strictEqual(getHpaStatusBadgeClass('ScalingLimited'), 'bg-(--warning)')
      assert.strictEqual(getHpaStatusSeverity('ScalingLimited'), 'warn')

      assert.strictEqual(getHpaStatusBadgeClass('Pending'), 'bg-(--warning)')
      assert.strictEqual(getHpaStatusSeverity('Pending'), 'warn')

      assert.strictEqual(getHpaStatusBadgeClass('Failed'), 'bg-(--danger)')
      assert.strictEqual(getHpaStatusSeverity('Failed'), 'danger')

      assert.strictEqual(getHpaStatusBadgeClass('Unknown'), 'bg-muted-color')
      assert.strictEqual(getHpaStatusSeverity('Unknown'), 'secondary')
    })
  })

  describe('formatHpaConditions', () => {
    it('formats condition list into UI tags with status severity', () => {
      const conditions: HpaConditionInfo[] = [
        { type: 'AbleToScale', status: 'True' },
        { type: 'ScalingActive', status: 'True' },
        { type: 'ScalingLimited', status: 'False' }
      ]
      const formatted = formatHpaConditions(conditions)
      assert.strictEqual(formatted.length, 3)
      assert.strictEqual(formatted[0].severity, 'success')
      assert.strictEqual(formatted[1].severity, 'success')
      assert.strictEqual(formatted[2].severity, 'secondary')
    })
  })
})
