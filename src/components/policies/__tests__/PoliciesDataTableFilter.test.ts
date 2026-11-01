import assert from 'node:assert/strict'
import test from 'node:test'
import { filterPolicies, getFilterOptionsForCategory } from '../policyFilter.ts'
import type { PolicyInfo } from '../../../types/kubernetes.ts'

const samplePolicies: PolicyInfo[] = [
  {
    uid: '1',
    name: 'netpol-ingress',
    type: 'Network Policy',
    scope: 'Namespaced',
    namespace: 'default',
    status: 'Enforced',
    mode: 'Ingress',
    violations: 0,
    lastUpdated: '1d',
    description: 'Ingress rule',
    rules: '',
    kind: 'NetworkPolicy'
  },
  {
    uid: '2',
    name: 'psa-prod',
    type: 'Pod Security',
    scope: 'Namespaced',
    namespace: 'prod',
    status: 'Enforced',
    mode: 'enforce: restricted',
    violations: 0,
    lastUpdated: '2d',
    description: 'PSA: enforce=restricted',
    rules: '',
    kind: 'Namespace'
  },
  {
    uid: '3',
    name: 'cpu-quota',
    type: 'Resource Quota',
    scope: 'Namespaced',
    namespace: 'dev',
    status: 'Enforced',
    mode: 'enforce',
    violations: 1,
    lastUpdated: '3d',
    description: 'Resource limits',
    rules: '',
    kind: 'ResourceQuota'
  },
  {
    uid: '4',
    name: 'mem-limits',
    type: 'Limit Range',
    scope: 'Namespaced',
    namespace: 'dev',
    status: 'Enforced',
    mode: 'enforce',
    violations: 0,
    lastUpdated: '4d',
    description: 'Limits for: Container',
    rules: '',
    kind: 'LimitRange'
  },
  {
    uid: '5',
    name: 'val-webhook',
    type: 'Admission Policy',
    scope: 'Cluster',
    namespace: '-',
    status: 'Enforced',
    mode: 'Fail',
    violations: 0,
    lastUpdated: '5d',
    description: '1 webhook(s)',
    rules: '',
    kind: 'ValidatingWebhookConfiguration'
  },
  {
    uid: '6',
    name: 'mut-webhook',
    type: 'Admission Policy',
    scope: 'Cluster',
    namespace: '-',
    status: 'Audit',
    mode: 'Ignore',
    violations: 0,
    lastUpdated: '6d',
    description: '1 webhook(s)',
    rules: '',
    kind: 'MutatingWebhookConfiguration'
  }
]

test('getFilterOptionsForCategory returns correct options', () => {
  assert.deepEqual(getFilterOptionsForCategory('overview'), [
    'All Types',
    'Network Policy',
    'Pod Security',
    'Resource Quota',
    'Limit Range',
    'Admission Policy'
  ])
  assert.deepEqual(getFilterOptionsForCategory('resource'), [
    'All Resource Policies',
    'Resource Quotas',
    'Limit Ranges'
  ])
  assert.deepEqual(getFilterOptionsForCategory('admission'), [
    'All Webhooks',
    'Validating Webhooks',
    'Mutating Webhooks'
  ])
  assert.deepEqual(getFilterOptionsForCategory('network'), [])
  assert.deepEqual(getFilterOptionsForCategory('pod-security'), [])
})

test('filterPolicies handles overview category and sub-type selection', () => {
  const all = filterPolicies(samplePolicies, 'overview', 'All Types')
  assert.equal(all.length, 6)

  const netOnly = filterPolicies(samplePolicies, 'overview', 'Network Policy')
  assert.equal(netOnly.length, 1)
  assert.equal(netOnly[0].name, 'netpol-ingress')
})

test('filterPolicies handles network category', () => {
  const res = filterPolicies(samplePolicies, 'network', '')
  assert.equal(res.length, 1)
  assert.equal(res[0].type, 'Network Policy')
})

test('filterPolicies handles pod-security category', () => {
  const res = filterPolicies(samplePolicies, 'pod-security', '')
  assert.equal(res.length, 1)
  assert.equal(res[0].type, 'Pod Security')
})

test('filterPolicies handles resource category and sub-types', () => {
  const allRes = filterPolicies(samplePolicies, 'resource', 'All Resource Policies')
  assert.equal(allRes.length, 2)

  const quotas = filterPolicies(samplePolicies, 'resource', 'Resource Quotas')
  assert.equal(quotas.length, 1)
  assert.equal(quotas[0].name, 'cpu-quota')

  const limits = filterPolicies(samplePolicies, 'resource', 'Limit Ranges')
  assert.equal(limits.length, 1)
  assert.equal(limits[0].name, 'mem-limits')
})

test('filterPolicies handles admission category and sub-types', () => {
  const allAdm = filterPolicies(samplePolicies, 'admission', 'All Webhooks')
  assert.equal(allAdm.length, 2)

  const val = filterPolicies(samplePolicies, 'admission', 'Validating Webhooks')
  assert.equal(val.length, 1)
  assert.equal(val[0].name, 'val-webhook')

  const mut = filterPolicies(samplePolicies, 'admission', 'Mutating Webhooks')
  assert.equal(mut.length, 1)
  assert.equal(mut[0].name, 'mut-webhook')
})
