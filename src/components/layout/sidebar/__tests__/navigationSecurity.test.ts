import assert from 'node:assert/strict'
import test from 'node:test'
import { categoryNavItems } from '../navigation.ts'

test('categoryNavItems.security defines exactly 5 policy category tabs', () => {
  const securityItems = categoryNavItems.security
  assert.equal(securityItems.length, 5)

  assert.deepEqual(
    securityItems.map((item) => ({ key: item.key, label: item.label, route: item.route })),
    [
      { key: 'security-overview', label: 'Overview', route: '/policies?tab=overview' },
      { key: 'security-network', label: 'Network Policies', route: '/policies?tab=network' },
      { key: 'security-pod-security', label: 'Pod Security', route: '/policies?tab=pod-security' },
      {
        key: 'security-resource',
        label: 'Resource Quotas & Limits',
        route: '/policies?tab=resource'
      },
      { key: 'security-admission', label: 'Admission Webhooks', route: '/policies?tab=admission' }
    ]
  )
})
