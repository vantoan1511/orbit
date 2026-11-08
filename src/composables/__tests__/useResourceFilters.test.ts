import assert from 'node:assert/strict'
import test, { beforeEach } from 'node:test'
import { createPinia, setActivePinia } from 'pinia'
import { computed, effectScope, nextTick, ref } from 'vue'
import { useKubernetesStore } from '../../stores/kubernetesStore.ts'
import { useResourceFilters, type ResourceItem } from '../useResourceFilters.ts'

beforeEach(() => {
  setActivePinia(createPinia())
})

test('useResourceFilters: when isHydrated is ref(false), does not initialize selectedNamespace and filteredResources returns [] without reading resources getter', async () => {
  const scope = effectScope()
  await scope.run(async () => {
    const k8sStore = useKubernetesStore()
    k8sStore.setNamespaces([
      { name: 'kube-system', status: 'Active' },
      { name: 'default', status: 'Active' },
      { name: 'my-app', status: 'Active' }
    ])

    let getterRead = false
    const rawData = [
      { name: 'pod-1', namespace: 'default' },
      { name: 'pod-2', namespace: 'my-app' }
    ]
    const resources = computed<ResourceItem[]>(() => {
      getterRead = true
      return rawData
    })

    const isHydrated = ref(false)
    const { selectedNamespace, filteredResources } = useResourceFilters(
      resources,
      ['name'],
      'test-table-1',
      false,
      isHydrated
    )

    // With isHydrated=false:
    assert.deepEqual(
      selectedNamespace.value,
      [],
      'selectedNamespace should not initialize when !isHydrated'
    )
    assert.deepEqual(
      filteredResources.value,
      [],
      'filteredResources should return [] when !isHydrated'
    )
    assert.equal(
      getterRead,
      false,
      'resources computed getter should not be evaluated when !isHydrated'
    )

    // When isHydrated flips to true:
    isHydrated.value = true
    await nextTick()
    assert.deepEqual(
      selectedNamespace.value,
      ['default', 'my-app'],
      'selectedNamespace initializes with non-system namespaces once hydrated'
    )
    assert.equal(filteredResources.value.length, 2, 'filteredResources returns items once hydrated')
    assert.equal(getterRead, true, 'resources computed getter was evaluated after hydration')
  })
  scope.stop()
})

test('useResourceFilters: when isHydrated is omitted, initializes and filters immediately (backward compatibility)', () => {
  const scope = effectScope()
  scope.run(() => {
    const k8sStore = useKubernetesStore()
    k8sStore.setNamespaces([
      { name: 'kube-system', status: 'Active' },
      { name: 'default', status: 'Active' }
    ])

    const rawData = [{ name: 'pod-1', namespace: 'default' }]
    const resources = ref<ResourceItem[]>(rawData)

    const { selectedNamespace, filteredResources } = useResourceFilters(
      resources,
      ['name'],
      'test-table-2'
    )

    assert.deepEqual(
      selectedNamespace.value,
      ['default'],
      'selectedNamespace initializes immediately'
    )
    assert.equal(filteredResources.value.length, 1, 'filteredResources filters immediately')
  })
  scope.stop()
})
