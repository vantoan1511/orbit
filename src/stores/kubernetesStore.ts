import { defineStore } from 'pinia'
import { ref } from 'vue'
import type {
  PodInfo,
  ClusterInfo,
  DeploymentInfo,
  StatefulSetInfo,
  DaemonSetInfo,
  ReplicaSetInfo,
  JobInfo,
  CronJobInfo,
  NodeInfo,
  ServiceInfo,
  ConfigMapInfo,
  SecretInfo,
  PersistentVolumeInfo,
  PersistentVolumeClaimInfo,
  StorageClassInfo
} from '@/types/kubernetes'
import { kubernetesService } from '@/services/kubernetesService'

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
  const configMapsLoading = ref(false)
  const secretsLoading = ref(false)
  const persistentVolumes = ref<PersistentVolumeInfo[]>([])
  const persistentVolumeClaims = ref<PersistentVolumeClaimInfo[]>([])
  const storageClasses = ref<StorageClassInfo[]>([])
  const persistentVolumesLoading = ref(false)
  const persistentVolumeClaimsLoading = ref(false)
  const storageClassesLoading = ref(false)
  const namespaces = ref<string[]>(['All Namespaces'])
  const clusters = ref<ClusterInfo[]>([])
  const activeClusterId = ref<string | null>(null)

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

  function setConfigMapsLoading(loading: boolean) {
    configMapsLoading.value = loading
  }

  function setSecretsLoading(loading: boolean) {
    secretsLoading.value = loading
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

  function setNamespaces(newNamespaces: string[]) {
    namespaces.value = ['All Namespaces', ...newNamespaces]
  }

  function setClusters(newClusters: ClusterInfo[]) {
    clusters.value = newClusters
  }

  function setActiveClusterId(id: string | null) {
    activeClusterId.value = id
    // Clear workloads when cluster changes to prevent stale data
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
      await kubernetesService.getNodes()
      await kubernetesService.getServices()
      await fetchConfigMaps()
      await fetchSecrets()
      await fetchPersistentVolumes()
      await fetchPersistentVolumeClaims()
      await fetchStorageClasses()
    }
  }

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
    persistentVolumes,
    persistentVolumeClaims,
    storageClasses,
    configMapsLoading,
    secretsLoading,
    persistentVolumesLoading,
    persistentVolumeClaimsLoading,
    storageClassesLoading,
    namespaces,
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
    setPersistentVolumes,
    setPersistentVolumeClaims,
    setStorageClasses,
    setConfigMapsLoading,
    setSecretsLoading,
    setPersistentVolumesLoading,
    setPersistentVolumeClaimsLoading,
    setStorageClassesLoading,
    setNamespaces,
    setClusters,
    setActiveClusterId,
    fetchConfigMaps,
    fetchSecrets,
    fetchPersistentVolumes,
    fetchPersistentVolumeClaims,
    fetchStorageClasses,
    loadInitialData
  }
})
