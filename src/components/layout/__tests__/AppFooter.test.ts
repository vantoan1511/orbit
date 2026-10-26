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

test('AppFooter component mounts AppBreadcrumbs in the left placeholder slot', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/layout/AppFooter.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes("import AppBreadcrumbs from './AppBreadcrumbs.vue'"),
    'AppFooter must import AppBreadcrumbs'
  )
  assert.ok(
    content.includes('<AppBreadcrumbs />') || content.includes('<AppBreadcrumbs/>'),
    'AppFooter must mount AppBreadcrumbs component'
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

test('AppFooter component renders watcher connection indicators and reconnect action', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/layout/AppFooter.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('watcherConnectionState'),
    'AppFooter must check watcherConnectionState'
  )
  assert.ok(content.includes('degraded'), 'AppFooter must handle degraded state')
  assert.ok(content.includes('suspended'), 'AppFooter must handle suspended state')
  assert.ok(content.includes('Reconnect'), 'AppFooter must render Reconnect action button')
  assert.ok(content.includes('v-tooltip'), 'AppFooter must use v-tooltip for hints')
  assert.ok(!content.includes('title='), 'AppFooter must never use raw title= attributes')
})

test('AppFooter renders Restart app and Reconnect actions through FooterStatusAction (AC-11)', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/layout/AppFooter.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes("import FooterStatusAction from './FooterStatusAction.vue'"),
    'AppFooter must import FooterStatusAction'
  )
  assert.ok(
    content.includes('label="Restart app"'),
    'AppFooter must use FooterStatusAction for Restart app'
  )
  assert.ok(
    content.includes('@click="restartApp"'),
    'AppFooter must bind restartApp to Restart app action'
  )
  assert.ok(
    content.includes('label="Reconnect"'),
    'AppFooter must use FooterStatusAction for Reconnect'
  )
  assert.ok(
    content.includes('@click="handleReconnectCluster"'),
    'AppFooter must bind handleReconnectCluster to Reconnect action'
  )
  assert.ok(
    content.includes(':loading="isReconnectingCluster"'),
    'AppFooter must bind isReconnectingCluster to Reconnect loading state'
  )
})
