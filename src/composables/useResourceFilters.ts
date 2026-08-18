import { useTableFilterStore } from '@/stores/tableFilterStore'
import { computed, ref, watch, type Ref } from 'vue'

export interface ResourceItem {
  name: string
  namespace?: string
  [key: string]: unknown
}

export function useResourceFilters<T extends ResourceItem>(
  resources: Ref<T[]>,
  searchFields: (keyof T)[] = ['name'],
  storeKey?: string
) {
  const filterStore = storeKey ? useTableFilterStore() : null
  const stored = storeKey && filterStore ? filterStore.getFilters(storeKey) : null

  const searchQuery = ref(stored?.searchQuery ?? '')
  const selectedNamespace = ref<string[]>(stored ? [...stored.selectedNamespace] : [])
  const showSystemNamespaces = ref(stored?.showSystemNamespaces ?? false)

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
    watch(showSystemNamespaces, (val) => {
      filterStore.setFilter(storeKey, 'showSystemNamespaces', val)
    })
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

      // 3. System Namespaces filter
      const isSystem =
        item.namespace && ['kube-system', 'monitoring', 'logging'].includes(item.namespace)
      if (!showSystemNamespaces.value && isSystem && selectedNamespace.value.length === 0) {
        return false
      }

      return true
    })
  })

  return {
    searchQuery,
    selectedNamespace,
    showSystemNamespaces,
    filteredResources
  }
}
