import assert from 'node:assert/strict'
import test from 'node:test'
import {
  buildEditRequestId,
  extractResourceMetadataFormValues,
  matchesEditResponse,
  matchesSaveResult,
  shouldHandleCommandResult
} from '../editSession.ts'

test('matchesEditResponse matches by requestId equality even if namespace missing', () => {
  const payload = {
    kind: 'Deployment',
    name: 'app',
    requestId: 'req-1'
  }
  const ctx = {
    kind: 'Deployment',
    namespace: 'prod',
    name: 'app',
    requestId: 'req-1'
  }
  assert.equal(matchesEditResponse(payload, ctx), true)
})

test('matchesEditResponse rejects same kind+name with different namespace', () => {
  const payload = {
    kind: 'Deployment',
    name: 'app',
    namespace: 'dev'
  }
  const ctx = {
    kind: 'Deployment',
    namespace: 'prod',
    name: 'app'
  }
  assert.equal(matchesEditResponse(payload, ctx), false)
})

test('matchesEditResponse rejects different requestId even when kind/name/namespace equal', () => {
  const payload = {
    kind: 'Deployment',
    namespace: 'prod',
    name: 'app',
    requestId: 'req-2'
  }
  const ctx = {
    kind: 'Deployment',
    namespace: 'prod',
    name: 'app',
    requestId: 'req-1'
  }
  assert.equal(matchesEditResponse(payload, ctx), false)
})

test('matchesEditResponse matches legacy payload (no requestId, no namespace) on kind+name only', () => {
  const payload = {
    kind: 'Deployment',
    name: 'app'
  }
  const ctx = {
    kind: 'Deployment',
    namespace: 'prod',
    name: 'app'
  }
  assert.equal(matchesEditResponse(payload, ctx), true)

  const differentKindPayload = {
    kind: 'StatefulSet',
    name: 'app'
  }
  assert.equal(matchesEditResponse(differentKindPayload, ctx), false)

  const differentNamePayload = {
    kind: 'Deployment',
    name: 'other-app'
  }
  assert.equal(matchesEditResponse(differentNamePayload, ctx), false)
})

test('buildEditRequestId is deterministic for same inputs + nonce and differs across namespaces', () => {
  const id1 = buildEditRequestId('Deployment', 'ns-a', 'web', 'nonce-1')
  const id2 = buildEditRequestId('Deployment', 'ns-a', 'web', 'nonce-1')
  const id3 = buildEditRequestId('Deployment', 'ns-b', 'web', 'nonce-1')

  assert.equal(id1, id2)
  assert.notEqual(id1, id3)
})

test('shouldHandleCommandResult returns true only when saving', () => {
  assert.equal(shouldHandleCommandResult(true), true)
  assert.equal(shouldHandleCommandResult(false), false)
})

test('matchesSaveResult returns false when isSaving is false even if requestId matches', () => {
  const payload = {
    kind: 'Deployment',
    name: 'api',
    namespace: 'prod',
    requestId: 'req-save-1',
    success: true
  }
  const ctx = {
    isSaving: false,
    kind: 'Deployment',
    namespace: 'prod',
    name: 'api',
    requestId: 'req-save-1'
  }
  assert.equal(matchesSaveResult(payload, ctx), false)
})

test('matchesSaveResult returns true when isSaving is true and requestId matches', () => {
  const payload = {
    kind: 'Deployment',
    name: 'api',
    namespace: 'prod',
    requestId: 'req-save-1',
    success: true
  }
  const ctx = {
    isSaving: true,
    kind: 'Deployment',
    namespace: 'prod',
    name: 'api',
    requestId: 'req-save-1'
  }
  assert.equal(matchesSaveResult(payload, ctx), true)
})

test('matchesSaveResult returns false when payload has different requestId or omits requestId when ctx has requestId', () => {
  const ctx = {
    isSaving: true,
    kind: 'Deployment',
    namespace: 'prod',
    name: 'api',
    requestId: 'req-save-1'
  }

  const payloadMismatchedReq = {
    kind: 'Deployment',
    name: 'api',
    namespace: 'prod',
    requestId: 'req-save-2',
    success: true
  }
  assert.equal(matchesSaveResult(payloadMismatchedReq, ctx), false)

  const payloadMissingReq = {
    kind: 'Deployment',
    name: 'api',
    namespace: 'prod',
    success: true
  }
  assert.equal(matchesSaveResult(payloadMissingReq, ctx), false)
})

test('extractResourceMetadataFormValues extracts name, namespace, labels, annotations and handles missing metadata safely', () => {
  const resource = {
    apiVersion: 'apps/v1',
    kind: 'Deployment',
    metadata: {
      name: 'web',
      namespace: 'production',
      labels: {
        app: 'web',
        tier: 'frontend'
      },
      annotations: {
        'deployment.kubernetes.io/revision': '2'
      }
    }
  }

  const values = extractResourceMetadataFormValues(resource)
  assert.equal(values.name, 'web')
  assert.equal(values.namespace, 'production')
  assert.deepEqual(values.labels, [
    { key: 'app', value: 'web' },
    { key: 'tier', value: 'frontend' }
  ])
  assert.deepEqual(values.annotations, [{ key: 'deployment.kubernetes.io/revision', value: '2' }])

  const emptyValues = extractResourceMetadataFormValues(null)
  assert.equal(emptyValues.name, '')
  assert.equal(emptyValues.namespace, '')
  assert.deepEqual(emptyValues.labels, [])
  assert.deepEqual(emptyValues.annotations, [])

  const partialValues = extractResourceMetadataFormValues({ metadata: { name: 'bare' } })
  assert.equal(partialValues.name, 'bare')
  assert.equal(partialValues.namespace, '')
  assert.deepEqual(partialValues.labels, [])
  assert.deepEqual(partialValues.annotations, [])
})
