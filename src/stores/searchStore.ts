import { defineStore } from 'pinia'
import { computed, ref, type Component } from 'vue'
import { useKubernetesStore } from './kubernetesStore.ts'
import { useLogsStore } from './logsStore.ts'
import { useTableFilterStore } from './tableFilterStore.ts'
import { categoryNavItems } from '../components/layout/sidebar/navigation.ts'
import { KUBERNETES_RESOURCE_KIND } from '../constants/kubernetes.ts'
import type { SearchCategory, SearchCategoryCounts, SearchResultItem } from '../types/search.ts'
import {
  Archive,
  Box,
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
  KeyRound,
  Layers,
  LayoutDashboard,
  Network,
  Server,
  Settings2,
  ShieldCheck
} from '@lucide/vue'

export interface NavigationTarget {
  path: string
  query?: Record<string, string>
}

export type NavigationHandler = (target: NavigationTarget) => void

let fallbackNavigationHandler: NavigationHandler | null = null

export function setNavigationHandler(handler: NavigationHandler | null) {
  fallbackNavigationHandler = handler
}

/**
 * Calculates a relevance score for a search item based on title, subtitle, namespace, and kind.
 */
export function scoreSearchItem(
  query: string,
  title: string,
  subtitle?: string,
  namespace?: string,
  kind?: string
): number {
  const q = query.trim().toLowerCase()
  if (!q) return 0

  const t = title.toLowerCase()
  if (t === q) return 100
  if (t.startsWith(q)) return 80
  if (t.includes(q)) return 50

  const sub = subtitle?.toLowerCase() ?? ''
  if (sub.includes(q)) return 30

  const ns = namespace?.toLowerCase() ?? ''
  if (ns.includes(q)) return 30

  const k = kind?.toLowerCase() ?? ''
  if (k.includes(q)) return 30

  return 0
}

export const useSearchStore = defineStore('search', () => {
  const isOpen = ref<boolean>(false)
  const searchQuery = ref<string>('')
  const activeCategory = ref<SearchCategory>('all')
  const selectedIndex = ref<number>(0)
  const navigationHandler = ref<NavigationHandler | null>(null)

  const k8sStore = useKubernetesStore()
  const logsStore = useLogsStore()
  const tableFilterStore = useTableFilterStore()

  function setNavigationHandler(handler: NavigationHandler | null) {
    navigationHandler.value = handler
  }

  function open() {
    isOpen.value = true
    searchQuery.value = ''
    selectedIndex.value = 0
  }

  function close() {
    isOpen.value = false
  }

  function toggle() {
    if (isOpen.value) {
      close()
    } else {
      open()
    }
  }

  function setCategory(cat: SearchCategory) {
    activeCategory.value = cat
    selectedIndex.value = 0
  }

  function navigateTo(target: NavigationTarget) {
    if (navigationHandler.value) {
      navigationHandler.value(target)
    } else if (fallbackNavigationHandler) {
      fallbackNavigationHandler(target)
    }
  }

  function parseRoute(route: string): NavigationTarget {
    const [pathPart, queryPart] = route.split('?')
    const query: Record<string, string> = {}
    if (queryPart) {
      const searchParams = new URLSearchParams(queryPart)
      searchParams.forEach((val, key) => {
        query[key] = val
      })
    }
    return { path: pathPart || '/', query }
  }

  function jumpToResource(
    storeKey: string,
    name: string,
    namespace?: string,
    path?: string,
    query?: Record<string, string>
  ) {
    try {
      tableFilterStore.setFilter(storeKey, 'searchQuery', name)
      if (namespace) {
        tableFilterStore.setFilter(storeKey, 'selectedNamespace', [namespace])
      }
    } catch {
      // Ignored if table filter store is not ready
    }
    if (path) {
      navigateTo({ path, query })
    }
  }

  function openLog(
    namespace: string,
    kind: string,
    workload: string,
    pod: string = 'All',
    container: string = 'All'
  ) {
    navigateTo({
      path: '/logs',
      query: {
        namespace,
        kind,
        workload,
        pod,
        container
      }
    })
  }

  // Generate Navigation Items
  const navigationItems = computed<SearchResultItem[]>(() => {
    const items: SearchResultItem[] = []
    for (const catItems of Object.values(categoryNavItems)) {
      for (const item of catItems) {
        if (!item.label) continue
        const route = item.route || '/'
        items.push({
          id: `nav-${item.key || route}`,
          title: item.label,
          subtitle: 'Navigation',
          category: 'navigation',
          icon: (item.customIcon as Component) || LayoutDashboard,
          iconColorClass: 'text-primary',
          action: () => {
            navigateTo(parseRoute(route))
          }
        })
      }
    }
    // Also add Settings
    items.push({
      id: 'nav-settings',
      title: 'Settings',
      subtitle: 'Navigation',
      category: 'navigation',
      icon: Settings2,
      iconColorClass: 'text-primary',
      action: () => {
        navigateTo({ path: '/settings' })
      }
    })
    return items
  })

  // Generate Kubernetes Resource Items
  const resourceItems = computed<SearchResultItem[]>(() => {
    if (!isOpen.value) return []
    const items: SearchResultItem[] = []

    // Pods
    for (const p of k8sStore.pods) {
      items.push({
        id: `pod-${p.namespace}/${p.name}`,
        title: p.name,
        subtitle: `${p.namespace} • Pod • ${p.status}`,
        category: 'resources',
        kind: 'Pod',
        namespace: p.namespace,
        status: p.status,
        icon: Box,
        iconColorClass: 'text-pod',
        action: () => jumpToResource('pod', p.name, p.namespace, '/workloads', { tab: 'pods' })
      })
    }

    // Deployments
    for (const d of k8sStore.deployments) {
      const replicasText = d.replicas ? `${d.replicas.current}/${d.replicas.desired} Replicas` : ''
      items.push({
        id: `dep-${d.namespace}/${d.name}`,
        title: d.name,
        subtitle: `${d.namespace} • Deployment • ${replicasText}`,
        category: 'resources',
        kind: 'Deployment',
        namespace: d.namespace,
        status: d.status,
        icon: Layers,
        iconColorClass: 'text-deployment',
        action: () =>
          jumpToResource('deployment', d.name, d.namespace, '/workloads', { tab: 'deployments' })
      })
    }

    // StatefulSets
    for (const s of k8sStore.statefulSets) {
      const replicasText = s.replicas ? `${s.replicas.current}/${s.replicas.desired} Replicas` : ''
      items.push({
        id: `sts-${s.namespace}/${s.name}`,
        title: s.name,
        subtitle: `${s.namespace} • StatefulSet • ${replicasText}`,
        category: 'resources',
        kind: 'StatefulSet',
        namespace: s.namespace,
        status: s.status,
        icon: Database,
        iconColorClass: 'text-statefulset',
        action: () =>
          jumpToResource('statefulset', s.name, s.namespace, '/workloads', {
            tab: 'statefulsets'
          })
      })
    }

    // DaemonSets
    for (const d of k8sStore.daemonSets) {
      const replicasText = d.replicas ? `${d.replicas.current}/${d.replicas.desired} Replicas` : ''
      items.push({
        id: `ds-${d.namespace}/${d.name}`,
        title: d.name,
        subtitle: `${d.namespace} • DaemonSet • ${replicasText}`,
        category: 'resources',
        kind: 'DaemonSet',
        namespace: d.namespace,
        status: d.status,
        icon: Ghost,
        iconColorClass: 'text-daemonset',
        action: () =>
          jumpToResource('daemonset', d.name, d.namespace, '/workloads', { tab: 'daemonsets' })
      })
    }

    // ReplicaSets
    for (const r of k8sStore.replicaSets) {
      const replicasText = r.replicas ? `${r.replicas.current}/${r.replicas.desired} Replicas` : ''
      items.push({
        id: `rs-${r.namespace}/${r.name}`,
        title: r.name,
        subtitle: `${r.namespace} • ReplicaSet • ${replicasText}`,
        category: 'resources',
        kind: 'ReplicaSet',
        namespace: r.namespace,
        status: r.status,
        icon: Copy,
        iconColorClass: 'text-replicaset',
        action: () =>
          jumpToResource('replicaset', r.name, r.namespace, '/workloads', { tab: 'replicasets' })
      })
    }

    // Jobs
    for (const j of k8sStore.jobs) {
      items.push({
        id: `job-${j.namespace}/${j.name}`,
        title: j.name,
        subtitle: `${j.namespace} • Job • ${j.status}`,
        category: 'resources',
        kind: 'Job',
        namespace: j.namespace,
        status: j.status,
        icon: Hammer,
        iconColorClass: 'text-job',
        action: () => jumpToResource('job', j.name, j.namespace, '/workloads', { tab: 'jobs' })
      })
    }

    // CronJobs
    for (const c of k8sStore.cronJobs) {
      items.push({
        id: `cj-${c.namespace}/${c.name}`,
        title: c.name,
        subtitle: `${c.namespace} • CronJob • ${c.schedule}`,
        category: 'resources',
        kind: 'CronJob',
        namespace: c.namespace,
        status: c.suspend ? 'Suspended' : 'Active',
        icon: Clock,
        iconColorClass: 'text-job',
        action: () =>
          jumpToResource('cronjob', c.name, c.namespace, '/workloads', { tab: 'cronjobs' })
      })
    }

    // Nodes
    for (const n of k8sStore.nodes) {
      items.push({
        id: `node-${n.name}`,
        title: n.name,
        subtitle: `Node • ${n.role} • ${n.status}`,
        category: 'resources',
        kind: 'Node',
        status: n.status,
        icon: Server,
        iconColorClass: 'text-node',
        action: () => jumpToResource('node', n.name, undefined, '/nodes')
      })
    }

    // Services
    for (const s of k8sStore.services) {
      items.push({
        id: `svc-${s.namespace}/${s.name}`,
        title: s.name,
        subtitle: `${s.namespace} • Service • ${s.type}`,
        category: 'resources',
        kind: 'Service',
        namespace: s.namespace,
        icon: Network,
        iconColorClass: 'text-service',
        action: () =>
          jumpToResource('service', s.name, s.namespace, '/network', { tab: 'services' })
      })
    }

    // Ingresses
    for (const i of k8sStore.ingresses) {
      items.push({
        id: `ing-${i.namespace}/${i.name}`,
        title: i.name,
        subtitle: `${i.namespace} • Ingress • ${i.hosts || 'No host'}`,
        category: 'resources',
        kind: 'Ingress',
        namespace: i.namespace,
        icon: Globe,
        iconColorClass: 'text-ingress',
        action: () =>
          jumpToResource('ingress', i.name, i.namespace, '/network', { tab: 'ingresses' })
      })
    }

    // ConfigMaps
    for (const cm of k8sStore.configMaps) {
      items.push({
        id: `cm-${cm.namespace}/${cm.name}`,
        title: cm.name,
        subtitle: `${cm.namespace} • ConfigMap • ${cm.keysCount} keys`,
        category: 'resources',
        kind: 'ConfigMap',
        namespace: cm.namespace,
        icon: FileJson,
        iconColorClass: 'text-configmap',
        action: () =>
          jumpToResource('configmap', cm.name, cm.namespace, '/config', { tab: 'configmaps' })
      })
    }

    // Secrets
    for (const sec of k8sStore.secrets) {
      items.push({
        id: `sec-${sec.namespace}/${sec.name}`,
        title: sec.name,
        subtitle: `${sec.namespace} • Secret • ${sec.type}`,
        category: 'resources',
        kind: 'Secret',
        namespace: sec.namespace,
        icon: KeyRound,
        iconColorClass: 'text-secret',
        action: () =>
          jumpToResource('secret', sec.name, sec.namespace, '/config', { tab: 'secrets' })
      })
    }

    // PersistentVolumes
    for (const pv of k8sStore.persistentVolumes) {
      items.push({
        id: `pv-${pv.name}`,
        title: pv.name,
        subtitle: `PersistentVolume • ${pv.capacity} • ${pv.status}`,
        category: 'resources',
        kind: 'PersistentVolume',
        status: pv.status,
        icon: Archive,
        iconColorClass: 'text-primary',
        action: () =>
          jumpToResource('persistentvolume', pv.name, undefined, '/storage', { tab: 'pvs' })
      })
    }

    // PersistentVolumeClaims
    for (const pvc of k8sStore.persistentVolumeClaims) {
      items.push({
        id: `pvc-${pvc.namespace}/${pvc.name}`,
        title: pvc.name,
        subtitle: `${pvc.namespace} • VolumeClaim • ${pvc.capacity}`,
        category: 'resources',
        kind: 'PersistentVolumeClaim',
        namespace: pvc.namespace,
        status: pvc.status,
        icon: FileDown,
        iconColorClass: 'text-primary',
        action: () =>
          jumpToResource('persistentvolumeclaim', pvc.name, pvc.namespace, '/storage', {
            tab: 'pvcs'
          })
      })
    }

    // StorageClasses
    for (const sc of k8sStore.storageClasses) {
      items.push({
        id: `sc-${sc.name}`,
        title: sc.name,
        subtitle: `StorageClass • ${sc.provisioner}`,
        category: 'resources',
        kind: 'StorageClass',
        icon: Settings2,
        iconColorClass: 'text-primary',
        action: () =>
          jumpToResource('storageclass', sc.name, undefined, '/storage', { tab: 'classes' })
      })
    }

    // Namespaces
    for (const ns of k8sStore.namespaces) {
      if (ns === 'All Namespaces') continue
      items.push({
        id: `ns-${ns}`,
        title: ns,
        subtitle: 'Namespace',
        category: 'resources',
        kind: 'Namespace',
        icon: FolderOpen,
        iconColorClass: 'text-primary',
        action: () => jumpToResource('namespace', ns, undefined, '/namespaces')
      })
    }

    // Policies
    for (const pol of k8sStore.policies) {
      items.push({
        id: `pol-${pol.namespace}/${pol.name}`,
        title: pol.name,
        subtitle: `${pol.namespace} • NetworkPolicy`,
        category: 'resources',
        kind: 'NetworkPolicy',
        namespace: pol.namespace,
        icon: ShieldCheck,
        iconColorClass: 'text-primary',
        action: () => jumpToResource('policy', pol.name, pol.namespace, '/policies')
      })
    }

    return items
  })

  // Generate Logs Items
  const logsItems = computed<SearchResultItem[]>(() => {
    if (!isOpen.value) return []
    const items: SearchResultItem[] = []

    // Recent Logs from logsStore
    for (const log of logsStore.recentLogs) {
      items.push({
        id: `recent-log-${log.namespace}/${log.workloadKind}/${log.workloadName}`,
        title: log.workloadName,
        subtitle: `${log.namespace} • Recent Log • ${log.workloadKind}`,
        category: 'logs',
        kind: log.workloadKind,
        namespace: log.namespace,
        icon: Clock,
        iconColorClass: 'text-muted-color',
        action: () =>
          openLog(log.namespace, log.workloadKind, log.workloadName, log.pod, log.container)
      })
    }

    // Workloads that can be viewed in logs
    const addWorkloadLog = (name: string, namespace: string, kind: string) => {
      // Avoid duplicate with recent logs
      const isAlreadyRecent = logsStore.recentLogs.some(
        (r) => r.workloadName === name && r.namespace === namespace && r.workloadKind === kind
      )
      if (isAlreadyRecent) return

      items.push({
        id: `log-target-${namespace}/${kind}/${name}`,
        title: name,
        subtitle: `${namespace} • Logs • ${kind}`,
        category: 'logs',
        kind,
        namespace,
        icon: FileText,
        iconColorClass: 'text-primary',
        action: () => openLog(namespace, kind, name)
      })
    }

    for (const d of k8sStore.deployments)
      addWorkloadLog(d.name, d.namespace, KUBERNETES_RESOURCE_KIND.Deployment)
    for (const s of k8sStore.statefulSets)
      addWorkloadLog(s.name, s.namespace, KUBERNETES_RESOURCE_KIND.StatefulSet)
    for (const d of k8sStore.daemonSets)
      addWorkloadLog(d.name, d.namespace, KUBERNETES_RESOURCE_KIND.DaemonSet)
    for (const r of k8sStore.replicaSets)
      addWorkloadLog(r.name, r.namespace, KUBERNETES_RESOURCE_KIND.ReplicaSet)
    for (const j of k8sStore.jobs) addWorkloadLog(j.name, j.namespace, KUBERNETES_RESOURCE_KIND.Job)
    for (const c of k8sStore.cronJobs)
      addWorkloadLog(c.name, c.namespace, KUBERNETES_RESOURCE_KIND.CronJob)
    for (const p of k8sStore.pods) addWorkloadLog(p.name, p.namespace, KUBERNETES_RESOURCE_KIND.Pod)

    return items
  })

  // All Candidate Items Combined
  const allCandidates = computed<SearchResultItem[]>(() => {
    if (!isOpen.value) return []
    return [...resourceItems.value, ...logsItems.value, ...navigationItems.value]
  })

  // Filtered & Ranked Results
  const filteredResults = computed<SearchResultItem[]>(() => {
    if (!isOpen.value) return []
    const query = searchQuery.value.trim()
    const category = activeCategory.value

    // When query is empty: show quick picks
    if (!query) {
      if (category === 'logs') {
        return logsItems.value.slice(0, 30)
      }
      if (category === 'resources') {
        return resourceItems.value.slice(0, 30)
      }
      if (category === 'navigation') {
        return navigationItems.value.slice(0, 30)
      }
      // 'all': Show recent logs + main navigation entries + sample resources
      const recent = logsItems.value.slice(0, 6)
      const nav = navigationItems.value.slice(0, 8)
      const res = resourceItems.value.slice(0, 10)
      return [...recent, ...nav, ...res]
    }

    // Scored list
    const scored: Array<{ item: SearchResultItem; score: number }> = []

    for (const item of allCandidates.value) {
      if (category !== 'all' && item.category !== category) {
        continue
      }
      const score = scoreSearchItem(query, item.title, item.subtitle, item.namespace, item.kind)
      if (score > 0) {
        scored.push({ item: { ...item, score }, score })
      }
    }

    // Sort descending by score, then alphabetically
    scored.sort((a, b) => {
      if (b.score !== a.score) return b.score - a.score
      return a.item.title.localeCompare(b.item.title)
    })

    const limit = category === 'all' ? 30 : 50
    return scored.slice(0, limit).map((s) => s.item)
  })

  // Category counts matching current query
  const categoryCounts = computed<SearchCategoryCounts>(() => {
    if (!isOpen.value) {
      return {
        all: 0,
        resources: 0,
        logs: 0,
        navigation: 0
      }
    }

    const query = searchQuery.value.trim()
    if (!query) {
      return {
        all: allCandidates.value.length,
        resources: resourceItems.value.length,
        logs: logsItems.value.length,
        navigation: navigationItems.value.length
      }
    }

    let resources = 0
    let logs = 0
    let navigation = 0

    for (const item of allCandidates.value) {
      const score = scoreSearchItem(query, item.title, item.subtitle, item.namespace, item.kind)
      if (score > 0) {
        if (item.category === 'resources') resources++
        else if (item.category === 'logs') logs++
        else if (item.category === 'navigation') navigation++
      }
    }

    return {
      all: resources + logs + navigation,
      resources,
      logs,
      navigation
    }
  })

  function selectNext() {
    const count = filteredResults.value.length
    if (count === 0) {
      selectedIndex.value = 0
      return
    }
    selectedIndex.value = (selectedIndex.value + 1) % count
  }

  function selectPrev() {
    const count = filteredResults.value.length
    if (count === 0) {
      selectedIndex.value = 0
      return
    }
    selectedIndex.value = (selectedIndex.value - 1 + count) % count
  }

  function executeSelected() {
    const results = filteredResults.value
    if (results.length > 0) {
      const selected = results[selectedIndex.value]
      if (selected) {
        close()
        void selected.action()
      }
    }
  }

  return {
    isOpen,
    searchQuery,
    activeCategory,
    selectedIndex,
    filteredResults,
    categoryCounts,
    open,
    close,
    toggle,
    setCategory,
    setNavigationHandler,
    selectNext,
    selectPrev,
    executeSelected
  }
})
