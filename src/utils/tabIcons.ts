import type { Component } from 'vue'
import {
  Activity,
  Archive,
  Box,
  Boxes,
  Cable,
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
  Settings,
  Settings2,
  ShieldCheck
} from '@lucide/vue'

export interface TabMetadata {
  title: string
  iconName: string
  icon: Component
  iconColorClass: string
  category: string
}

export const ICON_MAP: Record<string, Component> = {
  Activity,
  Archive,
  Box,
  Boxes,
  Cable,
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
  Settings,
  Settings2,
  ShieldCheck
}

export function getTabMetadataForRoute(path: string, query?: Record<string, string>): TabMetadata {
  const tab = query?.tab

  if (path === '/' || path === '') {
    return {
      title: 'Overview',
      iconName: 'LayoutDashboard',
      icon: LayoutDashboard,
      iconColorClass: 'text-primary',
      category: 'core'
    }
  }

  if (path === '/nodes') {
    return {
      title: 'Nodes',
      iconName: 'Server',
      icon: Server,
      iconColorClass: 'text-node',
      category: 'core'
    }
  }

  if (path === '/namespaces') {
    return {
      title: 'Namespaces',
      iconName: 'FolderOpen',
      icon: FolderOpen,
      iconColorClass: 'text-primary',
      category: 'core'
    }
  }

  if (path === '/events') {
    return {
      title: 'Events',
      iconName: 'Activity',
      icon: Activity,
      iconColorClass: 'text-primary',
      category: 'core'
    }
  }

  if (path === '/pods' || (path === '/workloads' && tab === 'pods')) {
    return {
      title: 'Pods',
      iconName: 'Box',
      icon: Box,
      iconColorClass: 'text-pod',
      category: 'workloads'
    }
  }

  if (path === '/workloads') {
    switch (tab) {
      case 'overview':
        return {
          title: 'Overview',
          iconName: 'Boxes',
          icon: Boxes,
          iconColorClass: 'text-primary',
          category: 'workloads'
        }
      case 'pods':
        return {
          title: 'Pods',
          iconName: 'Box',
          icon: Box,
          iconColorClass: 'text-pod',
          category: 'workloads'
        }
      case 'statefulsets':
        return {
          title: 'StatefulSets',
          iconName: 'Database',
          icon: Database,
          iconColorClass: 'text-statefulset',
          category: 'workloads'
        }
      case 'daemonsets':
        return {
          title: 'DaemonSets',
          iconName: 'Ghost',
          icon: Ghost,
          iconColorClass: 'text-daemonset',
          category: 'workloads'
        }
      case 'replicasets':
        return {
          title: 'ReplicaSets',
          iconName: 'Copy',
          icon: Copy,
          iconColorClass: 'text-replicaset',
          category: 'workloads'
        }
      case 'jobs':
        return {
          title: 'Jobs',
          iconName: 'Hammer',
          icon: Hammer,
          iconColorClass: 'text-job',
          category: 'workloads'
        }
      case 'cronjobs':
        return {
          title: 'CronJobs',
          iconName: 'Clock',
          icon: Clock,
          iconColorClass: 'text-job',
          category: 'workloads'
        }
      case 'deployments':
      default:
        return {
          title: 'Deployments',
          iconName: 'Layers',
          icon: Layers,
          iconColorClass: 'text-deployment',
          category: 'workloads'
        }
    }
  }

  if (path === '/network') {
    switch (tab) {
      case 'ingresses':
        return {
          title: 'Ingresses',
          iconName: 'Globe',
          icon: Globe,
          iconColorClass: 'text-ingress',
          category: 'network'
        }
      case 'port-forward':
        return {
          title: 'Port Forward',
          iconName: 'Cable',
          icon: Cable,
          iconColorClass: 'text-primary',
          category: 'network'
        }
      case 'services':
      default:
        return {
          title: 'Services',
          iconName: 'Network',
          icon: Network,
          iconColorClass: 'text-service',
          category: 'network'
        }
    }
  }

  if (path === '/storage') {
    switch (tab) {
      case 'pvs':
        return {
          title: 'PersistentVolumes',
          iconName: 'Archive',
          icon: Archive,
          iconColorClass: 'text-primary',
          category: 'storage'
        }
      case 'pvcs':
        return {
          title: 'Volume Claims',
          iconName: 'FileDown',
          icon: FileDown,
          iconColorClass: 'text-primary',
          category: 'storage'
        }
      case 'classes':
        return {
          title: 'StorageClasses',
          iconName: 'Settings2',
          icon: Settings2,
          iconColorClass: 'text-primary',
          category: 'storage'
        }
      case 'overview':
      default:
        return {
          title: 'Storage',
          iconName: 'HardDrive',
          icon: HardDrive,
          iconColorClass: 'text-primary',
          category: 'storage'
        }
    }
  }

  if (path === '/config') {
    switch (tab) {
      case 'secrets':
        return {
          title: 'Secrets',
          iconName: 'KeyRound',
          icon: KeyRound,
          iconColorClass: 'text-secret',
          category: 'config'
        }
      case 'configmaps':
      default:
        return {
          title: 'ConfigMaps',
          iconName: 'FileJson',
          icon: FileJson,
          iconColorClass: 'text-configmap',
          category: 'config'
        }
    }
  }

  if (path === '/policies') {
    return {
      title: 'Policies',
      iconName: 'ShieldCheck',
      icon: ShieldCheck,
      iconColorClass: 'text-primary',
      category: 'security'
    }
  }

  if (path === '/logs') {
    let title = 'Logs'
    if (query?.workload && query.workload.trim() && query.workload !== 'All') {
      title = query.workload.trim()
    } else if (query?.pod && query.pod.trim() && query.pod !== 'All') {
      title = query.pod.trim()
    }
    return {
      title,
      iconName: 'FileText',
      icon: FileText,
      iconColorClass: 'text-primary',
      category: 'logs'
    }
  }

  if (path === '/settings') {
    return {
      title: 'Settings',
      iconName: 'Settings',
      icon: Settings,
      iconColorClass: 'text-primary',
      category: 'settings'
    }
  }

  // Fallback for custom or unknown paths
  const cleanTitle =
    path
      .replace(/^\//, '')
      .split(/[-/]/)
      .filter(Boolean)
      .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
      .join(' ') || 'Overview'

  return {
    title: cleanTitle,
    iconName: 'LayoutDashboard',
    icon: LayoutDashboard,
    iconColorClass: 'text-primary',
    category: 'core'
  }
}
