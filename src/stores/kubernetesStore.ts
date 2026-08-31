import { kubernetesService } from '@/services/kubernetesService'
import { events as nativeEvents } from '@/services/nativeService'
import { useTableFilterStore } from '@/stores/tableFilterStore'
import { OrbitEvents } from '@/types/events'
import {
  KUBERNETES_ACTION,
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
} from '@/types/kubernetes'
import { defineStore } from 'pinia'
import { computed, onScopeDispose, ref, watch, type Ref } from 'vue'

type ResourceMatcher<T> = (existing: T, incoming: T) => boolean

const matchNamespaced: ResourceMatcher<{ name: string; namespace: string }> = (a, b) =>
  a.name === b.name && a.namespace === b.namespace

const matchByName: ResourceMatcher<{ name?: string }> = (a, b) =>
  Boolean(a.name && a.name === b.name)

const matchByUid: ResourceMatcher<{ uid?: string }> = (a, b) => Boolean(a.uid && a.uid === b.uid)

const matchService: ResourceMatcher<ServiceInfo> = (a, b) =>
  (Boolean(a.uid) && a.uid === b.uid) || (a.name === b.name && a.namespace === b.namespace)

function updateResourceList<T>(
  listRef: Ref<T[]>,
  action: KubernetesAction,
  item: T,
  match: ResourceMatcher<T>
) {
  const current = listRef.value
  const index = current.findIndex((existing) => match(existing, item))

  if (action === KUBERNETES_ACTION.Applied) {
    if (index !== -1) {
      current.splice(index, 1, item)
    } else {
      current.push(item)
    }
  } else if (action === KUBERNETES_ACTION.Deleted) {
    if (index !== -1) {
      current.splice(index, 1)
    }
  }
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

  const persistentVolumesLoading = ref(false)
  const persistentVolumeClaimsLoading = ref(false)
  const storageClassesLoading = ref(false)
  const namespaceList = ref<NamespaceInfo[]>([])
  const clusters = ref<ClusterInfo[]>([])
  const activeClusterId = ref<string | null>(null)
  const lastUpdatedAt = ref<Date | null>(null)

  const cpuHistory = ref<number[]>([0, 0, 0, 0, 0, 0, 0])
  const memHistory = ref<number[]>([0, 0, 0, 0, 0, 0, 0])

  watch(
    nodes,
    (newNodes: NodeInfo[]) => {
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

  type ResourceInfo =
    | ServiceInfo
    | IngressInfo
    | DeploymentInfo
    | PodInfo
    | StatefulSetInfo
    | DaemonSetInfo
    | ReplicaSetInfo
    | JobInfo
    | CronJobInfo
    | NodeInfo
    | NamespaceInfo
    | ConfigMapInfo
    | SecretInfo
    | EventInfo
    | PersistentVolumeInfo
    | PersistentVolumeClaimInfo
    | StorageClassInfo
    | PolicyInfo

  const resourceUpdaters: Record<string, (action: KubernetesAction, data: ResourceInfo) => void> = {
    [KUBERNETES_RESOURCE_KIND.Deployment]: (action, data) =>
      updateResourceList(deployments, action, data as DeploymentInfo, matchNamespaced),
    [KUBERNETES_RESOURCE_KIND.DaemonSet]: (action, data) =>
      updateResourceList(daemonSets, action, data as DaemonSetInfo, matchNamespaced),
    [KUBERNETES_RESOURCE_KIND.StatefulSet]: (action, data) =>
      updateResourceList(statefulSets, action, data as StatefulSetInfo, matchNamespaced),
    [KUBERNETES_RESOURCE_KIND.ReplicaSet]: (action, data) =>
      updateResourceList(replicaSets, action, data as ReplicaSetInfo, matchNamespaced),
    [KUBERNETES_RESOURCE_KIND.Job]: (action, data) =>
      updateResourceList(jobs, action, data as JobInfo, matchNamespaced),
    [KUBERNETES_RESOURCE_KIND.CronJob]: (action, data) =>
      updateResourceList(cronJobs, action, data as CronJobInfo, matchNamespaced),
    [KUBERNETES_RESOURCE_KIND.Pod]: (action, data) =>
      updateResourceList(pods, action, data as PodInfo, matchNamespaced),
    [KUBERNETES_RESOURCE_KIND.ConfigMap]: (action, data) =>
      updateResourceList(configMaps, action, data as ConfigMapInfo, matchNamespaced),
    [KUBERNETES_RESOURCE_KIND.Secret]: (action, data) =>
      updateResourceList(secrets, action, data as SecretInfo, matchNamespaced),
    [KUBERNETES_RESOURCE_KIND.PersistentVolumeClaim]: (action, data) =>
      updateResourceList(
        persistentVolumeClaims,
        action,
        data as PersistentVolumeClaimInfo,
        matchNamespaced
      ),
    [KUBERNETES_RESOURCE_KIND.Ingress]: (action, data) =>
      updateResourceList(ingresses, action, data as IngressInfo, matchNamespaced),
    [KUBERNETES_RESOURCE_KIND.Service]: (action, data) =>
      updateResourceList(services, action, data as ServiceInfo, matchService),
    [KUBERNETES_RESOURCE_KIND.Namespace]: (action, data) =>
      updateResourceList(namespaceList, action, data as NamespaceInfo, matchByName),
    [KUBERNETES_RESOURCE_KIND.Node]: (action, data) =>
      updateResourceList(nodes, action, data as NodeInfo, matchByName),
    [KUBERNETES_RESOURCE_KIND.PersistentVolume]: (action, data) =>
      updateResourceList(persistentVolumes, action, data as PersistentVolumeInfo, matchByName),
    [KUBERNETES_RESOURCE_KIND.StorageClass]: (action, data) =>
      updateResourceList(storageClasses, action, data as StorageClassInfo, matchByName),
    [KUBERNETES_RESOURCE_KIND.Event]: (action, data) =>
      updateResourceList(events, action, data as EventInfo, matchByUid),
    [KUBERNETES_RESOURCE_KIND.Policy]: (action, data) =>
      updateResourceList(policies, action, data as PolicyInfo, matchByUid)
  }

  function onResourceUpdated(payload: {
    kind: string
    action: KubernetesAction
    data: ResourceInfo
  }) {
    const handler = resourceUpdaters[payload.kind]
    if (handler) {
      handler(payload.action, payload.data)
    }
  }

  const activePortForwards = ref<ActivePortForward[]>([])

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
  }

  function onPortForwardStarted(payload: ActivePortForward) {
    if (!activePortForwards.value.some((f) => f.id === payload.id)) {
      activePortForwards.value = [...activePortForwards.value, payload]
    }
  }

  function onPortForwardStopped(payload: { id: string }) {
    activePortForwards.value = activePortForwards.value.filter((f) => f.id !== payload.id)
  }

  nativeEvents.on(OrbitEvents.ResourceUpdated, onResourceUpdated)
  nativeEvents.on(OrbitEvents.PodMetricsUpdated, onPodMetricsUpdated)
  nativeEvents.on(OrbitEvents.PortForwardStarted, onPortForwardStarted)
  nativeEvents.on(OrbitEvents.PortForwardStopped, onPortForwardStopped)

  onScopeDispose(() => {
    nativeEvents.off(OrbitEvents.ResourceUpdated, onResourceUpdated)
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
