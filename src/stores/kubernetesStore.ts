import { kubernetesService } from '@/services/kubernetesService'
import { events as nativeEvents } from '@/services/nativeService'
import type {
  ClusterInfo,
  ConfigMapInfo,
  CronJobInfo,
  DaemonSetInfo,
  DeploymentInfo,
  EventInfo,
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
import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'

export const useKubernetesStore = defineStore('kubernetes', () => {
  const isEngineReady = ref(false)
  const pods = ref<PodInfo[]>([])
  const deployments = ref<DeploymentInfo[]>([])
  const statefulSets = ref<StatefulSetInfo[]>([])
  const daemonSets = ref<DaemonSetInfo[]>([])
  const replicaSets = ref<ReplicaSetInfo[]>([])
  const jobs = ref<JobInfo[]>([])
  const cronJobs = ref<CronJobInfo[]>([])
  const nodes = ref<NodeInfo[]>([])
  const services = ref<ServiceInfo[]>([])
  const configMaps = ref<ConfigMapInfo[]>([])
  const secrets = ref<SecretInfo[]>([])
  const events = ref<EventInfo[]>([])
  const policies = ref<PolicyInfo[]>([])
  const configMapsLoading = ref(false)
  const secretsLoading = ref(false)
  const eventsLoading = ref(false)
  const policiesLoading = ref(false)
  const persistentVolumes = ref<PersistentVolumeInfo[]>([])
  const persistentVolumeClaims = ref<PersistentVolumeClaimInfo[]>([])
  const storageClasses = ref<StorageClassInfo[]>([])
  const persistentVolumesLoading = ref(false)
  const persistentVolumeClaimsLoading = ref(false)
  const storageClassesLoading = ref(false)
  const namespaceList = ref<NamespaceInfo[]>([])
  const clusters = ref<ClusterInfo[]>([])
  const activeClusterId = ref<string | null>(null)

  const cpuHistory = ref<number[]>([0, 0, 0, 0, 0, 0, 0])
  const memHistory = ref<number[]>([0, 0, 0, 0, 0, 0, 0])

  watch(
    nodes,
    (newNodes) => {
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

  function setPods(newPods: PodInfo[]) {
    pods.value = newPods
  }

  function setDeployments(newDeployments: DeploymentInfo[]) {
    deployments.value = newDeployments
  }

  function setStatefulSets(newStatefulSets: StatefulSetInfo[]) {
    statefulSets.value = newStatefulSets
  }

  function setDaemonSets(newDaemonSets: DaemonSetInfo[]) {
    daemonSets.value = newDaemonSets
  }

  function setReplicaSets(newReplicaSets: ReplicaSetInfo[]) {
    replicaSets.value = newReplicaSets
  }

  function setJobs(newJobs: JobInfo[]) {
    jobs.value = newJobs
  }

  function setCronJobs(newCronJobs: CronJobInfo[]) {
    cronJobs.value = newCronJobs
  }

  function setNodes(newNodes: NodeInfo[]) {
    nodes.value = newNodes
  }

  function setServices(newServices: ServiceInfo[]) {
    services.value = newServices
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

  const namespaces = computed(() => {
    return ['All Namespaces', ...namespaceList.value.map((n) => n.name)]
  })

  function setNamespaces(newNamespaces: NamespaceInfo[]) {
    namespaceList.value = newNamespaces
  }

  function setClusters(newClusters: ClusterInfo[]) {
    clusters.value = newClusters
  }

  function setActiveClusterId(id: string | null) {
    activeClusterId.value = id
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
    configMaps.value = []
    secrets.value = []
    persistentVolumes.value = []
    persistentVolumeClaims.value = []
    storageClasses.value = []
    events.value = []
    policies.value = []
    cpuHistory.value = [0, 0, 0, 0, 0, 0, 0]
    memHistory.value = [0, 0, 0, 0, 0, 0, 0]
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
      await kubernetesService.getClusters()
      await kubernetesService.getNamespaces()
      await kubernetesService.getPods()
      await kubernetesService.getDeployments()
      await kubernetesService.getStatefulSets()
      await kubernetesService.getDaemonSets()
      await kubernetesService.getJobs()
      await kubernetesService.getCronJobs()
      await kubernetesService.getNodes()
      await kubernetesService.getServices()
      await fetchConfigMaps()
      await fetchSecrets()
      await fetchPersistentVolumes()
      await fetchPersistentVolumeClaims()
      await fetchStorageClasses()
      await fetchEvents()
      await fetchPolicies()
    }
  }

  nativeEvents.on('resourceUpdated', (payload) => {
    console.log('[Real-Time] Resource Updated:', payload)
    const { kind, action, data } = payload
    if (kind === 'Service') {
      const svc = data as ServiceInfo
      if (action === 'Applied') {
        const index = services.value.findIndex((s) => s.uid === svc.uid)
        if (index !== -1) {
          services.value.splice(index, 1, svc)
        } else {
          services.value.push(svc)
        }
      } else if (action === 'Deleted') {
        services.value = services.value.filter((s) => s.uid !== svc.uid)
      }
    }
  })

  return {
    isEngineReady,
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
    namespaces,
    namespaceList,
    clusters,
    activeClusterId,
    setEngineReady,
    setPods,
    setDeployments,
    setStatefulSets,
    setDaemonSets,
    setReplicaSets,
    setJobs,
    setCronJobs,
    setNodes,
    setServices,
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
