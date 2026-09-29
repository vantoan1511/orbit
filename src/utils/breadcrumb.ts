import { KUBERNETES_RESOURCE_KIND } from '../constants/kubernetes.ts'
import type { BreadcrumbItem, ResourceCategoryMetadata } from '../types/breadcrumb.ts'

export const RESOURCE_BREADCRUMB_MAP: Record<string, ResourceCategoryMetadata> = {
  [KUBERNETES_RESOURCE_KIND.Deployment]: {
    kind: KUBERNETES_RESOURCE_KIND.Deployment,
    categoryId: 'workloads',
    categoryLabel: 'Workloads',
    categoryRoute: '/workloads',
    typeLabel: 'Deployments',
    typeRoute: '/workloads?tab=deployments',
    isClusterScoped: false
  },
  [KUBERNETES_RESOURCE_KIND.DaemonSet]: {
    kind: KUBERNETES_RESOURCE_KIND.DaemonSet,
    categoryId: 'workloads',
    categoryLabel: 'Workloads',
    categoryRoute: '/workloads',
    typeLabel: 'DaemonSets',
    typeRoute: '/workloads?tab=daemonsets',
    isClusterScoped: false
  },
  [KUBERNETES_RESOURCE_KIND.StatefulSet]: {
    kind: KUBERNETES_RESOURCE_KIND.StatefulSet,
    categoryId: 'workloads',
    categoryLabel: 'Workloads',
    categoryRoute: '/workloads',
    typeLabel: 'StatefulSets',
    typeRoute: '/workloads?tab=statefulsets',
    isClusterScoped: false
  },
  [KUBERNETES_RESOURCE_KIND.ReplicaSet]: {
    kind: KUBERNETES_RESOURCE_KIND.ReplicaSet,
    categoryId: 'workloads',
    categoryLabel: 'Workloads',
    categoryRoute: '/workloads',
    typeLabel: 'ReplicaSets',
    typeRoute: '/workloads?tab=replicasets',
    isClusterScoped: false
  },
  [KUBERNETES_RESOURCE_KIND.Job]: {
    kind: KUBERNETES_RESOURCE_KIND.Job,
    categoryId: 'workloads',
    categoryLabel: 'Workloads',
    categoryRoute: '/workloads',
    typeLabel: 'Jobs',
    typeRoute: '/workloads?tab=jobs',
    isClusterScoped: false
  },
  [KUBERNETES_RESOURCE_KIND.CronJob]: {
    kind: KUBERNETES_RESOURCE_KIND.CronJob,
    categoryId: 'workloads',
    categoryLabel: 'Workloads',
    categoryRoute: '/workloads',
    typeLabel: 'CronJobs',
    typeRoute: '/workloads?tab=cronjobs',
    isClusterScoped: false
  },
  [KUBERNETES_RESOURCE_KIND.Pod]: {
    kind: KUBERNETES_RESOURCE_KIND.Pod,
    categoryId: 'workloads',
    categoryLabel: 'Workloads',
    categoryRoute: '/workloads',
    typeLabel: 'Pods',
    typeRoute: '/workloads?tab=pods',
    isClusterScoped: false
  },
  [KUBERNETES_RESOURCE_KIND.Service]: {
    kind: KUBERNETES_RESOURCE_KIND.Service,
    categoryId: 'network',
    categoryLabel: 'Network',
    categoryRoute: '/network',
    typeLabel: 'Services',
    typeRoute: '/network?tab=services',
    isClusterScoped: false
  },
  [KUBERNETES_RESOURCE_KIND.Ingress]: {
    kind: KUBERNETES_RESOURCE_KIND.Ingress,
    categoryId: 'network',
    categoryLabel: 'Network',
    categoryRoute: '/network',
    typeLabel: 'Ingresses',
    typeRoute: '/network?tab=ingresses',
    isClusterScoped: false
  },
  [KUBERNETES_RESOURCE_KIND.ConfigMap]: {
    kind: KUBERNETES_RESOURCE_KIND.ConfigMap,
    categoryId: 'config',
    categoryLabel: 'ConfigMaps & Secrets',
    categoryRoute: '/config',
    typeLabel: 'ConfigMaps',
    typeRoute: '/config?tab=configmaps',
    isClusterScoped: false
  },
  [KUBERNETES_RESOURCE_KIND.Secret]: {
    kind: KUBERNETES_RESOURCE_KIND.Secret,
    categoryId: 'config',
    categoryLabel: 'ConfigMaps & Secrets',
    categoryRoute: '/config',
    typeLabel: 'Secrets',
    typeRoute: '/config?tab=secrets',
    isClusterScoped: false
  },
  [KUBERNETES_RESOURCE_KIND.PersistentVolumeClaim]: {
    kind: KUBERNETES_RESOURCE_KIND.PersistentVolumeClaim,
    categoryId: 'storage',
    categoryLabel: 'Storage',
    categoryRoute: '/storage',
    typeLabel: 'Volume Claims',
    typeRoute: '/storage?tab=pvcs',
    isClusterScoped: false
  },
  [KUBERNETES_RESOURCE_KIND.PersistentVolume]: {
    kind: KUBERNETES_RESOURCE_KIND.PersistentVolume,
    categoryId: 'storage',
    categoryLabel: 'Storage',
    categoryRoute: '/storage',
    typeLabel: 'PersistentVolumes',
    typeRoute: '/storage?tab=pvs',
    isClusterScoped: true
  },
  [KUBERNETES_RESOURCE_KIND.StorageClass]: {
    kind: KUBERNETES_RESOURCE_KIND.StorageClass,
    categoryId: 'storage',
    categoryLabel: 'Storage',
    categoryRoute: '/storage',
    typeLabel: 'StorageClasses',
    typeRoute: '/storage?tab=classes',
    isClusterScoped: true
  },
  [KUBERNETES_RESOURCE_KIND.Namespace]: {
    kind: KUBERNETES_RESOURCE_KIND.Namespace,
    categoryId: 'core',
    categoryLabel: 'Namespaces',
    categoryRoute: '/namespaces',
    typeLabel: 'Namespaces',
    typeRoute: '/namespaces',
    isClusterScoped: true
  },
  [KUBERNETES_RESOURCE_KIND.Node]: {
    kind: KUBERNETES_RESOURCE_KIND.Node,
    categoryId: 'core',
    categoryLabel: 'Nodes',
    categoryRoute: '/nodes',
    typeLabel: 'Nodes',
    typeRoute: '/nodes',
    isClusterScoped: true
  },
  [KUBERNETES_RESOURCE_KIND.Policy]: {
    kind: KUBERNETES_RESOURCE_KIND.Policy,
    categoryId: 'security',
    categoryLabel: 'Policies',
    categoryRoute: '/policies',
    typeLabel: 'Policies',
    typeRoute: '/policies',
    isClusterScoped: true
  }
}

/**
 * Derives the breadcrumb trail items for a Kubernetes resource in Edit view.
 */
export function getResourceBreadcrumbs(params: {
  kind?: string
  namespace?: string
  name?: string
}): BreadcrumbItem[] {
  const kind = params.kind || ''
  const meta: ResourceCategoryMetadata = RESOURCE_BREADCRUMB_MAP[kind] || {
    kind,
    categoryId: 'workloads',
    categoryLabel: 'Workloads',
    categoryRoute: '/workloads',
    typeLabel: kind,
    typeRoute: `/workloads?tab=${kind.toLowerCase()}s`,
    isClusterScoped: false
  }

  const breadcrumbs: BreadcrumbItem[] = []

  // 1. Root Category
  breadcrumbs.push({
    label: meta.categoryLabel,
    route: meta.categoryRoute
  })

  // 2. Resource Type / Tab (if distinct from root category)
  if (meta.categoryRoute !== meta.typeRoute) {
    breadcrumbs.push({
      label: meta.typeLabel,
      route: meta.typeRoute
    })
  }

  // 3. Namespace (only for namespaced resources)
  if (!meta.isClusterScoped && params.namespace && params.namespace !== '_cluster') {
    breadcrumbs.push({
      label: params.namespace
    })
  }

  // 4. Resource Name
  if (params.name) {
    breadcrumbs.push({
      label: params.name
    })
  }

  // 5. Terminal action (non-clickable current-page indicator)
  breadcrumbs.push({
    label: 'Edit'
  })

  return breadcrumbs
}
