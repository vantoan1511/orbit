import assert from 'node:assert/strict'
import test, { beforeEach } from 'node:test'
import { createPinia, setActivePinia } from 'pinia'
import { useKubernetesStore } from '../kubernetesStore.ts'
import { KUBERNETES_RESOURCE_KIND } from '../../types/kubernetes.ts'
import { kubernetesService } from '../../services/kubernetesService.ts'

beforeEach(() => {
  setActivePinia(createPinia())
})

test('ensureKindLoaded dispatches fetch and sets loading=true when kind is not yet loaded', async () => {
  const store = useKubernetesStore()
  store.setEngineReady(true)
  store.setActiveClusterId('cluster-1')
  store.resetAllLoading()

  let getSecretsCalled = 0
  const originalGetSecrets = kubernetesService.getSecrets
  kubernetesService.getSecrets = async () => {
    getSecretsCalled++
  }

  try {
    assert.equal(store.secretsLoading, false)
    const promise = store.ensureKindLoaded(KUBERNETES_RESOURCE_KIND.Secret)
    assert.equal(store.secretsLoading, true, 'secretsLoading must be true during dispatch')
    await promise
    assert.equal(getSecretsCalled, 1, 'kubernetesService.getSecrets must be called')
  } finally {
    kubernetesService.getSecrets = originalGetSecrets
  }
})

test('ensureKindLoaded is a no-op when kind was already loaded via setter or is currently loading', async () => {
  const store = useKubernetesStore()
  store.setEngineReady(true)
  store.setActiveClusterId('cluster-1')
  store.resetAllLoading()

  // 1. Mark ConfigMap loaded via setter
  store.setConfigMaps([
    {
      name: 'cm-1',
      namespace: 'default',
      keysCount: 2,
      dataKeys: ['a', 'b'],
      age: '10m',
      labels: {}
    }
  ])
  assert.equal(store.configMapsLoading, false)

  let getConfigMapsCalled = 0
  const originalGetConfigMaps = kubernetesService.getConfigMaps
  kubernetesService.getConfigMaps = async () => {
    getConfigMapsCalled++
  }

  try {
    await store.ensureKindLoaded(KUBERNETES_RESOURCE_KIND.ConfigMap)
    assert.equal(
      getConfigMapsCalled,
      0,
      'Must not dispatch fetch if kind was already marked loaded'
    )
    assert.equal(store.configMapsLoading, false)

    // 2. If kind is currently loading, ensureKindLoaded is also a no-op
    store.resetAllLoading()
    let getPodsCalled = 0
    const originalGetPods = kubernetesService.getPods
    kubernetesService.getPods = async () => {
      getPodsCalled++
    }
    try {
      store.podsLoading = true
      await store.ensureKindLoaded(KUBERNETES_RESOURCE_KIND.Pod)
      assert.equal(getPodsCalled, 0, 'Must not dispatch fetch if kind is already loading')
    } finally {
      kubernetesService.getPods = originalGetPods
    }
  } finally {
    kubernetesService.getConfigMaps = originalGetConfigMaps
  }
})

test('setActiveClusterId resets loadedKinds so ensureKindLoaded fetches for new cluster', async () => {
  const store = useKubernetesStore()
  store.setEngineReady(true)
  store.setActiveClusterId('cluster-1')
  store.resetAllLoading()

  // Mark ConfigMap loaded on cluster-1
  store.setConfigMaps([{ name: 'cm-1', namespace: 'default', keysCount: 1, age: '1m', labels: {} }])

  let getConfigMapsCalled = 0
  const originalGetConfigMaps = kubernetesService.getConfigMaps
  kubernetesService.getConfigMaps = async () => {
    getConfigMapsCalled++
  }

  try {
    // Should be no-op on cluster-1
    await store.ensureKindLoaded(KUBERNETES_RESOURCE_KIND.ConfigMap)
    assert.equal(getConfigMapsCalled, 0)

    // Switch cluster
    store.setActiveClusterId('cluster-2')
    store.resetAllLoading()
    getConfigMapsCalled = 0

    // Now ensureKindLoaded should fetch because loadedKinds was cleared
    await store.ensureKindLoaded(KUBERNETES_RESOURCE_KIND.ConfigMap)
    assert.equal(getConfigMapsCalled, 1, 'Must dispatch fetch on new cluster context')
  } finally {
    kubernetesService.getConfigMaps = originalGetConfigMaps
  }
})
