import assert from 'node:assert/strict'
import test from 'node:test'
import {
  applyMetadataFormValuesToResource,
  buildEditRequestId,
  extractResourceMetadataFormValues,
  matchesEditResponse,
  matchesSaveResult,
  shouldHandleCommandResult,
  transitionEditMode
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

test('applyMetadataFormValuesToResource updates labels and annotations immutably while preserving spec and other metadata fields', () => {
  const cronJob = {
    apiVersion: 'batch/v1',
    kind: 'CronJob',
    metadata: {
      name: 'nightly-backup',
      namespace: 'ops',
      uid: 'uid-1234',
      creationTimestamp: '2026-01-01T00:00:00Z',
      labels: { env: 'production', old: 'to-remove' }
    },
    spec: {
      schedule: '0 0 * * *',
      jobTemplate: {
        spec: {
          template: {
            spec: {
              containers: [{ name: 'backup', image: 'backup:v1' }]
            }
          }
        }
      }
    }
  }

  const formValues = {
    name: 'nightly-backup',
    namespace: 'ops',
    labels: [
      { key: 'env', value: 'staging' },
      { key: 'tier', value: 'backend' },
      { key: '   ', value: 'ignored-empty-key' }
    ],
    annotations: [
      { key: 'orbit.dev/managed-by', value: 'admin' },
      { key: '', value: 'ignored' }
    ]
  }

  const updated = applyMetadataFormValuesToResource(cronJob, formValues)

  // Verify immutability
  assert.notEqual(updated, cronJob)
  assert.notEqual(updated.metadata, cronJob.metadata)

  // Verify spec and original metadata preservation
  assert.deepEqual(updated.spec, cronJob.spec)
  assert.equal(updated.metadata.name, 'nightly-backup')
  assert.equal(updated.metadata.namespace, 'ops')
  assert.equal(updated.metadata.uid, 'uid-1234')
  assert.equal(updated.metadata.creationTimestamp, '2026-01-01T00:00:00Z')

  // Verify updated labels & annotations (empty/whitespace keys stripped)
  assert.deepEqual(updated.metadata.labels, {
    env: 'staging',
    tier: 'backend'
  })
  assert.deepEqual(updated.metadata.annotations, {
    'orbit.dev/managed-by': 'admin'
  })
})

test('transitionEditMode Form -> YAML serializes latest rawData including Deployment and fallback edits', () => {
  const rawData = {
    apiVersion: 'apps/v1',
    kind: 'Deployment',
    metadata: {
      name: 'web',
      namespace: 'default',
      labels: { app: 'web' }
    },
    spec: {
      replicas: 3
    }
  }

  const result = transitionEditMode({
    targetYamlMode: true,
    rawData,
    yamlContent: '',
    hasCustomForm: true
  })

  assert.equal(result.ok, true)
  if (result.ok) {
    assert.equal(result.isYamlMode, true)
    assert.deepEqual(result.rawData, rawData)
    assert.match(result.yamlContent, /replicas: 3/)
    assert.match(result.yamlContent, /name: web/)
  }
})

test('transitionEditMode YAML -> Form parses valid YAML, updates rawData, and refreshes formValues for fallback resources', () => {
  const oldData = {
    apiVersion: 'v1',
    kind: 'ConfigMap',
    metadata: { name: 'settings', namespace: 'default' }
  }

  const editedYaml = `apiVersion: v1
kind: ConfigMap
metadata:
  name: settings
  namespace: default
  labels:
    app.kubernetes.io/name: orbit
  annotations:
    description: main config
data:
  theme: noir
`

  const result = transitionEditMode({
    targetYamlMode: false,
    rawData: oldData,
    yamlContent: editedYaml,
    hasCustomForm: false
  })

  assert.equal(result.ok, true)
  if (result.ok) {
    assert.equal(result.isYamlMode, false)
    assert.equal(result.rawData?.kind, 'ConfigMap')
    assert.deepEqual(result.rawData?.data, { theme: 'noir' })
    assert.ok(
      result.formValues,
      'formValues must be populated for fallback resources without custom form'
    )
    assert.equal(result.formValues?.name, 'settings')
    assert.deepEqual(result.formValues?.labels, [{ key: 'app.kubernetes.io/name', value: 'orbit' }])
    assert.deepEqual(result.formValues?.annotations, [{ key: 'description', value: 'main config' }])
  }
})

test('transitionEditMode YAML -> Form rejects invalid YAML syntax or scalar/array YAML and stays in YAML mode', () => {
  const malformedYaml = `apiVersion: v1
  bad: [unclosed`

  const resultInvalid = transitionEditMode({
    targetYamlMode: false,
    rawData: null,
    yamlContent: malformedYaml,
    hasCustomForm: false
  })

  assert.equal(resultInvalid.ok, false)
  assert.equal(resultInvalid.isYamlMode, true)
  if (!resultInvalid.ok) {
    assert.ok(resultInvalid.errorMessage.length > 0)
  }

  const scalarYaml = 'just-a-string'
  const resultScalar = transitionEditMode({
    targetYamlMode: false,
    rawData: null,
    yamlContent: scalarYaml,
    hasCustomForm: false
  })

  assert.equal(resultScalar.ok, false)
  assert.equal(resultScalar.isYamlMode, true)
  if (!resultScalar.ok) {
    assert.match(resultScalar.errorMessage, /object/i)
  }

  const arrayYaml = '- item1\n- item2'
  const resultArray = transitionEditMode({
    targetYamlMode: false,
    rawData: null,
    yamlContent: arrayYaml,
    hasCustomForm: false
  })

  assert.equal(resultArray.ok, false)
  assert.equal(resultArray.isYamlMode, true)
  if (!resultArray.ok) {
    assert.match(resultArray.errorMessage, /object/i)
  }
})

test('applyMetadataFormValuesToResource omits metadata.namespace for cluster-scoped resources when empty', () => {
  const result = applyMetadataFormValuesToResource(
    { apiVersion: 'v1', kind: 'PersistentVolume', metadata: { name: 'pv-1' } },
    { name: 'pv-1', namespace: '', labels: [], annotations: [] }
  )
  assert.equal(result.metadata?.namespace, undefined)
})
