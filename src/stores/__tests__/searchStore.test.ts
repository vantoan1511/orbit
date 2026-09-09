import assert from 'node:assert/strict'
import test, { beforeEach } from 'node:test'
import { createPinia, setActivePinia } from 'pinia'
import { scoreSearchItem, useSearchStore } from '../searchStore.ts'
import { useKubernetesStore } from '../kubernetesStore.ts'

beforeEach(() => {
  setActivePinia(createPinia())
})

test('scoreSearchItem: calculates match scores accurately and handles edge cases', () => {
  // Exact match
  assert.equal(scoreSearchItem('nginx', 'nginx'), 100)
  // Case-insensitive exact match
  assert.equal(scoreSearchItem('NGINX', 'nginx'), 100)

  // Prefix match
  const prefixScore = scoreSearchItem('ng', 'nginx-deployment')
  assert.equal(prefixScore, 80)

  // Contains match
  const containsScore = scoreSearchItem('deploy', 'nginx-deployment')
  assert.equal(containsScore, 50)

  // Subtitle / namespace / kind match
  const nsScore = scoreSearchItem(
    'prod',
    'my-service',
    'production • Service',
    'production',
    'Service'
  )
  assert.equal(nsScore, 30)

  // No match
  assert.equal(scoreSearchItem('redis', 'nginx'), 0)

  // Empty query returns 0
  assert.equal(scoreSearchItem('', 'nginx'), 0)

  // Special regex characters in query do not throw and match literally
  assert.equal(scoreSearchItem('app[0]', 'app[0]-worker'), 80)
  assert.equal(scoreSearchItem('v1.2*', 'v1.2*'), 100)
})

test('useSearchStore: state initialization and open/close/toggle', () => {
  const store = useSearchStore()

  assert.equal(store.isOpen, false)
  assert.equal(store.searchQuery, '')
  assert.equal(store.activeCategory, 'all')
  assert.equal(store.selectedIndex, 0)

  store.open()
  assert.equal(store.isOpen, true)
  assert.equal(store.selectedIndex, 0)

  store.searchQuery = 'test'
  store.close()
  assert.equal(store.isOpen, false)

  store.toggle()
  assert.equal(store.isOpen, true)
  assert.equal(store.searchQuery, '') // Resets on open
})

test('useSearchStore: category switching updates activeCategory and resets selectedIndex', () => {
  const store = useSearchStore()
  store.selectedIndex = 3
  store.setCategory('logs')

  assert.equal(store.activeCategory, 'logs')
  assert.equal(store.selectedIndex, 0)
})

test('useSearchStore: selectNext and selectPrev clamp within bounds', () => {
  const store = useSearchStore()

  // With 0 results (when closed)
  store.selectNext()
  assert.equal(store.selectedIndex, 0)
  store.selectPrev()
  assert.equal(store.selectedIndex, 0)

  // With open store and navigation items
  store.open()
  store.setCategory('navigation')
  const total = store.filteredResults.length
  assert.ok(total >= 3)

  store.selectedIndex = 0
  store.selectNext()
  assert.equal(store.selectedIndex, 1)
  store.selectNext()
  assert.equal(store.selectedIndex, 2)

  // Wrap to first
  store.selectedIndex = total - 1
  store.selectNext()
  assert.equal(store.selectedIndex, 0)

  // Wrap back to last
  store.selectPrev()
  assert.equal(store.selectedIndex, total - 1)
})

test('useSearchStore: executeSelected executes the item action and closes the store', () => {
  const store = useSearchStore()
  let navigatedPath = ''

  store.setNavigationHandler((target) => {
    navigatedPath = target.path
  })

  store.open()
  store.searchQuery = 'Settings'
  assert.ok(store.filteredResults.length > 0)
  assert.equal(store.filteredResults[0]?.title, 'Settings')

  store.executeSelected()
  assert.equal(navigatedPath, '/settings')
  assert.equal(store.isOpen, false)
})

test('useSearchStore: searches resources from kubernetes store', () => {
  const k8sStore = useKubernetesStore()
  k8sStore.pods = [
    {
      name: 'test-api-pod',
      namespace: 'production',
      status: 'Running',
      node: 'worker-1',
      ip: '10.244.0.5',
      restarts: 0,
      age: '2d',
      containers: []
    }
  ]

  const store = useSearchStore()
  store.open()
  store.searchQuery = 'test-api'

  const results = store.filteredResults
  assert.ok(results.length > 0)
  const pod = results.find((r) => r.title === 'test-api-pod')
  assert.ok(pod)
  assert.equal(pod.kind, 'Pod')
  assert.equal(pod.namespace, 'production')
  assert.equal(pod.category, 'resources')
})
