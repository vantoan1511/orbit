import assert from 'node:assert/strict'
import test from 'node:test'
import fs from 'node:fs'
import path from 'node:path'
import { getStatusVisualStyle, getNamespaceSeverity } from '../badgeStyles.ts'

test('getStatusVisualStyle returns exact token classes for success, warning, danger, and default statuses and reuses frozen style objects', () => {
  const running = getStatusVisualStyle('Running')
  assert.equal(running.dotClass, 'bg-(--success)')
  assert.equal(running.textClass, 'text-(--success)')
  assert.ok(Object.isFrozen(running))

  // Repeated call returns identical frozen object reference (O(1) table lookup)
  assert.equal(getStatusVisualStyle('Running'), running)

  const pending = getStatusVisualStyle('Pending')
  assert.equal(pending.dotClass, 'bg-(--warning)')
  assert.equal(pending.textClass, 'text-(--warning)')
  assert.ok(Object.isFrozen(pending))
  assert.equal(getStatusVisualStyle('Pending'), pending)

  const failed = getStatusVisualStyle('Failed')
  assert.equal(failed.dotClass, 'bg-(--danger)')
  assert.equal(failed.textClass, 'text-(--danger)')
  assert.ok(Object.isFrozen(failed))

  const unknown = getStatusVisualStyle('UnknownStatus')
  assert.equal(unknown.dotClass, 'bg-(--text-muted)')
  assert.equal(unknown.textClass, 'text-(--text-muted)')
  assert.ok(Object.isFrozen(unknown))

  const undefinedStatus = getStatusVisualStyle(undefined)
  assert.equal(undefinedStatus, unknown)
})

test('getNamespaceSeverity maps kube-system, monitoring, logging, and custom namespaces to expected PrimeVue Tag severities', () => {
  assert.equal(getNamespaceSeverity('kube-system'), 'secondary')
  assert.equal(getNamespaceSeverity('monitoring'), 'warn')
  assert.equal(getNamespaceSeverity('logging'), 'info')
  assert.equal(getNamespaceSeverity('default'), 'secondary')
  assert.equal(getNamespaceSeverity('custom-app'), 'secondary')
  assert.equal(getNamespaceSeverity(undefined), 'secondary')
})

test('StatusBadge and NamespaceBadge delegate to badgeStyles without multi-computed chains', () => {
  const statusBadgePath = path.resolve('src/components/shared/StatusBadge.vue')
  const statusBadgeSrc = fs.readFileSync(statusBadgePath, 'utf8')
  assert.match(statusBadgeSrc, /getStatusVisualStyle/)
  assert.doesNotMatch(statusBadgeSrc, /const statusColor = computed/)
  assert.doesNotMatch(statusBadgeSrc, /const dotClass = computed/)
  assert.doesNotMatch(statusBadgeSrc, /const textClass = computed/)

  const namespaceBadgePath = path.resolve('src/components/shared/NamespaceBadge.vue')
  const namespaceBadgeSrc = fs.readFileSync(namespaceBadgePath, 'utf8')
  assert.match(namespaceBadgeSrc, /getNamespaceSeverity/)
  assert.doesNotMatch(namespaceBadgeSrc, /switch \(props\.namespace\)/)
})
