import { KUBERNETES_ACTION } from '../constants/kubernetes.ts'
import type { KubernetesAction, NodeInfo, ServiceInfo } from '../types/kubernetes.ts'
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
 * Merges a single Node watch update with the existing Node record.
 * Preserves CPU, memory, and pod metric counts when the incoming watch payload
 * has zero/empty metrics (since watch events come from kube-apiserver without metrics server data).
 */
export function mergeNodeWatchUpdate(existing: NodeInfo, incoming: NodeInfo): NodeInfo {
  const hasIncomingMetrics =
    Boolean(incoming.cpuPct && incoming.cpuPct > 0) ||
    Boolean(incoming.cpuUsed && incoming.cpuUsed !== '0' && incoming.cpuUsed !== '0.00') ||
    Boolean(incoming.memPct && incoming.memPct > 0) ||
    Boolean(incoming.memUsed && incoming.memUsed !== '0' && incoming.memUsed !== '0.00') ||
    Boolean(incoming.podsCount && incoming.podsCount > 0)

  if (
    !hasIncomingMetrics &&
    (existing.cpuPct ||
      existing.cpuUsed ||
      existing.memPct ||
      existing.memUsed ||
      existing.podsCount)
  ) {
    return {
      ...incoming,
      cpuPct: existing.cpuPct,
      cpuUsed: existing.cpuUsed,
      memPct: existing.memPct,
      memUsed: existing.memUsed,
      podsCount: existing.podsCount
    }
  }

  return incoming
}

/**
 * Structural equality for plain JSON resource payloads.
 * Handles nested objects, order-sensitive arrays, and treats missing keys vs undefined as equal.
 */
export function resourcesEqual(a: unknown, b: unknown): boolean {
  if (a === b) return true
  if (typeof a !== 'object' || a === null || typeof b !== 'object' || b === null) {
    return false
  }

  const isArrA = Array.isArray(a)
  const isArrB = Array.isArray(b)
  if (isArrA !== isArrB) return false

  if (isArrA && isArrB) {
    const arrA = a as unknown[]
    const arrB = b as unknown[]
    if (arrA.length !== arrB.length) return false
    for (let i = 0; i < arrA.length; i++) {
      if (!resourcesEqual(arrA[i], arrB[i])) return false
    }
    return true
  }

  const objA = a as Record<string, unknown>
  const objB = b as Record<string, unknown>

  const keys = new Set<string>()
  for (const k in objA) {
    if (Object.prototype.hasOwnProperty.call(objA, k)) {
      keys.add(k)
    }
  }
  for (const k in objB) {
    if (Object.prototype.hasOwnProperty.call(objB, k)) {
      keys.add(k)
    }
  }

  for (const k of keys) {
    const valA = objA[k]
    const valB = objB[k]
    if (valA === undefined && valB === undefined) {
      continue
    }
    if (!resourcesEqual(valA, valB)) {
      return false
    }
  }

  return true
}

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
        const existing = (current ? current[existingIdx] : baseList[existingIdx]) as T | undefined
        const next = merge && existing !== undefined ? merge(existing, update.data) : update.data
        if (existing !== undefined && resourcesEqual(existing, next)) {
          continue
        }
        if (!current) current = [...baseList]
        current[existingIdx] = next
        changed = true
      } else {
        if (!current) current = [...baseList]
        indexMap.set(key, current.length)
        current.push(update.data)
        changed = true
      }
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
