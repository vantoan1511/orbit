import test from 'node:test'
import assert from 'node:assert/strict'
import { ref } from 'vue'
import { storage } from '../../services/nativeService.ts'
import { categories } from '../../components/layout/sidebar/navigation.ts'
import { useSidebarState } from '../useSidebarState.ts'

test('useSidebarState initializes with default expanded state', () => {
  const currentPath = ref('/workloads')
  const hasActiveCluster = ref(true)

  const { isCollapsed, activeTab } = useSidebarState({
    storageKey: 'test_sidebar_collapsed',
    currentPath,
    hasActiveCluster
  })

  assert.equal(isCollapsed.value, false)
  assert.equal(activeTab.value, 'clusters')
})

test('useSidebarState loads collapsed state from native storage', async () => {
  const originalGetData = storage.getData
  storage.getData = async (key: string) => {
    if (key === 'test_sidebar_collapsed_load') {
      return 'true'
    }
    throw new Error('Not found')
  }

  try {
    const currentPath = ref('/workloads')
    const hasActiveCluster = ref(true)

    const { isCollapsed, activeTab, loadStoredState } = useSidebarState({
      storageKey: 'test_sidebar_collapsed_load',
      currentPath,
      hasActiveCluster
    })

    await loadStoredState()
    assert.equal(isCollapsed.value, true)
    assert.equal(activeTab.value, null)
  } finally {
    storage.getData = originalGetData
  }
})

test('useSidebarState migrates legacy localStorage collapsed preference', async () => {
  const originalGetData = storage.getData
  const originalSetData = storage.setData

  let savedNativeKey = ''
  let savedNativeVal = ''
  let removedLocalKey = ''

  storage.getData = async () => {
    throw new Error('Not in native storage')
  }
  storage.setData = async (key: string, val?: string | null) => {
    savedNativeKey = key
    savedNativeVal = val || ''
  }

  const originalLocalStorage = globalThis.localStorage
  const mockLocalStorage = {
    getItem(key: string) {
      if (key === 'test_legacy_sidebar_collapsed') {
        return 'true'
      }
      return null
    },
    removeItem(key: string) {
      removedLocalKey = key
    }
  }
  globalThis.localStorage = mockLocalStorage as unknown as Storage

  try {
    const { isCollapsed, activeTab, loadStoredState } = useSidebarState({
      storageKey: 'test_legacy_sidebar_collapsed',
      hasActiveCluster: ref(true)
    })

    await loadStoredState()
    assert.equal(isCollapsed.value, true)
    assert.equal(activeTab.value, null)
    assert.equal(savedNativeKey, 'test_legacy_sidebar_collapsed')
    assert.equal(savedNativeVal, 'true')
    assert.equal(removedLocalKey, 'test_legacy_sidebar_collapsed')
  } finally {
    storage.getData = originalGetData
    storage.setData = originalSetData
    globalThis.localStorage = originalLocalStorage
  }
})

test('syncWithRoute does NOT expand sidebar when isCollapsed is true', () => {
  const currentPath = ref('/logs')
  const hasActiveCluster = ref(true)

  const { isCollapsed, activeTab, syncWithRoute, handleCollapse } = useSidebarState({
    storageKey: 'test_sidebar_sync_collapsed',
    currentPath,
    hasActiveCluster
  })

  // Start by collapsing the sidebar
  handleCollapse()
  assert.equal(isCollapsed.value, true)
  assert.equal(activeTab.value, null)

  // Navigate to workloads (different category from logs)
  currentPath.value = '/workloads'
  syncWithRoute('/workloads')

  // Sidebar must remain collapsed and activeTab must remain null
  assert.equal(isCollapsed.value, true)
  assert.equal(activeTab.value, null)
})

test('syncWithRoute updates activeTab to matching category when isCollapsed is false', () => {
  const currentPath = ref('/logs')
  const hasActiveCluster = ref(true)

  const { isCollapsed, activeTab, syncWithRoute } = useSidebarState({
    storageKey: 'test_sidebar_sync_open',
    currentPath,
    hasActiveCluster
  })

  // Set to a normal category first (not 'clusters')
  activeTab.value = 'logs'
  assert.equal(isCollapsed.value, false)

  // Switching route to workloads updates activeTab
  currentPath.value = '/workloads'
  syncWithRoute('/workloads')
  assert.equal(activeTab.value, 'workloads')
  assert.equal(isCollapsed.value, false)

  // Switching route to network updates activeTab
  currentPath.value = '/network'
  syncWithRoute('/network')
  assert.equal(activeTab.value, 'network')
})

test('syncWithRoute does not override activeTab if activeTab is clusters', () => {
  const currentPath = ref('/workloads')
  const hasActiveCluster = ref(true)

  const { activeTab, syncWithRoute } = useSidebarState({
    currentPath,
    hasActiveCluster
  })

  activeTab.value = 'clusters'
  syncWithRoute('/workloads')
  assert.equal(activeTab.value, 'clusters')
})

test('toggleCategory expands sidebar and navigates when collapsed', () => {
  let navigatedRoute = ''
  const hasActiveCluster = ref(true)

  const { isCollapsed, activeTab, handleCollapse, toggleCategory } = useSidebarState({
    hasActiveCluster,
    onNavigate: (route) => {
      navigatedRoute = route
    }
  })

  handleCollapse()
  assert.equal(isCollapsed.value, true)
  assert.equal(activeTab.value, null)

  const workloadsCat = categories.find((c) => c.id === 'workloads')!
  toggleCategory(workloadsCat)

  assert.equal(isCollapsed.value, false)
  assert.equal(activeTab.value, 'workloads')
  assert.equal(navigatedRoute, '/workloads?tab=deployments')
})

test('toggleCategory collapses sidebar and clears activeTab when active category is clicked', () => {
  const hasActiveCluster = ref(true)

  const { isCollapsed, activeTab, toggleCategory } = useSidebarState({
    hasActiveCluster
  })

  activeTab.value = 'workloads'
  isCollapsed.value = false

  const workloadsCat = categories.find((c) => c.id === 'workloads')!
  toggleCategory(workloadsCat)

  assert.equal(isCollapsed.value, true)
  assert.equal(activeTab.value, null)
})

test('toggleCategory switches category and preserves expanded state when clicking different category', () => {
  let navigatedRoute = ''
  const hasActiveCluster = ref(true)

  const { isCollapsed, activeTab, toggleCategory } = useSidebarState({
    hasActiveCluster,
    onNavigate: (route) => {
      navigatedRoute = route
    }
  })

  activeTab.value = 'workloads'
  isCollapsed.value = false

  const networkCat = categories.find((c) => c.id === 'network')!
  toggleCategory(networkCat)

  assert.equal(isCollapsed.value, false)
  assert.equal(activeTab.value, 'network')
  assert.equal(navigatedRoute, '/network?tab=services')
})

test('toggleCategory is a no-op if category requires cluster and hasActiveCluster is false', () => {
  const hasActiveCluster = ref(false)

  const { isCollapsed, activeTab, toggleCategory } = useSidebarState({
    hasActiveCluster
  })

  activeTab.value = 'clusters'
  const workloadsCat = categories.find((c) => c.id === 'workloads')!
  toggleCategory(workloadsCat)

  assert.equal(activeTab.value, 'clusters')
  assert.equal(isCollapsed.value, false)
})

test('handleCollapse sets isCollapsed to true and persists state', async () => {
  let savedKey = ''
  let savedVal = ''
  const originalSetData = storage.setData
  storage.setData = async (key: string, val?: string | null) => {
    savedKey = key
    savedVal = val || ''
  }

  try {
    const { isCollapsed, activeTab, handleCollapse } = useSidebarState({
      storageKey: 'test_collapse_persist',
      hasActiveCluster: ref(true)
    })

    activeTab.value = 'workloads'
    handleCollapse()

    assert.equal(isCollapsed.value, true)
    assert.equal(activeTab.value, null)
    assert.equal(savedKey, 'test_collapse_persist')
    assert.equal(savedVal, 'true')
  } finally {
    storage.setData = originalSetData
  }
})

test('handleClusterSwitched restores category when expanded or keeps collapsed when collapsed', () => {
  const currentPath = ref('/workloads')
  const { isCollapsed, activeTab, handleClusterSwitched, handleCollapse } = useSidebarState({
    currentPath,
    hasActiveCluster: ref(true)
  })

  // When expanded, switches to route category
  activeTab.value = 'clusters'
  handleClusterSwitched('/workloads')
  assert.equal(activeTab.value, 'workloads')

  // When collapsed, remains collapsed
  handleCollapse()
  handleClusterSwitched('/workloads')
  assert.equal(isCollapsed.value, true)
  assert.equal(activeTab.value, null)
})
