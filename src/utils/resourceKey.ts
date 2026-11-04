export interface IdentifiableResource {
  uid?: string
  name?: string
  namespace?: string
  id?: string
  objectName?: string
}

const compositeKeyCache = new WeakMap<object, string>()

/**
 * Computes a unique composite key for a Kubernetes resource.
 * Prefers `uid` when present, otherwise defaults to `${namespace}/${name}` or `/${name}` for cluster-scoped resources.
 * Results are memoised by object identity in a WeakMap for O(1) repeated evaluation.
 */
export function getResourceCompositeKey(item: IdentifiableResource): string {
  if (typeof item === 'object' && item !== null) {
    const cached = compositeKeyCache.get(item)
    if (cached !== undefined) {
      return cached
    }
  }

  let key: string
  if (item.uid) {
    key = item.uid
  } else {
    const name = item.name ?? item.objectName ?? item.id ?? ''
    if (item.namespace) {
      key = `${item.namespace}/${name}`
    } else {
      key = `/${name}`
    }
  }

  if (typeof item === 'object' && item !== null) {
    compositeKeyCache.set(item, key)
  }

  return key
}

/**
 * Determines whether the full table skeleton should be displayed.
 * Skeletons are only displayed on initial load when data is empty; subsequent refreshes preserve the mounted table.
 */
export function shouldShowSkeleton(loading: boolean, dataLength: number): boolean {
  return Boolean(loading && dataLength === 0)
}
