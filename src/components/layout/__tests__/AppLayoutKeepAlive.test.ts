import assert from 'node:assert/strict'
import test from 'node:test'
import type { AppTab } from '../../../types/tabs.ts'
import {
  getViewCacheNames,
  resolveViewKey,
  viewTabCacheName
} from '../../../utils/viewKeepAlive.ts'
import { getEditCacheNames, editTabCacheName } from '../../../utils/editKeepAlive.ts'

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
