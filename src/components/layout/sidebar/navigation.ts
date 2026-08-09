import {
  Activity,
  Archive,
  Box,
  Boxes,
  Clock,
  Copy,
  Database,
  FileDown,
  FileJson,
  FileText,
  FolderOpen,
  Ghost,
  Globe,
  Hammer,
  HardDrive,
  KeyRound,
  Layers,
  LayoutDashboard,
  Network,
  Server,
  Settings2,
  ShieldCheck
} from '@lucide/vue'
import type { MenuItem } from 'primevue/menuitem'
import type { Component } from 'vue'

export type CategoryId =
  'clusters' | 'core' | 'workloads' | 'network' | 'storage' | 'config' | 'security' | 'logs'

export interface SidebarCategory {
  id: CategoryId
  name: string
  icon: Component
  defaultPath: string | null
}

export const categories: SidebarCategory[] = [
  { id: 'clusters', name: 'Clusters', icon: Server, defaultPath: null },
  { id: 'core', name: 'Overview & Core', icon: LayoutDashboard, defaultPath: '/' },
  {
    id: 'workloads',
    name: 'Workloads',
    icon: Boxes,
    defaultPath: '/workloads?tab=deployments'
  },
  { id: 'network', name: 'Network', icon: Network, defaultPath: '/network?tab=services' },
  {
    id: 'storage',
    name: 'Storage',
    icon: HardDrive,
    defaultPath: '/storage?tab=overview'
  },
  {
    id: 'config',
    name: 'Config & Secrets',
    icon: Settings2,
    defaultPath: '/config?tab=configmaps'
  },
  { id: 'security', name: 'Security', icon: ShieldCheck, defaultPath: '/policies' },
  { id: 'logs', name: 'Logs', icon: FileText, defaultPath: '/logs' }
]

export interface AppSidebarMenuItem extends Omit<MenuItem, 'icon'> {
  label?: string
  customIcon?: Component
  route?: string
  items?: AppSidebarMenuItem[]
}

export const categoryNavItems: Record<Exclude<CategoryId, 'clusters'>, AppSidebarMenuItem[]> = {
  core: [
    { key: 'core-overview', label: 'Overview', customIcon: LayoutDashboard, route: '/' },
    { key: 'core-nodes', label: 'Nodes', customIcon: Server, route: '/nodes' },
    { key: 'core-namespaces', label: 'Namespaces', customIcon: FolderOpen, route: '/namespaces' },
    { key: 'core-events', label: 'Events', customIcon: Activity, route: '/events' }
  ],
  workloads: [
    {
      key: 'workloads-overview',
      label: 'Overview',
      customIcon: Boxes,
      route: '/workloads?tab=overview'
    },
    { key: 'workloads-pods', label: 'Pods', customIcon: Box, route: '/pods' },
    {
      key: 'workloads-deployments',
      label: 'Deployments',
      customIcon: Layers,
      route: '/workloads?tab=deployments'
    },
    {
      key: 'workloads-statefulsets',
      label: 'StatefulSets',
      customIcon: Database,
      route: '/workloads?tab=statefulsets'
    },
    {
      key: 'workloads-daemonsets',
      label: 'DaemonSets',
      customIcon: Ghost,
      route: '/workloads?tab=daemonsets'
    },
    {
      key: 'workloads-replicasets',
      label: 'ReplicaSets',
      customIcon: Copy,
      route: '/workloads?tab=replicasets'
    },
    { key: 'workloads-jobs', label: 'Jobs', customIcon: Hammer, route: '/workloads?tab=jobs' },
    {
      key: 'workloads-cronjobs',
      label: 'CronJobs',
      customIcon: Clock,
      route: '/workloads?tab=cronjobs'
    }
  ],
  network: [
    {
      key: 'network-services',
      label: 'Services',
      customIcon: Network,
      route: '/network?tab=services'
    },
    {
      key: 'network-ingresses',
      label: 'Ingresses',
      customIcon: Globe,
      route: '/network?tab=ingresses'
    }
  ],
  storage: [
    {
      key: 'storage-overview',
      label: 'Overview',
      customIcon: HardDrive,
      route: '/storage?tab=overview'
    },
    {
      key: 'storage-pvs',
      label: 'PersistentVolumes',
      customIcon: Archive,
      route: '/storage?tab=pvs'
    },
    {
      key: 'storage-pvcs',
      label: 'Volume Claims',
      customIcon: FileDown,
      route: '/storage?tab=pvcs'
    },
    {
      key: 'storage-classes',
      label: 'StorageClasses',
      customIcon: Settings2,
      route: '/storage?tab=classes'
    }
  ],
  config: [
    {
      key: 'config-configmaps',
      label: 'ConfigMaps',
      customIcon: FileJson,
      route: '/config?tab=configmaps'
    },
    {
      key: 'config-secrets',
      label: 'Secrets',
      customIcon: KeyRound,
      route: '/config?tab=secrets'
    }
  ],
  security: [
    { key: 'security-policies', label: 'Policies', customIcon: ShieldCheck, route: '/policies' }
  ],
  logs: [{ key: 'logs-main', label: 'Logs', customIcon: FileText, route: '/logs' }]
}
