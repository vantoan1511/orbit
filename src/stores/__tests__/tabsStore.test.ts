import assert from 'node:assert/strict'
import test, { beforeEach } from 'node:test'
import { createPinia, setActivePinia } from 'pinia'
import { tabStorageService } from '../../services/tabStorageService.ts'
import { useTabsStore } from '../tabsStore.ts'

beforeEach(() => {
  setActivePinia(createPinia())
})

test('tabsStore initializes with default state without minimize or placement', () => {
  const store = useTabsStore()
  assert.deepEqual(store.tabs, [])
  assert.equal(store.activeTabId, null)
  assert.equal(store.activeTab, null)
  assert.equal('placement' in store, false)
  assert.equal('isMinimized' in store, false)
  assert.equal('toggleMinimize' in store, false)
  assert.equal('setPlacement' in store, false)
})

test('tabsStore.openTab adds tab and deduplicates by route', () => {
  const store = useTabsStore()

  const tab1 = store.openTab({
    route: '/workloads?tab=deployments',
    title: 'Deployments',
    iconName: 'Layers',
    category: 'workloads'
  })

  assert.equal(store.tabs.length, 1)
  assert.equal(store.activeTabId, tab1.id)
  assert.equal(store.activeTab?.title, 'Deployments')

  // Open a second tab
  const tab2 = store.openTab({
    route: '/workloads?tab=pods',
    title: 'Pods',
    iconName: 'Box',
    category: 'workloads'
  })

  assert.equal(store.tabs.length, 2)
  assert.equal(store.activeTabId, tab2.id)

  // Re-opening the first tab does not duplicate, but activates it
  const reOpened = store.openTab({
    route: '/workloads?tab=deployments',
    title: 'Deployments'
  })

  assert.equal(store.tabs.length, 2)
  assert.equal(reOpened.id, tab1.id)
  assert.equal(store.activeTabId, tab1.id)
})

test('tabsStore.closeTab closes active tab and switches to adjacent tab', () => {
  const store = useTabsStore()

  store.openTab({ route: '/tab1', title: 'Tab 1' })
  store.openTab({ route: '/tab2', title: 'Tab 2' })
  store.openTab({ route: '/tab3', title: 'Tab 3' })

  assert.equal(store.activeTabId, '/tab3')

  // Close active tab 3 (last tab) -> switches to tab 2
  store.closeTab('/tab3')
  assert.equal(store.tabs.length, 2)
  assert.equal(store.activeTabId, '/tab2')

  // Close inactive tab 1 -> active tab 2 remains active
  store.closeTab('/tab1')
  assert.equal(store.tabs.length, 1)
  assert.equal(store.activeTabId, '/tab2')

  // Close only remaining tab -> activeTabId becomes null
  store.closeTab('/tab2')
  assert.equal(store.tabs.length, 0)
  assert.equal(store.activeTabId, null)
})

test('tabsStore.closeOtherTabs keeps only target tab', () => {
  const store = useTabsStore()

  store.openTab({ route: '/tab1', title: 'Tab 1' })
  store.openTab({ route: '/tab2', title: 'Tab 2' })
  store.openTab({ route: '/tab3', title: 'Tab 3' })

  store.closeOtherTabs('/tab2')

  assert.equal(store.tabs.length, 1)
  assert.equal(store.tabs[0]?.route, '/tab2')
  assert.equal(store.activeTabId, '/tab2')
})

test('tabsStore.closeTabsToTheRight removes subsequent tabs', () => {
  const store = useTabsStore()

  store.openTab({ route: '/tab1', title: 'Tab 1' })
  store.openTab({ route: '/tab2', title: 'Tab 2' })
  store.openTab({ route: '/tab3', title: 'Tab 3' })

  store.closeTabsToTheRight('/tab1')

  assert.equal(store.tabs.length, 1)
  assert.equal(store.tabs[0]?.route, '/tab1')
  assert.equal(store.activeTabId, '/tab1')
})

test('tabsStore.closeAllTabs clears all tabs', () => {
  const store = useTabsStore()

  store.openTab({ route: '/tab1', title: 'Tab 1' })
  store.openTab({ route: '/tab2', title: 'Tab 2' })

  store.closeAllTabs()

  assert.equal(store.tabs.length, 0)
  assert.equal(store.activeTabId, null)
})

test('tabsStore reorderTabs reorders array items', () => {
  const store = useTabsStore()

  store.openTab({ route: '/tab1', title: 'Tab 1' })
  store.openTab({ route: '/tab2', title: 'Tab 2' })
  store.openTab({ route: '/tab3', title: 'Tab 3' })

  store.reorderTabs(0, 2)
  assert.deepEqual(
    store.tabs.map((t) => t.route),
    ['/tab2', '/tab3', '/tab1']
  )
})

test('tabsStore.init hydrates tabs state from storage', async () => {
  const store = useTabsStore()
  const originalLoad = tabStorageService.loadTabsState

  tabStorageService.loadTabsState = async () => ({
    tabs: [
      {
        id: '/workloads?tab=deployments',
        title: 'Deployments',
        route: '/workloads?tab=deployments',
        path: '/workloads',
        closable: true
      }
    ],
    activeTabId: '/workloads?tab=deployments'
  })

  try {
    await store.init('my-cluster')
    assert.equal(store.tabs.length, 1)
    assert.equal(store.activeTabId, '/workloads?tab=deployments')
  } finally {
    tabStorageService.loadTabsState = originalLoad
  }
})

test('tabsStore.openTab automatically resolves title and icon if omitted or default', () => {
  const store = useTabsStore()

  // Opening deployment tab without title
  const depTab = store.openTab({ route: '/workloads?tab=deployments' })
  assert.equal(depTab.title, 'Deployments')
  assert.equal(depTab.iconName, 'Layers')

  // Opening statefulsets tab without title
  const stsTab = store.openTab({ route: '/workloads?tab=statefulsets' })
  assert.equal(stsTab.title, 'StatefulSets')
  assert.equal(stsTab.iconName, 'Database')
})

test('tabsStore.syncWithRoute automatically resolves title from route metadata', () => {
  const store = useTabsStore()

  const tab = store.syncWithRoute('/workloads', { tab: 'pods' })
  assert.ok(tab)
  assert.equal(tab.title, 'Pods')
  assert.equal(tab.iconName, 'Box')
})

test('tabsStore.init auto-heals legacy tabs with "Tab" title', async () => {
  const store = useTabsStore()
  const originalLoad = tabStorageService.loadTabsState

  tabStorageService.loadTabsState = async () => ({
    tabs: [
      {
        id: '/workloads?tab=statefulsets',
        title: 'Tab',
        route: '/workloads?tab=statefulsets',
        path: '/workloads',
        query: { tab: 'statefulsets' },
        closable: true
      },
      {
        id: '/workloads?tab=deployments',
        title: 'Tab',
        route: '/workloads?tab=deployments',
        path: '/workloads',
        query: { tab: 'deployments' },
        closable: true
      }
    ],
    activeTabId: '/workloads?tab=deployments'
  })

  try {
    await store.init('heal-cluster')
    assert.equal(store.tabs.length, 2)
    assert.equal(store.tabs[0]?.title, 'StatefulSets')
    assert.equal(store.tabs[1]?.title, 'Deployments')
  } finally {
    tabStorageService.loadTabsState = originalLoad
  }
})

test('tabsStore upgrades existing tab with "Tab" title when accessed', () => {
  const store = useTabsStore()

  // Simulate existing tab with "Tab"
  store.tabs.push({
    id: '/workloads?tab=deployments',
    title: 'Tab',
    route: '/workloads?tab=deployments',
    path: '/workloads',
    closable: true
  })

  const reopened = store.openTab({ route: '/workloads?tab=deployments' })
  assert.equal(reopened.title, 'Deployments')
})

test('tabsStore.openTab resolves workload title for log routes', () => {
  const store = useTabsStore()
  const tab = store.openTab({ route: '/logs?workload=my-service' })
  assert.equal(tab.title, 'my-service')
  assert.equal(tab.iconName, 'FileText')
  assert.equal(tab.category, 'logs')
})

test('tabsStore.syncWithRoute resolves workload title for log routes', () => {
  const store = useTabsStore()
  const tab = store.syncWithRoute('/logs', { workload: 'order-api' })
  assert.ok(tab)
  assert.equal(tab.title, 'order-api')
  assert.equal(tab.iconName, 'FileText')
  assert.equal(tab.category, 'logs')
})

test('tabsStore.init auto-heals legacy log tabs with generic "Logs" or "Tab" title', async () => {
  const store = useTabsStore()
  const originalLoad = tabStorageService.loadTabsState

  tabStorageService.loadTabsState = async () => ({
    tabs: [
      {
        id: '/logs?workload=payment-svc',
        title: 'Logs',
        route: '/logs?workload=payment-svc',
        path: '/logs',
        query: { workload: 'payment-svc' },
        closable: true
      },
      {
        id: '/logs?workload=auth-svc',
        title: 'Tab',
        route: '/logs?workload=auth-svc',
        path: '/logs',
        query: { workload: 'auth-svc' },
        closable: true
      }
    ],
    activeTabId: '/logs?workload=payment-svc'
  })

  try {
    await store.init('heal-log-cluster')
    const paymentTab = store.tabs.find((t) => t.id === '/logs?workload=payment-svc')
    const authTab = store.tabs.find((t) => t.id === '/logs?workload=auth-svc')
    assert.equal(paymentTab?.title, 'payment-svc')
    assert.equal(authTab?.title, 'auth-svc')
  } finally {
    tabStorageService.loadTabsState = originalLoad
  }
})

test('tabsStore upgrades existing log tab with "Logs" title when accessed with workload query', () => {
  const store = useTabsStore()

  store.tabs.push({
    id: '/logs?workload=my-service',
    title: 'Logs',
    route: '/logs?workload=my-service',
    path: '/logs',
    query: { workload: 'my-service' },
    closable: true
  })

  const reopened = store.openTab({ route: '/logs?workload=my-service' })
  assert.equal(reopened.title, 'my-service')
})
