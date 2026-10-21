import type { ClusterInfo } from '@/types/kubernetes'

/**
 * Returns the friendly display name for a cluster.
 * Prefers custom alias if set, otherwise falls back to the kubeconfig context name.
 */
export function getClusterDisplayName(cluster?: ClusterInfo | null): string {
  if (!cluster) return ''
  const trimmedAlias = cluster.alias?.trim()
  if (trimmedAlias) {
    return trimmedAlias
  }
  return cluster.name || ''
}

/**
 * Collects, deduplicates, and sorts all tags present across all clusters.
 */
export function getAllClusterTags(clusters: ClusterInfo[]): string[] {
  const tagSet = new Set<string>()
  for (const cluster of clusters) {
    if (Array.isArray(cluster.tags)) {
      for (const tag of cluster.tags) {
        const trimmed = tag.trim()
        if (trimmed) {
          tagSet.add(trimmed)
        }
      }
    }
  }
  return Array.from(tagSet).sort((a, b) => a.localeCompare(b))
}

/**
 * Filters clusters based on an optional search query and an optional selected tag.
 * The search query is matched case-insensitively against alias, name, description, and tags.
 */
export function filterClusters(
  clusters: ClusterInfo[],
  searchQuery = '',
  selectedTag?: string | null
): ClusterInfo[] {
  const normalizedQuery = searchQuery.trim().toLowerCase()
  const normalizedTag = selectedTag?.trim().toLowerCase()

  return clusters.filter((cluster) => {
    // 1. Tag filter matching
    if (normalizedTag) {
      const hasTag = cluster.tags?.some((t) => t.toLowerCase() === normalizedTag)
      if (!hasTag) return false
    }

    // 2. Query matching
    if (!normalizedQuery) return true

    const nameMatch = cluster.name.toLowerCase().includes(normalizedQuery)
    const aliasMatch = cluster.alias?.toLowerCase().includes(normalizedQuery) || false
    const descMatch = cluster.description?.toLowerCase().includes(normalizedQuery) || false
    const tagMatch = cluster.tags?.some((t) => t.toLowerCase().includes(normalizedQuery)) || false

    return nameMatch || aliasMatch || descMatch || tagMatch
  })
}
