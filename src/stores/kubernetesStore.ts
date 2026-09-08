import { kubernetesService } from '../services/kubernetesService.ts'
import { events as nativeEvents } from '../services/nativeService.ts'
import { useTableFilterStore } from './tableFilterStore.ts'
import { OrbitEvents, type KubernetesResourceInfo } from '../types/events.ts'
import { formatCpuCores, formatDecimal, formatMemoryMiB } from '../utils/metrics.ts'
import {
  KUBERNETES_RESOURCE_KIND,
  type ActivePortForward,
  type ClusterInfo,
  type ConfigMapInfo,
  type CronJobInfo,
  type DaemonSetInfo,
  type DeploymentInfo,
  type EventInfo,
  type IngressInfo,
  type JobInfo,
  type KubernetesAction,
  type NamespaceInfo,
  type NodeInfo,
  type PersistentVolumeClaimInfo,
  type PersistentVolumeInfo,
  type PodInfo,
  type PolicyInfo,
  type ReplicaSetInfo,
  type SecretInfo,
  type ServiceInfo,
  type StatefulSetInfo,
  type StorageClassInfo
} from '../types/kubernetes.ts'
import { defineStore } from 'pinia'
import { computed, onScopeDispose, ref, shallowRef, watch } from 'vue'
import {
  keyByName,
  keyByUid,
  keyNamespaced,
  keyService,
  MAX_EVENTS_RETAINED,
  updateResourceBatch,
  type ResourceKeyExtractor
} from './kubernetesStoreBatch.ts'

export {
  MAX_EVENTS_RETAINED,
  type ResourceKeyExtractor,
  keyNamespaced,
  keyByName,
  keyByUid,
  keyService,
  updateResourceBatch
}

export const useKubernetesStore = defineStore('kubernetes', () => {
  const isEngineReady = ref(false)
  const isAppLoading = ref(true)
  const pods = shallowRef<PodInfo[]>([])
  const deployments = shallowRef<DeploymentInfo[]>([])
  const statefulSets = shallowRef<StatefulSetInfo[]>([])
  const daemonSets = shallowRef<DaemonSetInfo[]>([])
  const replicaSets = shallowRef<ReplicaSetInfo[]>([])
  const jobs = shallowRef<JobInfo[]>([])
  const cronJobs = shallowRef<CronJobInfo[]>([])
  const nodes = shallowRef<NodeInfo[]>([])
  const services = shallowRef<ServiceInfo[]>([])
  const ingresses = shallowRef<IngressInfo[]>([])
  const configMaps = shallowRef<ConfigMapInfo[]>([])
  const secrets = shallowRef<SecretInfo[]>([])
  const events = shallowRef<EventInfo[]>([])
  const policies = shallowRef<PolicyInfo[]>([])
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
  const persistentVolumes = shallowRef<PersistentVolumeInfo[]>([])
  const persistentVolumeClaims = shallowRef<PersistentVolumeClaimInfo[]>([])
  const storageClasses = shallowRef<StorageClassInfo[]>([])

  const persistentVolumesLoading = ref(false)
  const persistentVolumeClaimsLoading = ref(false)
  const storageClassesLoading = ref(false)
  const namespaceList = shallowRef<NamespaceInfo[]>([])
  const clusters = shallowRef<ClusterInfo[]>([])
  const activeClusterId = ref<string | null>(null)
  const lastUpdatedAt = ref<Date | null>(null)

  const cpuHistory = ref<number[]>([0, 0, 0, 0, 0, 0, 0])
  const memHistory = ref<number[]>([0, 0, 0, 0, 0, 0, 0])

  watch(nodes, (newNodes: NodeInfo[]) => {
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
  })

  const podMetricsMap = new Map<
    string,
    {
      cpu: string
      memory: string
      cpuCores?: number
      memoryBytes?: number
    }
  >()

  function enrichPodWithMetrics(pod: PodInfo, nodesList: NodeInfo[] = nodes.value): PodInfo {
    const key = `${pod.namespace}/${pod.name}`
    const cached = podMetricsMap.get(key)

    const cpuCores = typeof pod.cpuCores === 'number' ? pod.cpuCores : cached?.cpuCores
    const memoryBytes = typeof pod.memoryBytes === 'number' ? pod.memoryBytes : cached?.memoryBytes
    let cpu = pod.cpu ?? cached?.cpu
    let memory = pod.memory ?? cached?.memory
    let cpuPct = typeof pod.cpuPct === 'number' ? pod.cpuPct : undefined
    let memoryPct = typeof pod.memoryPct === 'number' ? pod.memoryPct : undefined

    const node = pod.node ? nodesList.find((n) => n.name === pod.node) : undefined

    if (cpuCores !== undefined && node) {
      const nodeCpuTotal = parseFloat(node.cpuTotal || '0')
      if (nodeCpuTotal > 0) {
        cpuPct = Number(((cpuCores / nodeCpuTotal) * 100).toFixed(2))
        cpu = `${formatDecimal(cpuCores, 2)}/${formatDecimal(nodeCpuTotal, 2)} cores`
      }
    } else if (cpuCores !== undefined && (!cpu || cpu === '-')) {
      cpu = formatCpuCores(cpuCores)
    }

    if (memoryBytes !== undefined && node) {
      const nodeMemGib = parseFloat(node.memTotal || '0')
      if (nodeMemGib > 0) {
        const nodeMemBytes = nodeMemGib * 1024 * 1024 * 1024
        memoryPct = Number(((memoryBytes / nodeMemBytes) * 100).toFixed(2))
      }
    }

    if (memoryBytes !== undefined && (!memory || memory === '-')) {
      const memMib = memoryBytes / (1024 * 1024)
      memory = formatMemoryMiB(memMib)
    }

    return {
      ...pod,
      cpu,
      cpuCores,
      cpuPct,
      memory,
      memoryBytes,
      memoryPct
    }
  }

  function setEngineReady(ready: boolean) {
    isEngineReady.value = ready
  }

  function setAppLoading(loading: boolean) {
    isAppLoading.value = loading
  }

  function setPods(newPods: PodInfo[]) {
    pods.value = newPods.map((p) => enrichPodWithMetrics(p))
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
    if (pods.value.length > 0) {
      pods.value = pods.value.map((p) => enrichPodWithMetrics(p, newNodes))
    }
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
    // Sync active cluster and reset transient table selection state when switching clusters
    const tableFilterStore = useTableFilterStore()
    tableFilterStore.setActiveClusterId(id)
    tableFilterStore.resetAllSelections()
    podMetricsMap.clear()
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
    kubernetesService.stopPortForward().catch((err) => {
      console.error('Failed to stop port forwards on cluster switch:', err)
    })
    activePortForwards.value = []
    cpuHistory.value = [0, 0, 0, 0, 0, 0, 0]
    memHistory.value = [0, 0, 0, 0, 0, 0, 0]

    // Load data for the newly selected cluster
    if (id !== null) {
      loadInitialData()
    } else {
      lastUpdatedAt.value = null
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
      lastUpdatedAt.value = new Date()
    }
  }

  const resourceUpdaters: Record<
    string,
    (updates: Array<{ action: KubernetesAction; data: KubernetesResourceInfo }>) => void
  > = {
    [KUBERNETES_RESOURCE_KIND.Deployment]: (updates) =>
      updateResourceBatch(
        deployments,
        updates as Array<{ action: KubernetesAction; data: DeploymentInfo }>,
        keyNamespaced
      ),
    [KUBERNETES_RESOURCE_KIND.DaemonSet]: (updates) =>
      updateResourceBatch(
        daemonSets,
        updates as Array<{ action: KubernetesAction; data: DaemonSetInfo }>,
        keyNamespaced
      ),
    [KUBERNETES_RESOURCE_KIND.StatefulSet]: (updates) =>
      updateResourceBatch(
        statefulSets,
        updates as Array<{ action: KubernetesAction; data: StatefulSetInfo }>,
        keyNamespaced
      ),
    [KUBERNETES_RESOURCE_KIND.ReplicaSet]: (updates) =>
      updateResourceBatch(
        replicaSets,
        updates as Array<{ action: KubernetesAction; data: ReplicaSetInfo }>,
        keyNamespaced
      ),
    [KUBERNETES_RESOURCE_KIND.Job]: (updates) =>
      updateResourceBatch(
        jobs,
        updates as Array<{ action: KubernetesAction; data: JobInfo }>,
        keyNamespaced
      ),
    [KUBERNETES_RESOURCE_KIND.CronJob]: (updates) =>
      updateResourceBatch(
        cronJobs,
        updates as Array<{ action: KubernetesAction; data: CronJobInfo }>,
        keyNamespaced
      ),
    [KUBERNETES_RESOURCE_KIND.Pod]: (updates) =>
      updateResourceBatch(
        pods,
        updates as Array<{ action: KubernetesAction; data: PodInfo }>,
        keyNamespaced,
        (existing, incoming) =>
          enrichPodWithMetrics({
            ...incoming,
            cpu: incoming.cpu ?? existing.cpu,
            cpuCores: typeof incoming.cpuCores === 'number' ? incoming.cpuCores : existing.cpuCores,
            cpuPct: typeof incoming.cpuPct === 'number' ? incoming.cpuPct : existing.cpuPct,
            memory: incoming.memory ?? existing.memory,
            memoryBytes:
              typeof incoming.memoryBytes === 'number'
                ? incoming.memoryBytes
                : existing.memoryBytes,
            memoryPct:
              typeof incoming.memoryPct === 'number' ? incoming.memoryPct : existing.memoryPct
          })
      ),
    [KUBERNETES_RESOURCE_KIND.ConfigMap]: (updates) =>
      updateResourceBatch(
        configMaps,
        updates as Array<{ action: KubernetesAction; data: ConfigMapInfo }>,
        keyNamespaced
      ),
    [KUBERNETES_RESOURCE_KIND.Secret]: (updates) =>
      updateResourceBatch(
        secrets,
        updates as Array<{ action: KubernetesAction; data: SecretInfo }>,
        keyNamespaced
      ),
    [KUBERNETES_RESOURCE_KIND.PersistentVolumeClaim]: (updates) =>
      updateResourceBatch(
        persistentVolumeClaims,
        updates as Array<{ action: KubernetesAction; data: PersistentVolumeClaimInfo }>,
        keyNamespaced
      ),
    [KUBERNETES_RESOURCE_KIND.Ingress]: (updates) =>
      updateResourceBatch(
        ingresses,
        updates as Array<{ action: KubernetesAction; data: IngressInfo }>,
        keyNamespaced
      ),
    [KUBERNETES_RESOURCE_KIND.Service]: (updates) =>
      updateResourceBatch(
        services,
        updates as Array<{ action: KubernetesAction; data: ServiceInfo }>,
        keyService
      ),
    [KUBERNETES_RESOURCE_KIND.Namespace]: (updates) =>
      updateResourceBatch(
        namespaceList,
        updates as Array<{ action: KubernetesAction; data: NamespaceInfo }>,
        keyByName
      ),
    [KUBERNETES_RESOURCE_KIND.Node]: (updates) =>
      updateResourceBatch(
        nodes,
        updates as Array<{ action: KubernetesAction; data: NodeInfo }>,
        keyByName
      ),
    [KUBERNETES_RESOURCE_KIND.PersistentVolume]: (updates) =>
      updateResourceBatch(
        persistentVolumes,
        updates as Array<{ action: KubernetesAction; data: PersistentVolumeInfo }>,
        keyByName
      ),
    [KUBERNETES_RESOURCE_KIND.StorageClass]: (updates) =>
      updateResourceBatch(
        storageClasses,
        updates as Array<{ action: KubernetesAction; data: StorageClassInfo }>,
        keyByName
      ),
    [KUBERNETES_RESOURCE_KIND.Event]: (updates) =>
      updateResourceBatch(
        events,
        updates as Array<{ action: KubernetesAction; data: EventInfo }>,
        keyByUid,
        undefined,
        MAX_EVENTS_RETAINED
      ),
    [KUBERNETES_RESOURCE_KIND.Policy]: (updates) =>
      updateResourceBatch(
        policies,
        updates as Array<{ action: KubernetesAction; data: PolicyInfo }>,
        keyByUid
      )
  }

  function onResourceBatchUpdated(payload: {
    kind: string
    updates: Array<{ action: KubernetesAction; data: KubernetesResourceInfo }>
  }) {
    const handler = resourceUpdaters[payload.kind]
    if (handler) {
      handler(payload.updates)
    }
  }

  const activePortForwards = ref<ActivePortForward[]>([])

  function onPodMetricsUpdated(payload: {
    metrics: Array<{
      name: string
      namespace: string
      cpu: string
      memory: string
      cpuCores?: number
      memoryBytes?: number
    }>
  }) {
    for (const m of payload.metrics) {
      podMetricsMap.set(`${m.namespace}/${m.name}`, {
        cpu: m.cpu,
        memory: m.memory,
        cpuCores: m.cpuCores,
        memoryBytes: m.memoryBytes
      })
    }
    if (pods.value.length > 0) {
      pods.value = pods.value.map((p) => enrichPodWithMetrics(p))
    }
  }

  function onPortForwardStarted(payload: ActivePortForward) {
    if (!activePortForwards.value.some((f) => f.id === payload.id)) {
      activePortForwards.value = [...activePortForwards.value, payload]
    }
  }

  function onPortForwardStopped(payload: { id: string }) {
    activePortForwards.value = activePortForwards.value.filter((f) => f.id !== payload.id)
  }

  nativeEvents.on(OrbitEvents.ResourceBatchUpdated, onResourceBatchUpdated)
  nativeEvents.on(OrbitEvents.PodMetricsUpdated, onPodMetricsUpdated)
  nativeEvents.on(OrbitEvents.PortForwardStarted, onPortForwardStarted)
  nativeEvents.on(OrbitEvents.PortForwardStopped, onPortForwardStopped)

  onScopeDispose(() => {
    nativeEvents.off(OrbitEvents.ResourceBatchUpdated, onResourceBatchUpdated)
    nativeEvents.off(OrbitEvents.PodMetricsUpdated, onPodMetricsUpdated)
    nativeEvents.off(OrbitEvents.PortForwardStarted, onPortForwardStarted)
    nativeEvents.off(OrbitEvents.PortForwardStopped, onPortForwardStopped)
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
    lastUpdatedAt,
    cpuHistory,
    memHistory,
    activePortForwards
  }
})
