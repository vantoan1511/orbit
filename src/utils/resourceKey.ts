export interface IdentifiableResource {
  uid?: string
  name?: string
  namespace?: string
  id?: string
  objectName?: string
}

/**
 * Computes a unique composite key for a Kubernetes resource.
 * Prefers `uid` when present, otherwise defaults to `${namespace}/${name}` or `/${name}` for cluster-scoped resources.
 */
export function getResourceCompositeKey(item: IdentifiableResource): string {
  if (item.uid) {
    return item.uid
  }
  const name = item.name ?? item.objectName ?? item.id ?? ''
  if (item.namespace) {
    return `${item.namespace}/${name}`
  }
  return `/${name}`
}

/**
 * Determines whether the full table skeleton should be displayed.
 * Skeletons are only displayed on initial load when data is empty; subsequent refreshes preserve the mounted table.
 */
export function shouldShowSkeleton(loading: boolean, dataLength: number): boolean {
  return Boolean(loading && dataLength === 0)
}
