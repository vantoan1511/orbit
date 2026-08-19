import { kubernetesService } from '@/services/kubernetesService'
import { events as nativeEvents } from '@/services/nativeService'
import { OrbitEvents } from '@/types/events'
import type {
  ClusterInfo,
  ConfigMapInfo,
  CronJobInfo,
  DaemonSetInfo,
  DeploymentInfo,
  EventInfo,
  IngressInfo,
  JobInfo,
  NamespaceInfo,
  NodeInfo,
  PersistentVolumeClaimInfo,
  PersistentVolumeInfo,
  PodInfo,
  PolicyInfo,
  ReplicaSetInfo,
  SecretInfo,
  ServiceInfo,
  StatefulSetInfo,
  StorageClassInfo
} from '@/types/kubernetes'
import { useTableFilterStore } from '@/stores/tableFilterStore'
import { defineStore } from 'pinia'
import { computed, onScopeDispose, ref, watch } from 'vue'

// Helpers for Kubernetes quantity strings emitted by the Metrics Server.
function parseCpuToCores(q: string): number {
  if (q.endsWith('m')) return parseFloat(q) / 1000
  if (q.endsWith('n')) return parseFloat(q) / 1_000_000_000
  if (q.endsWith('u')) return parseFloat(q) / 1_000_000
  return parseFloat(q) || 0
}

function parseMemToGiB(q: string): number {
  if (q.endsWith('Ki')) return parseFloat(q) / (1024 * 1024)
  if (q.endsWith('Mi')) return parseFloat(q) / 1024
  if (q.endsWith('Gi')) return parseFloat(q)
  if (q.endsWith('Ti')) return parseFloat(q) * 1024
  return 0
}

export const useKubernetesStore = defineStore('kubernetes', () => {
  const isEngineReady = ref(false)
  const isAppLoading = ref(true)
  const pods = ref<PodInfo[]>([])
  const deployments = ref<DeploymentInfo[]>([])
  const statefulSets = ref<StatefulSetInfo[]>([])
  const daemonSets = ref<DaemonSetInfo[]>([])
  const replicaSets = ref<ReplicaSetInfo[]>([])
  const jobs = ref<JobInfo[]>([])
  const cronJobs = ref<CronJobInfo[]>([])
  const nodes = ref<NodeInfo[]>([])
  const services = ref<ServiceInfo[]>([])
  const ingresses = ref<IngressInfo[]>([])
  const configMaps = ref<ConfigMapInfo[]>([])
  const secrets = ref<SecretInfo[]>([])
  const events = ref<EventInfo[]>([])
  const policies = ref<PolicyInfo[]>([])
  const configMapsLoading = ref(false)
  const secretsLoading = ref(false)
  const eventsLoading = ref(false)
  const policiesLoading = ref(false)
  const podsLoading = ref(false)
  const deploymentsLoading = ref(false)
  const statefulSetsLoading = ref(false)
  const daemonSetsLoading = ref(false)
  const replicaSetsLoading = ref(false)
  const jobsLoading = ref(false)
  const cronJobsLoading = ref(false)
  const nodesLoading = ref(false)
  const servicesLoading = ref(false)
  const ingressesLoading = ref(false)
  const namespacesLoading = ref(false)
  const persistentVolumes = ref<PersistentVolumeInfo[]>([])
  const persistentVolumeClaims = ref<PersistentVolumeClaimInfo[]>([])
  const storageClasses = ref<StorageClassInfo[]>([])

  interface NamespacedResource {
    name: string
    namespace: string
  }

  interface ClusterScopedResource {
    name?: string
    uid?: string
  }
  const persistentVolumesLoading = ref(false)
  const persistentVolumeClaimsLoading = ref(false)
  const storageClassesLoading = ref(false)
  const namespaceList = ref<NamespaceInfo[]>([])
  const clusters = ref<ClusterInfo[]>([])
  const activeClusterId = ref<string | null>(null)

  const cpuHistory = ref<number[]>([0, 0, 0, 0, 0, 0, 0])
  const memHistory = ref<number[]>([0, 0, 0, 0, 0, 0, 0])
  const hasReceivedMetrics = ref(false)

  // Provides history snapshots from node request-based data when the Metrics Server is
  // absent. When the Metrics Server is active, onPodMetricsUpdated is the primary driver.
  watch(
    nodes,
    (newNodes: NodeInfo[]) => {
      if (hasReceivedMetrics.value) return

      let totalCpu = 0
      let usedCpu = 0
      let totalMem = 0
      let usedMem = 0

      for (const node of newNodes) {
        totalCpu += parseFloat(node.cpuTotal || '0')
        usedCpu += parseFloat(node.cpuUsed || '0')
        totalMem += parseFloat(node.memTotal || '0')
        usedMem += parseFloat(node.memUsed || '0')
      }

      const cpuPct = totalCpu > 0 ? (usedCpu / totalCpu) * 100 : 0
      const memPct = totalMem > 0 ? (usedMem / totalMem) * 100 : 0

      cpuHistory.value.shift()
      cpuHistory.value.push(cpuPct)

      memHistory.value.shift()
      memHistory.value.push(memPct)
    },
    { deep: true }
  )

  function setEngineReady(ready: boolean) {
    isEngineReady.value = ready
  }

  function setAppLoading(loading: boolean) {
    isAppLoading.value = loading
  }

  function setPods(newPods: PodInfo[]) {
    pods.value = newPods
    podsLoading.value = false
  }

  function setDeployments(newDeployments: DeploymentInfo[]) {
    deployments.value = newDeployments
    deploymentsLoading.value = false
  }

  function setStatefulSets(newStatefulSets: StatefulSetInfo[]) {
    statefulSets.value = newStatefulSets
    statefulSetsLoading.value = false
  }

  function setDaemonSets(newDaemonSets: DaemonSetInfo[]) {
    daemonSets.value = newDaemonSets
    daemonSetsLoading.value = false
  }

  function setReplicaSets(newReplicaSets: ReplicaSetInfo[]) {
    replicaSets.value = newReplicaSets
    replicaSetsLoading.value = false
  }

  function setJobs(newJobs: JobInfo[]) {
    jobs.value = newJobs
    jobsLoading.value = false
  }

  function setCronJobs(newCronJobs: CronJobInfo[]) {
    cronJobs.value = newCronJobs
    cronJobsLoading.value = false
  }

  function setNodes(newNodes: NodeInfo[]) {
    nodes.value = newNodes
    nodesLoading.value = false
  }

  function setServices(newServices: ServiceInfo[]) {
    services.value = newServices
    servicesLoading.value = false
  }

  function setIngresses(newIngresses: IngressInfo[]) {
    ingresses.value = newIngresses
    ingressesLoading.value = false
  }

  function setConfigMaps(newConfigMaps: ConfigMapInfo[]) {
    configMaps.value = newConfigMaps
    configMapsLoading.value = false
  }

  function setSecrets(newSecrets: SecretInfo[]) {
    secrets.value = newSecrets
    secretsLoading.value = false
  }

  function setEvents(newEvents: EventInfo[]) {
    events.value = newEvents
    eventsLoading.value = false
  }

  function setPolicies(newPolicies: PolicyInfo[]) {
    policies.value = newPolicies
    policiesLoading.value = false
  }

  function setConfigMapsLoading(loading: boolean) {
    configMapsLoading.value = loading
  }

  function setSecretsLoading(loading: boolean) {
    secretsLoading.value = loading
  }

  function setEventsLoading(loading: boolean) {
    eventsLoading.value = loading
  }

  function setPoliciesLoading(loading: boolean) {
    policiesLoading.value = loading
  }

  function setPersistentVolumes(newPVs: PersistentVolumeInfo[]) {
    persistentVolumes.value = newPVs
    persistentVolumesLoading.value = false
  }

  function setPersistentVolumeClaims(newPVCs: PersistentVolumeClaimInfo[]) {
    persistentVolumeClaims.value = newPVCs
    persistentVolumeClaimsLoading.value = false
  }

  function setStorageClasses(newSCs: StorageClassInfo[]) {
    storageClasses.value = newSCs
    storageClassesLoading.value = false
  }

  function setPersistentVolumesLoading(loading: boolean) {
    persistentVolumesLoading.value = loading
  }

  function setPersistentVolumeClaimsLoading(loading: boolean) {
    persistentVolumeClaimsLoading.value = loading
  }

  function setStorageClassesLoading(loading: boolean) {
    storageClassesLoading.value = loading
  }

  function resetAllLoading() {
    isAppLoading.value = false
    podsLoading.value = false
    deploymentsLoading.value = false
    statefulSetsLoading.value = false
    daemonSetsLoading.value = false
    replicaSetsLoading.value = false
    jobsLoading.value = false
    cronJobsLoading.value = false
    nodesLoading.value = false
    servicesLoading.value = false
    ingressesLoading.value = false
    namespacesLoading.value = false
    configMapsLoading.value = false
    secretsLoading.value = false
    eventsLoading.value = false
    policiesLoading.value = false
    persistentVolumesLoading.value = false
    persistentVolumeClaimsLoading.value = false
    storageClassesLoading.value = false
  }

  const namespaces = computed(() => {
    return ['All Namespaces', ...namespaceList.value.map((n) => n.name)]
  })

  function setNamespaces(newNamespaces: NamespaceInfo[]) {
    namespaceList.value = newNamespaces
    namespacesLoading.value = false
  }

  function setClusters(newClusters: ClusterInfo[]) {
    clusters.value = newClusters
  }

  function setActiveClusterId(id: string | null) {
    if (activeClusterId.value === id) return
    activeClusterId.value = id
    // Clear filters when switching clusters
    useTableFilterStore().resetAll()
    // Clear workloads when cluster changes to prevent stale data
    namespaceList.value = []
    deployments.value = []
    statefulSets.value = []
    daemonSets.value = []
    replicaSets.value = []
    jobs.value = []
    cronJobs.value = []
    nodes.value = []
    services.value = []
    ingresses.value = []
    configMaps.value = []
    secrets.value = []
    persistentVolumes.value = []
    persistentVolumeClaims.value = []
    storageClasses.value = []
    events.value = []
    policies.value = []
    podsLoading.value = true
    deploymentsLoading.value = true
    statefulSetsLoading.value = true
    daemonSetsLoading.value = true
    replicaSetsLoading.value = true
    jobsLoading.value = true
    cronJobsLoading.value = true
    nodesLoading.value = true
    servicesLoading.value = true
    ingressesLoading.value = true
    configMapsLoading.value = true
    secretsLoading.value = true
    persistentVolumesLoading.value = true
    persistentVolumeClaimsLoading.value = true
    storageClassesLoading.value = true
    eventsLoading.value = true
    policiesLoading.value = true
    namespacesLoading.value = true
    cpuHistory.value = [0, 0, 0, 0, 0, 0, 0]
    memHistory.value = [0, 0, 0, 0, 0, 0, 0]
    hasReceivedMetrics.value = false

    // Load data for the newly selected cluster
    if (id !== null) {
      loadInitialData()
    }
  }

  async function fetchIngresses(namespace?: string) {
    ingressesLoading.value = true
    try {
      await kubernetesService.getIngresses(namespace)
    } catch (error) {
      ingressesLoading.value = false
      throw error
    }
  }

  async function fetchConfigMaps(namespace?: string) {
    configMapsLoading.value = true
    try {
      await kubernetesService.getConfigMaps(namespace)
    } catch (error) {
      configMapsLoading.value = false
      throw error
    }
  }

  async function fetchSecrets(namespace?: string) {
    secretsLoading.value = true
    try {
      await kubernetesService.getSecrets(namespace)
    } catch (error) {
      secretsLoading.value = false
      throw error
    }
  }

  async function fetchEvents(namespace?: string) {
    eventsLoading.value = true
    try {
      await kubernetesService.getEvents(namespace)
    } catch (error) {
      eventsLoading.value = false
      throw error
    }
  }

  async function fetchPolicies(namespace?: string) {
    policiesLoading.value = true
    try {
      await kubernetesService.getPolicies(namespace)
    } catch (error) {
      policiesLoading.value = false
      throw error
    }
  }

  async function fetchPersistentVolumes() {
    persistentVolumesLoading.value = true
    try {
      await kubernetesService.getPersistentVolumes()
    } catch (error) {
      persistentVolumesLoading.value = false
      throw error
    }
  }

  async function fetchPersistentVolumeClaims(namespace?: string) {
    persistentVolumeClaimsLoading.value = true
    try {
      await kubernetesService.getPersistentVolumeClaims(namespace)
    } catch (error) {
      persistentVolumeClaimsLoading.value = false
      throw error
    }
  }

  async function fetchStorageClasses() {
    storageClassesLoading.value = true
    try {
      await kubernetesService.getStorageClasses()
    } catch (error) {
      storageClassesLoading.value = false
      throw error
    }
  }

  async function loadInitialData() {
    if (isEngineReady.value) {
      podsLoading.value = true
      deploymentsLoading.value = true
      statefulSetsLoading.value = true
      daemonSetsLoading.value = true
      replicaSetsLoading.value = true
      jobsLoading.value = true
      cronJobsLoading.value = true
      nodesLoading.value = true
      servicesLoading.value = true
      namespacesLoading.value = true

      await Promise.all([
        kubernetesService.getClusters(),
        kubernetesService.getNamespaces(),
        kubernetesService.getPods(),
        kubernetesService.getDeployments(),
        kubernetesService.getStatefulSets(),
        kubernetesService.getDaemonSets(),
        kubernetesService.getReplicaSets(),
        kubernetesService.getJobs(),
        kubernetesService.getCronJobs(),
        kubernetesService.getNodes(),
        kubernetesService.getServices(),
        fetchIngresses(),
        fetchConfigMaps(),
        fetchSecrets(),
        fetchPersistentVolumes(),
        fetchPersistentVolumeClaims(),
        fetchStorageClasses(),
        fetchEvents(),
        fetchPolicies()
      ])
    }
  }

  type ResourceInfo =
    | ServiceInfo
    | DeploymentInfo
    | PodInfo
    | StatefulSetInfo
    | DaemonSetInfo
    | ReplicaSetInfo
    | JobInfo
    | CronJobInfo
    | NamespaceInfo
    | ConfigMapInfo
    | SecretInfo
    | EventInfo
    | PersistentVolumeInfo
    | PersistentVolumeClaimInfo
    | StorageClassInfo
    | PolicyInfo

  function onResourceUpdated(payload: {
    kind: string
    action: 'Applied' | 'Deleted'
    data: ResourceInfo
  }) {
    const { kind, action, data } = payload

    // Helper to update a namespaced list
    const updateNamespaced = <T extends NamespacedResource>(listRef: { value: T[] }, item: T) => {
      if (action === 'Applied') {
        const index = listRef.value.findIndex(
          (x) => x.name === item.name && x.namespace === item.namespace
        )
        if (index !== -1) listRef.value.splice(index, 1, item)
        else listRef.value.push(item)
      } else if (action === 'Deleted') {
        listRef.value = listRef.value.filter(
          (x) => !(x.name === item.name && x.namespace === item.namespace)
        )
      }
    }

    // Helper to update a cluster-scoped list (using name or uid)
    const updateClusterScoped = <T extends ClusterScopedResource, K extends 'name' | 'uid'>(
      listRef: { value: T[] },
      item: T,
      key: K
    ) => {
      if (action === 'Applied') {
        const index = listRef.value.findIndex((x) => x[key] === item[key])
        if (index !== -1) listRef.value.splice(index, 1, item)
        else listRef.value.push(item)
      } else if (action === 'Deleted') {
        listRef.value = listRef.value.filter((x) => x[key] !== item[key])
      }
    }

    switch (kind) {
      case 'Service':
        updateClusterScoped(services, data as ServiceInfo, 'uid')
        break
      case 'Deployment':
        updateNamespaced(deployments, data as DeploymentInfo)
        break
      case 'Pod':
        updateNamespaced(pods, data as PodInfo)
        break
      case 'StatefulSet':
        updateNamespaced(statefulSets, data as StatefulSetInfo)
        break
      case 'DaemonSet':
        updateNamespaced(daemonSets, data as DaemonSetInfo)
        break
      case 'ReplicaSet':
        updateNamespaced(replicaSets, data as ReplicaSetInfo)
        break
      case 'Job':
        updateNamespaced(jobs, data as JobInfo)
        break
      case 'CronJob':
        updateNamespaced(cronJobs, data as CronJobInfo)
        break
      case 'Namespace':
        updateClusterScoped(namespaceList, data as NamespaceInfo, 'name')
        break
      case 'ConfigMap':
        updateNamespaced(configMaps, data as ConfigMapInfo)
        break
      case 'Secret':
        updateNamespaced(secrets, data as SecretInfo)
        break
      case 'Event':
        updateClusterScoped(events, data as EventInfo, 'uid')
        break
      case 'PersistentVolume':
        updateClusterScoped(persistentVolumes, data as PersistentVolumeInfo, 'name')
        break
      case 'PersistentVolumeClaim':
        updateNamespaced(persistentVolumeClaims, data as PersistentVolumeClaimInfo)
        break
      case 'StorageClass':
        updateClusterScoped(storageClasses, data as StorageClassInfo, 'name')
        break
      case 'Policy':
        updateClusterScoped(policies, data as PolicyInfo, 'uid')
        break
    }
  }

  function onNodeMetricsUpdated(payload: {
    metrics: Array<{ name: string; cpu: string; memory: string }>
  }) {
    hasReceivedMetrics.value = true

    for (const m of payload.metrics) {
      const node = nodes.value.find((n) => n.name === m.name)
      if (node) {
        node.cpuUsed = m.cpu
        node.memUsed = m.memory

        const totalCpu = parseFloat(node.cpuTotal || '0')
        const totalMem = parseFloat(node.memTotal || '0')
        const usedCpu = parseFloat(m.cpu || '0')
        const usedMem = parseFloat(m.memory || '0')

        node.cpuPct = totalCpu > 0 ? (usedCpu / totalCpu) * 100 : 0
        node.memPct = totalMem > 0 ? (usedMem / totalMem) * 100 : 0
      }
    }

    // Aggregate cluster-wide CPU and memory usage from real node metrics
    let totalCpuCores = 0
    let totalUsedCpuCores = 0
    let totalMemGiB = 0
    let totalUsedMemGiB = 0

    for (const node of nodes.value) {
      totalCpuCores += parseFloat(node.cpuTotal || '0')
      totalUsedCpuCores += parseFloat(node.cpuUsed || '0')
      totalMemGiB += parseFloat(node.memTotal || '0')
      totalUsedMemGiB += parseFloat(node.memUsed || '0')
    }

    const cpuPct = totalCpuCores > 0 ? Math.min((totalUsedCpuCores / totalCpuCores) * 100, 100) : 0
    const memPct = totalMemGiB > 0 ? Math.min((totalUsedMemGiB / totalMemGiB) * 100, 100) : 0

    // Push new data points into the sliding-window history for the dashboard charts.
    cpuHistory.value.shift()
    cpuHistory.value.push(cpuPct)
    memHistory.value.shift()
    memHistory.value.push(memPct)
  }

  function onPodMetricsUpdated(payload: {
    metrics: Array<{ name: string; namespace: string; cpu: string; memory: string }>
  }) {
    for (const m of payload.metrics) {
      const pod = pods.value.find((p) => p.name === m.name && p.namespace === m.namespace)
      if (pod) {
        pod.cpu = m.cpu
        pod.memory = m.memory
      }
    }

    // If node metrics are not available on this cluster, use pod metrics as fallback for history.
    if (!hasReceivedMetrics.value) {
      let totalUsedCpuCores = 0
      let totalUsedMemGiB = 0

      for (const m of payload.metrics) {
        totalUsedCpuCores += parseCpuToCores(m.cpu)
        totalUsedMemGiB += parseMemToGiB(m.memory)
      }

      const totalCpuCores = nodes.value.reduce(
        (acc, node) => acc + parseFloat(node.cpuTotal || '0'),
        0
      )
      const totalMemGiB = nodes.value.reduce(
        (acc, node) => acc + parseFloat(node.memTotal || '0'),
        0
      )

      const cpuPct =
        totalCpuCores > 0 ? Math.min((totalUsedCpuCores / totalCpuCores) * 100, 100) : 0
      const memPct = totalMemGiB > 0 ? Math.min((totalUsedMemGiB / totalMemGiB) * 100, 100) : 0

      cpuHistory.value.shift()
      cpuHistory.value.push(cpuPct)
      memHistory.value.shift()
      memHistory.value.push(memPct)
    }
  }

  nativeEvents.on(OrbitEvents.ResourceUpdated, onResourceUpdated)
  nativeEvents.on(OrbitEvents.PodMetricsUpdated, onPodMetricsUpdated)
  nativeEvents.on(OrbitEvents.NodeMetricsUpdated, onNodeMetricsUpdated)

  onScopeDispose(() => {
    nativeEvents.off(OrbitEvents.ResourceUpdated, onResourceUpdated)
    nativeEvents.off(OrbitEvents.PodMetricsUpdated, onPodMetricsUpdated)
    nativeEvents.off(OrbitEvents.NodeMetricsUpdated, onNodeMetricsUpdated)
  })

  return {
    isEngineReady,
    isAppLoading,
    pods,
    deployments,
    statefulSets,
    daemonSets,
    replicaSets,
    jobs,
    cronJobs,
    nodes,
    services,
    configMaps,
    secrets,
    events,
    policies,
    persistentVolumes,
    persistentVolumeClaims,
    storageClasses,
    configMapsLoading,
    secretsLoading,
    eventsLoading,
    policiesLoading,
    persistentVolumesLoading,
    persistentVolumeClaimsLoading,
    storageClassesLoading,
    podsLoading,
    deploymentsLoading,
    statefulSetsLoading,
    daemonSetsLoading,
    replicaSetsLoading,
    jobsLoading,
    cronJobsLoading,
    nodesLoading,
    servicesLoading,
    ingresses,
    ingressesLoading,
    namespacesLoading,
    namespaces,
    namespaceList,
    clusters,
    activeClusterId,
    setEngineReady,
    setAppLoading,
    resetAllLoading,
    setPods,
    setDeployments,
    setStatefulSets,
    setDaemonSets,
    setReplicaSets,
    setJobs,
    setCronJobs,
    setNodes,
    setServices,
    setIngresses,
    setConfigMaps,
    setSecrets,
    setEvents,
    setPolicies,
    setPersistentVolumes,
    setPersistentVolumeClaims,
    setStorageClasses,
    setConfigMapsLoading,
    setSecretsLoading,
    setEventsLoading,
    setPoliciesLoading,
    setPersistentVolumesLoading,
    setPersistentVolumeClaimsLoading,
    setStorageClassesLoading,
    setNamespaces,
    setClusters,
    setActiveClusterId,
    fetchIngresses,
    fetchConfigMaps,
    fetchSecrets,
    fetchEvents,
    fetchPolicies,
    fetchPersistentVolumes,
    fetchPersistentVolumeClaims,
    fetchStorageClasses,
    loadInitialData,
    cpuHistory,
    memHistory
  }
})
