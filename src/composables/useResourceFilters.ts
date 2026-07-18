import { computed, ref, type Ref } from 'vue'

export interface ResourceItem {
  name: string
  namespace: string
  [key: string]: unknown
}

export function useResourceFilters<T extends ResourceItem>(
  resources: Ref<T[]>,
  searchFields: (keyof T)[] = ['name']
) {
  const searchQuery = ref('')
  const selectedNamespace = ref('All Namespaces')
  const showSystemNamespaces = ref(false)

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
        selectedNamespace.value !== 'All Namespaces' &&
        item.namespace !== selectedNamespace.value
      ) {
        return false
      }

      // 3. System Namespaces filter
      const isSystem = ['kube-system', 'monitoring', 'logging'].includes(item.namespace)
      if (!showSystemNamespaces.value && isSystem && selectedNamespace.value === 'All Namespaces') {
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
