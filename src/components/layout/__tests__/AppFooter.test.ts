import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('AppFooter component does not render cluster name or metadata in footer', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/layout/AppFooter.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    !content.includes('KubernetesIcon'),
    'AppFooter must not import or render KubernetesIcon'
  )
  assert.ok(
    !content.includes('clusterUptime'),
    'AppFooter must not compute or display clusterUptime'
  )
  assert.ok(
    !content.includes('detectCloudProvider'),
    'AppFooter must not import detectCloudProvider'
  )
  assert.ok(
    !content.includes('cloudProvider'),
    'AppFooter must not compute or display cloudProvider'
  )
})

test('AppFooter component preserves placeholder for breadcrumb on the left', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/layout/AppFooter.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('data-testid="footer-breadcrumbs-placeholder"'),
    'AppFooter must define a left-side placeholder container for future breadcrumb placement'
  )
})

test('AppFooter component preserves engine status, refresh, and terminal controls', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/layout/AppFooter.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('isReconnecting'),
    'AppFooter must retain engine reconnecting indicator'
  )
  assert.ok(
    content.includes('isFailed'),
    'AppFooter must retain engine failed state with restart action'
  )
  assert.ok(
    content.includes('lastUpdatedDisplay'),
    'AppFooter must retain updated timestamp display'
  )
  assert.ok(content.includes('refreshCluster'), 'AppFooter must retain refresh cluster button')
  assert.ok(
    content.includes('terminalStore.togglePanel()'),
    'AppFooter must retain terminal panel toggle button'
  )
})
