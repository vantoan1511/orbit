import { useKubernetesStore } from '@/stores/kubernetesStore'
import { useTableFilterStore } from '@/stores/tableFilterStore'
import { computed, ref, watch, type Ref } from 'vue'

export interface ResourceItem {
  name: string
  namespace?: string
  [key: string]: unknown
}

const SYSTEM_NAMESPACES = ['kube-system', 'kube-public', 'kube-node-lease', 'monitoring', 'logging']

export function useResourceFilters<T extends ResourceItem>(
  resources: Ref<T[]>,
  searchFields: (keyof T)[] = ['name'],
  storeKey?: string
) {
  const filterStore = storeKey ? useTableFilterStore() : null
  const stored = storeKey && filterStore ? filterStore.getFilters(storeKey) : null
  const k8sStore = useKubernetesStore()

  const searchQuery = ref(stored?.searchQuery ?? '')
  const selectedNamespace = ref<string[]>(stored ? [...stored.selectedNamespace] : [])

  if (storeKey && filterStore) {
    watch(searchQuery, (val) => {
      filterStore.setFilter(storeKey, 'searchQuery', val)
    })
    watch(
      selectedNamespace,
      (val) => {
        filterStore.setFilter(storeKey, 'selectedNamespace', [...val])
      },
      { deep: true }
    )

    watch(
      () => k8sStore.namespaceList,
      (newList) => {
        const state = filterStore.getFilters(storeKey)
        if (!state.isNamespaceInitialized && newList.length > 0) {
          const userNamespaces = newList
            .map((n) => n.name)
            .filter((n) => !SYSTEM_NAMESPACES.includes(n))

          selectedNamespace.value = userNamespaces
          filterStore.setFilter(storeKey, 'isNamespaceInitialized', true)
        }
      },
      { immediate: true }
    )
  }

  const filteredResources = computed(() => {
    return resources.value.filter((item) => {
      // 1. Search Query filter
      if (searchQuery.value) {
        const query = searchQuery.value.toLowerCase()
        const matches = searchFields.some((field) => {
          const val = item[field]
          if (typeof val === 'string') return val.toLowerCase().includes(query)
          if (Array.isArray(val))
            return val.some(
              (v: unknown) => typeof v === 'string' && v.toLowerCase().includes(query)
            )
          return false
        })
        if (!matches) return false
      }

      // 2. Namespace filter
      if (
        selectedNamespace.value.length > 0 &&
        (!item.namespace || !selectedNamespace.value.includes(item.namespace))
      ) {
        return false
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
