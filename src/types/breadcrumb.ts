export type CategoryId =
  'clusters' | 'core' | 'workloads' | 'network' | 'storage' | 'config' | 'security' | 'logs'

export interface BreadcrumbItem {
  label: string
  route?: string
  icon?: string
}

export interface ResourceCategoryMetadata {
  kind: string
  categoryId: CategoryId
  categoryLabel: string
  categoryRoute: string
  typeLabel: string
  typeRoute: string
  isClusterScoped: boolean
}
