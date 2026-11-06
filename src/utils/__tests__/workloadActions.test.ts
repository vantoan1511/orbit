import test from 'node:test'
import assert from 'node:assert/strict'
import {
  resolveEditTargetParams,
  resolveDeleteTargetParams,
  resolveNodeActionConfig
} from '../workloadActions.ts'

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

test('resolveEditTargetParams resolves PSA row to underlying Namespace manifest name and default namespace', () => {
  const row = {
    name: 'psa-production',
    namespace: 'production',
    kind: 'Namespace',
    type: 'Pod Security'
  }
  const params = resolveEditTargetParams(row, 'Policy')
  assert.deepEqual(params, {
    kind: 'Namespace',
    namespace: 'default',
    name: 'production'
  })
})

test('resolveDeleteTargetParams suppresses delete action for Node kind', () => {
  const row = {
    name: 'node-1',
    kind: 'Node'
  }
  const params = resolveDeleteTargetParams(row, 'Node')
  assert.equal(params.isSuppressed, true)
  assert.equal(params.kind, 'Node')
  assert.equal(params.name, 'node-1')
})

test('resolveNodeActionConfig returns correct label and confirmation config for cordoned vs uncordoned nodes', () => {
  const uncordoned = resolveNodeActionConfig(false)
  assert.equal(uncordoned.cordonLabel, 'Cordon')
  assert.equal(uncordoned.cordonIcon, 'pi pi-ban')
  assert.equal(uncordoned.needsConfirmation, true)

  const cordoned = resolveNodeActionConfig(true)
  assert.equal(cordoned.cordonLabel, 'Uncordon')
  assert.equal(cordoned.cordonIcon, 'pi pi-check-circle')
  assert.equal(cordoned.needsConfirmation, false)
})
