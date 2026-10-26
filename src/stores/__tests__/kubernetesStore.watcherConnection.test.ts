import assert from 'node:assert/strict'
import test, { beforeEach } from 'node:test'
import { createPinia, setActivePinia } from 'pinia'
import { useKubernetesStore } from '../kubernetesStore.ts'

beforeEach(() => {
  setActivePinia(createPinia())
})

test('kubernetesStore watcher connection state management', () => {
  const store = useKubernetesStore()

  // (a) default is connected
  assert.equal(store.watcherConnectionState, 'connected')

  // Set active cluster
  store.setActiveClusterId('cluster-a')

  // (b) degraded event for active cluster updates state
  store.setWatcherConnectionState('cluster-a', 'degraded')
  assert.equal(store.watcherConnectionState, 'degraded')

  // (c) event for a different cluster is ignored
  store.setWatcherConnectionState('cluster-b', 'suspended')
  assert.equal(store.watcherConnectionState, 'degraded')

  // (d) activeClusterChanged to different cluster resets to connected
  store.setActiveClusterId('cluster-b')
  assert.equal(store.watcherConnectionState, 'connected')
})

test('same-id setActiveClusterId resets suspended to connected without clearing data', () => {
  const store = useKubernetesStore()
  store.setActiveClusterId('c1')
  store.setWatcherConnectionState('c1', 'suspended')
  assert.equal(store.watcherConnectionState, 'suspended')

  // Seed some cluster data to verify it is NOT cleared when active cluster ID does not change
  store.pods = [{ metadata: { uid: 'p1' } }] as unknown as typeof store.pods
  const initialPods = store.pods

  // AC-1: same-id call resets connection state to connected
  store.setActiveClusterId('c1')
  assert.equal(store.watcherConnectionState, 'connected')
  assert.equal(store.pods, initialPods, 'Pod list reference must remain unchanged on same-id call')
  assert.equal(store.pods.length, 1)

  // An event for the active cluster after reset applies normally
  store.setWatcherConnectionState('c1', 'degraded')
  assert.equal(store.watcherConnectionState, 'degraded')
})
