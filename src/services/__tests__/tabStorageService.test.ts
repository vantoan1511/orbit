import assert from 'node:assert/strict'
import test from 'node:test'
import { storage } from '../nativeService.ts'
import {
  createDefaultTabsState,
  getTabsStorageKey,
  tabStorageService,
  validateTabsState
} from '../tabStorageService.ts'
import type { TabsStorageState } from '@/types/tabs.ts'

test('createDefaultTabsState returns valid initial state', () => {
  const defaults = createDefaultTabsState()
  assert.deepEqual(defaults, {
    tabs: [],
    activeTabId: null
  })
})

test('getTabsStorageKey isolates by clusterId when provided', () => {
  assert.equal(getTabsStorageKey(), 'orbit_app_tabs')
  assert.equal(getTabsStorageKey('prod-cluster'), 'orbit_app_tabs_prod-cluster')
})

test('validateTabsState sanitizes invalid inputs safely and cleans legacy minimize fields', () => {
  assert.deepEqual(validateTabsState(null), createDefaultTabsState())
  assert.deepEqual(validateTabsState(undefined), createDefaultTabsState())
  assert.deepEqual(validateTabsState('invalid'), createDefaultTabsState())
  assert.deepEqual(validateTabsState([]), createDefaultTabsState())
  assert.deepEqual(validateTabsState({}), createDefaultTabsState())

  // Filters out invalid tab items and strips legacy placement/isMinimized
  const partialWithLegacy = {
    tabs: [
      { id: '1', title: 'Pods', route: '/workloads?tab=pods', path: '/workloads' },
      { notATab: true }
    ],
    activeTabId: '1',
    placement: 'bottom',
    isMinimized: true
  }
  const validated = validateTabsState(partialWithLegacy)
  assert.equal(validated.tabs.length, 1)
  assert.equal(validated.tabs[0]?.title, 'Pods')
  assert.equal(validated.activeTabId, '1')
  // Verify obsolete properties are not on the returned state
  assert.equal('placement' in validated, false)
  assert.equal('isMinimized' in validated, false)
})

test('tabStorageService saves and loads tabs from native storage', async () => {
  const storeMap = new Map<string, string>()
  const originalGetData = storage.getData
  const originalSetData = storage.setData
  const originalRemoveData = storage.removeData

  storage.getData = async (key: string) => {
    return storeMap.get(key) ?? null
  }
  storage.setData = async (key: string, data?: string | null) => {
    if (data) storeMap.set(key, data)
    else storeMap.delete(key)
  }
  storage.removeData = async (key: string) => {
    storeMap.delete(key)
  }

  const sampleState: TabsStorageState = {
    tabs: [
      {
        id: '/workloads?tab=deployments',
        title: 'Deployments',
        route: '/workloads?tab=deployments',
        path: '/workloads',
        iconName: 'Layers',
        closable: true
      }
    ],
    activeTabId: '/workloads?tab=deployments'
  }

  try {
    await tabStorageService.saveTabsState(sampleState, 'test-cluster')

    const loaded = await tabStorageService.loadTabsState('test-cluster')
    assert.deepEqual(loaded, sampleState)

    await tabStorageService.clearTabsState('test-cluster')
    const afterClear = await tabStorageService.loadTabsState('test-cluster')
    assert.equal(afterClear, null)
  } finally {
    storage.getData = originalGetData
    storage.setData = originalSetData
    storage.removeData = originalRemoveData
  }
})
