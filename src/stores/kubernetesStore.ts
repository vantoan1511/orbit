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
  NodeInfo
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
  }

  async function loadInitialData() {
    if (isEngineReady.value) {
      await kubernetesService.getClusters()
      await kubernetesService.getNamespaces()
      await kubernetesService.getPods()
      await kubernetesService.getNodes()
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
    setNamespaces,
    setClusters,
    setActiveClusterId,
    loadInitialData
  }
})
