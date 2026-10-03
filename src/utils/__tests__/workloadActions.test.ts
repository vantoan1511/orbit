import test from 'node:test'
import assert from 'node:assert/strict'
import { resolveEditTargetParams } from '../workloadActions.ts'

test('resolveEditTargetParams uses row.kind when available on row object', () => {
  const row = {
    name: 'allow-dns',
    namespace: 'default',
    kind: 'NetworkPolicy'
  }
  const params = resolveEditTargetParams(row, 'Policy')
  assert.deepEqual(params, {
    kind: 'NetworkPolicy',
    namespace: 'default',
    name: 'allow-dns'
  })
})

test('resolveEditTargetParams falls back to defaultKind when row has no kind', () => {
  const row = {
    name: 'my-deployment',
    namespace: 'prod'
  }
  const params = resolveEditTargetParams(row, 'Deployment')
  assert.deepEqual(params, {
    kind: 'Deployment',
    namespace: 'prod',
    name: 'my-deployment'
  })
})

test('resolveEditTargetParams maps cluster-scoped namespace "-" to "default"', () => {
  const row = {
    name: 'val-webhook',
    namespace: '-',
    kind: 'ValidatingWebhookConfiguration'
  }
  const params = resolveEditTargetParams(row, 'Policy')
  assert.deepEqual(params, {
    kind: 'ValidatingWebhookConfiguration',
    namespace: 'default',
    name: 'val-webhook'
  })
})

test('resolveEditTargetParams falls back to "default" if namespace is missing', () => {
  const row = {
    name: 'worker-node-1',
    kind: 'Node'
  }
  const params = resolveEditTargetParams(row, 'Node')
  assert.deepEqual(params, {
    kind: 'Node',
    namespace: 'default',
    name: 'worker-node-1'
  })
})
