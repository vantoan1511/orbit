import { KUBERNETES_ACTION } from '../constants/kubernetes.ts'
import type { KubernetesAction, ServiceInfo } from '../types/kubernetes.ts'
import type { ShallowRef } from 'vue'

export const MAX_EVENTS_RETAINED = 1000

export type ResourceKeyExtractor<T> = (item: T) => string

export const keyNamespaced: ResourceKeyExtractor<{ name?: string; namespace?: string }> = (item) =>
  `${item.namespace ?? ''}/${item.name ?? ''}`

export const keyByName: ResourceKeyExtractor<{ name?: string }> = (item) => item.name ?? ''

export const keyByUid: ResourceKeyExtractor<{ uid?: string }> = (item) => item.uid ?? ''

export const keyService: ResourceKeyExtractor<ServiceInfo> = (item) =>
  item.uid || `${item.namespace ?? ''}/${item.name ?? ''}`

/**
 * Applies a batch of Kubernetes resource updates (Applied / Deleted) to a shallowRef array in O(N + M) time.
 */
export function updateResourceBatch<T>(
  listRef: ShallowRef<T[]>,
  updates: Array<{ action: KubernetesAction; data: T }>,
  getKey: ResourceKeyExtractor<T>,
  merge?: (existing: T, incoming: T) => T,
  maxCapacity?: number
) {
  if (!updates || updates.length === 0) return

  const baseList = listRef.value
  const indexMap = new Map<string, number>()
  for (let i = 0; i < baseList.length; i++) {
    const item = baseList[i]
    if (item !== undefined) {
      indexMap.set(getKey(item), i)
    }
  }

  let current: T[] | null = null
  let changed = false
  const deletedKeys = new Set<string>()

  for (const update of updates) {
    const key = getKey(update.data)
    if (!key) continue

    if (update.action === KUBERNETES_ACTION.Applied) {
      if (deletedKeys.has(key)) {
        deletedKeys.delete(key)
      }
      const existingIdx = indexMap.get(key)
      if (existingIdx !== undefined) {
        if (!current) current = [...baseList]
        const existing = current[existingIdx]
        current[existingIdx] =
          merge && existing !== undefined ? merge(existing, update.data) : update.data
      } else {
        if (!current) current = [...baseList]
        indexMap.set(key, current.length)
        current.push(update.data)
      }
      changed = true
    } else if (update.action === KUBERNETES_ACTION.Deleted) {
      if (indexMap.has(key)) {
        deletedKeys.add(key)
        changed = true
      }
    }
  }

  if (deletedKeys.size > 0) {
    if (!current) current = [...baseList]
    current = current.filter((item) => !deletedKeys.has(getKey(item)))
  }

  if (changed && current) {
    if (maxCapacity && current.length > maxCapacity) {
      current.splice(0, current.length - maxCapacity)
    }
    listRef.value = current
  }
}
