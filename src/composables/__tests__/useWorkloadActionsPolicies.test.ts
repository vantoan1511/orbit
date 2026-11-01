import assert from 'node:assert/strict'
import test from 'node:test'
import { resolveDeleteTargetParams } from '../../utils/workloadActions.ts'

test('resolveDeleteTargetParams resolves specific kind NetworkPolicy when row.kind is NetworkPolicy', () => {
  const row = {
    name: 'allow-dns',
    namespace: 'kube-system',
    kind: 'NetworkPolicy',
    type: 'Network Policy'
  }

  const target = resolveDeleteTargetParams(row, 'Policy')
  assert.equal(target.kind, 'NetworkPolicy')
  assert.equal(target.namespace, 'kube-system')
  assert.equal(target.name, 'allow-dns')
  assert.equal(target.isSuppressed, false)
})

test('resolveDeleteTargetParams suppresses deletion when row.type is Pod Security', () => {
  const psaRow = {
    name: 'psa-default',
    namespace: 'default',
    kind: 'Namespace',
    type: 'Pod Security'
  }

  const target = resolveDeleteTargetParams(psaRow, 'Policy')
  assert.equal(target.isSuppressed, true)
})

test('resolveDeleteTargetParams falls back to default kind when row.kind is undefined', () => {
  const row = {
    name: 'app-quota',
    namespace: 'prod'
  }

  const target = resolveDeleteTargetParams(row, 'Policy')
  assert.equal(target.kind, 'Policy')
  assert.equal(target.isSuppressed, false)
})
