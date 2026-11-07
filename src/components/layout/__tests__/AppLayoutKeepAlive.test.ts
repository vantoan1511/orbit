import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'
import type { AppTab } from '../../../types/tabs.ts'
import {
  getViewCacheNames,
  resolveViewKey,
  viewTabCacheName
} from '../../../utils/viewKeepAlive.ts'
import { getEditCacheNames, editTabCacheName } from '../../../utils/editKeepAlive.ts'

test('AppLayout clears defaultViewHostRegistry and defaultEditHostRegistry unconditionally on cluster change', () => {
  const layoutPath = path.resolve('src/components/layout/AppLayout.vue')
  const content = fs.readFileSync(layoutPath, 'utf-8')

  const watchBlockMatch = content.match(
    /watch\(\s*\(\)\s*=>\s*k8sStore\.activeClusterId[\s\S]*?\n\)/
  )
  assert.ok(watchBlockMatch, 'AppLayout must watch k8sStore.activeClusterId')
  const watchBlock = watchBlockMatch[0]

  const viewClearIndex = watchBlock.indexOf('defaultViewHostRegistry.clearAll()')
  const editClearIndex = watchBlock.indexOf('defaultEditHostRegistry.clearAll()')
  const ifConditionIndex = watchBlock.indexOf('if (newClusterId)')

  assert.ok(viewClearIndex !== -1, 'Must call defaultViewHostRegistry.clearAll()')
  assert.ok(editClearIndex !== -1, 'Must call defaultEditHostRegistry.clearAll()')
  assert.ok(ifConditionIndex !== -1, 'Must check if (newClusterId)')

  assert.ok(
    viewClearIndex < ifConditionIndex,
    'defaultViewHostRegistry.clearAll() must run before if (newClusterId)'
  )
  assert.ok(
    editClearIndex < ifConditionIndex,
    'defaultEditHostRegistry.clearAll() must run before if (newClusterId)'
  )
})

test('resolveViewKey returns fullPath for edit routes and path otherwise', () => {
  const editPath = '/workloads/Deployment/default/nginx/edit'
  const editFullPath = '/workloads/Deployment/default/nginx/edit?from=list'
  assert.equal(resolveViewKey(editPath, editFullPath), editFullPath)

  const normalPath = '/workloads'
  const normalFullPath = '/workloads?tab=deployments'
  assert.equal(resolveViewKey(normalPath, normalFullPath), normalPath)
})

test('KeepAlive include list aggregates edit and view cache names', () => {
  const tabs: AppTab[] = [
    {
      id: '/workloads/Deployment/default/nginx/edit',
      title: 'Edit nginx',
      route: '/workloads/Deployment/default/nginx/edit',
      path: '/workloads/Deployment/default/nginx/edit'
    },
    {
      id: '/workloads',
      title: 'Workloads',
      route: '/workloads',
      path: '/workloads'
    }
  ]

  const editNames = getEditCacheNames(tabs)
  const viewNames = getViewCacheNames(tabs)
  const combined = [...editNames, ...viewNames]

  assert.equal(combined.length, 2)
  assert.ok(combined.includes(editTabCacheName('/workloads/Deployment/default/nginx/edit')))
  assert.ok(combined.includes(viewTabCacheName('/workloads')))
})

test('AppLayout renders EmptyWorkspaceView whenever activeClusterId is set and tabs are empty, and re-syncs on router.afterEach', () => {
  const layoutPath = path.resolve('src/components/layout/AppLayout.vue')
  const content = fs.readFileSync(layoutPath, 'utf-8')

  assert.ok(
    content.includes('k8sStore.activeClusterId !== null && tabsStore.tabs.length === 0'),
    'AppLayout must render EmptyWorkspaceView when activeClusterId is set and tabsStore.tabs.length === 0'
  )
  assert.ok(content.includes('router.afterEach('), 'AppLayout must register router.afterEach hook')

  const navMenuPath = path.resolve('src/components/layout/sidebar/AppSidebarNavMenu.vue')
  const navMenuContent = fs.readFileSync(navMenuPath, 'utf-8')
  assert.ok(
    navMenuContent.includes('tabsStore.tabs.length === 0'),
    'AppSidebarNavMenu isLinkActive must check tabsStore.tabs.length === 0'
  )

  const logsMenuPath = path.resolve('src/components/layout/sidebar/AppSidebarLogsMenu.vue')
  const logsMenuContent = fs.readFileSync(logsMenuPath, 'utf-8')
  assert.ok(
    logsMenuContent.includes('tabsStore.tabs.length === 0'),
    'AppSidebarLogsMenu isCurrentLogActive must check tabsStore.tabs.length === 0'
  )

  const activityBarPath = path.resolve('src/components/layout/sidebar/AppSidebarActivityBar.vue')
  const activityBarContent = fs.readFileSync(activityBarPath, 'utf-8')
  assert.ok(
    activityBarContent.includes('tabsStore.tabs.length === 0'),
    'AppSidebarActivityBar Settings active state must check tabsStore.tabs.length === 0 when hasActiveCluster is true'
  )
})
