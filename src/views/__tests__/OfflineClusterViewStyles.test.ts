import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('OfflineClusterView uses semantic danger tokens and data-component', () => {
  const filePath = path.resolve('src/views/OfflineClusterView.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('data-component="offline-cluster-view"'),
    'OfflineClusterView must have data-component="offline-cluster-view"'
  )
  assert.ok(
    content.includes('bg-(--danger-soft)'),
    'OfflineClusterView must use bg-(--danger-soft)'
  )
  assert.ok(
    !content.includes('bg-rose-500'),
    'OfflineClusterView must not contain hardcoded bg-rose-500'
  )
})

test('Shell components contain data-component attributes', () => {
  const tabBarPath = path.resolve('src/components/layout/tabs/AppTabBar.vue')
  const tabBarContent = fs.readFileSync(tabBarPath, 'utf-8')
  assert.ok(tabBarContent.includes('data-component="app-tabbar"'))

  const tabItemPath = path.resolve('src/components/layout/tabs/AppTabItem.vue')
  const tabItemContent = fs.readFileSync(tabItemPath, 'utf-8')
  assert.ok(tabItemContent.includes('data-component="app-tab-item"'))

  const footerPath = path.resolve('src/components/layout/AppFooter.vue')
  const footerContent = fs.readFileSync(footerPath, 'utf-8')
  assert.ok(footerContent.includes('data-component="app-footer"'))

  const appPath = path.resolve('src/App.vue')
  const appContent = fs.readFileSync(appPath, 'utf-8')
  assert.ok(appContent.includes('data-component="app-root"'))

  const layoutPath = path.resolve('src/components/layout/AppLayout.vue')
  const layoutContent = fs.readFileSync(layoutPath, 'utf-8')
  assert.ok(layoutContent.includes('data-component="app-layout"'))
})
