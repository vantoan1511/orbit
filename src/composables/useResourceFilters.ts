import { useKubernetesStore } from '@/stores/kubernetesStore'
import { useTableFilterStore } from '@/stores/tableFilterStore'
import { computed, ref, unref, watch, type MaybeRef, type Ref } from 'vue'

export interface ResourceItem {
  name: string
  namespace?: string
  [key: string]: unknown
}

const SYSTEM_NAMESPACES = ['kube-system', 'kube-public', 'kube-node-lease', 'monitoring', 'logging']

export function useResourceFilters<T extends ResourceItem>(
  resources: Ref<T[]>,
  searchFields: (keyof T)[] = ['name'],
  storeKey?: string,
  hideNamespaceFilter: MaybeRef<boolean> = false
) {
  const filterStore = storeKey ? useTableFilterStore() : null
  const k8sStore = useKubernetesStore()
  const stored =
    storeKey && filterStore
      ? filterStore.getFilters(storeKey, k8sStore.activeClusterId || undefined)
      : null

  const searchQuery = ref(stored?.searchQuery ?? '')
  const selectedNamespace = ref<string[]>(stored ? [...stored.selectedNamespace] : [])

  const shouldHideNamespace = computed(() => unref(hideNamespaceFilter))

  if (storeKey && filterStore) {
    watch(searchQuery, (val) => {
      filterStore.setFilter(storeKey, 'searchQuery', val, k8sStore.activeClusterId || undefined)
    })
    watch(
      selectedNamespace,
      (val) => {
        filterStore.setFilter(
          storeKey,
          'selectedNamespace',
          [...val],
          k8sStore.activeClusterId || undefined
        )
      },
      { deep: true }
    )

    watch(
      () => k8sStore.activeClusterId,
      (newClusterId) => {
        const state = filterStore.getFilters(storeKey, newClusterId || undefined)
        searchQuery.value = state.searchQuery
        selectedNamespace.value = [...state.selectedNamespace]
      }
    )

    watch(
      () => k8sStore.namespaceList,
      (newList) => {
        if (shouldHideNamespace.value) return
        const cluster = k8sStore.activeClusterId || undefined
        const state = filterStore.getFilters(storeKey, cluster)
        if (!state.isNamespaceInitialized && newList.length > 0) {
          const userNamespaces = newList
            .map((n) => n.name)
            .filter((n) => !SYSTEM_NAMESPACES.includes(n))

          selectedNamespace.value = userNamespaces
          filterStore.setFilter(storeKey, 'isNamespaceInitialized', true, cluster)
        } else if (
          state.isNamespaceInitialized &&
          newList.length > 0 &&
          selectedNamespace.value.length > 0
        ) {
          const availableNamespaces = new Set(newList.map((n) => n.name))
          const validNamespaces = selectedNamespace.value.filter((n) => availableNamespaces.has(n))

          if (validNamespaces.length === 0) {
            const userNamespaces = newList
              .map((n) => n.name)
              .filter((n) => !SYSTEM_NAMESPACES.includes(n))
            selectedNamespace.value = userNamespaces
          } else if (validNamespaces.length !== selectedNamespace.value.length) {
            selectedNamespace.value = validNamespaces
          }
        }
      },
      { immediate: true }
    )
  }

  const filteredResources = computed(() => {
    const rawQuery = searchQuery.value.trim().toLowerCase()
    const hasQuery = rawQuery.length > 0

    const filterNamespace = !shouldHideNamespace.value && selectedNamespace.value.length > 0
    const selectedNsSet = filterNamespace ? new Set(selectedNamespace.value) : null

    // Fast-path: if no search query and no namespace filter, return entire array directly
    if (!hasQuery && !filterNamespace) {
      return resources.value
    }

    return resources.value.filter((item) => {
      // 1. Namespace filter: check before search for early exit
      if (filterNamespace && selectedNsSet) {
        const ns = item.namespace
        const isNamespaced =
          typeof ns === 'string' &&
          ns.trim().length > 0 &&
          ns !== '-' &&
          ns !== 'Cluster' &&
          ns !== 'All' &&
          item.scope !== 'Cluster'

        if (isNamespaced && !selectedNsSet.has(ns)) {
          return false
        }
      }

      // 2. Search Query filter
      if (hasQuery) {
        let matches = false
        for (let i = 0; i < searchFields.length; i++) {
          const field = searchFields[i]!
          const val = item[field]
          if (typeof val === 'string') {
            if (val.toLowerCase().includes(rawQuery)) {
              matches = true
              break
            }
          } else if (Array.isArray(val)) {
            for (let j = 0; j < val.length; j++) {
              const v = val[j]
              if (typeof v === 'string' && v.toLowerCase().includes(rawQuery)) {
                matches = true
                break
              }
            }
            if (matches) break
          }
        }
        if (!matches) return false
      }

      return true
    })
  })

  return {
    searchQuery,
    selectedNamespace,
    filteredResources
  }
}
