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

  // (d) activeClusterChanged resets to connected
  store.setActiveClusterId('cluster-b')
  assert.equal(store.watcherConnectionState, 'connected')
})
