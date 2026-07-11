<script setup lang="ts">
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import type {
  ClusterInfo,
  PodInfo,
  DeploymentInfo,
  StatefulSetInfo,
  DaemonSetInfo,
  ReplicaSetInfo,
  JobInfo,
  CronJobInfo,
  NodeInfo
} from '@/types/kubernetes'
import ConfirmDialog from 'primevue/confirmdialog'
import Toast from 'primevue/toast'
import { useToast } from 'primevue/usetoast'
import { onMounted, onUnmounted } from 'vue'
import AppLayout from './components/layout/AppLayout.vue'

const k8sStore = useKubernetesStore()
const toast = useToast()

const handleEngineConnected = (payload: { status: 'ready' | 'error'; message: string }) => {
  if (payload.status === 'ready') {
    k8sStore.setEngineReady(true)
    k8sStore.loadInitialData()
  } else {
    k8sStore.setEngineReady(false)
  }
}

const handleNamespacesUpdated = (payload: { namespaces: string[] }) => {
  k8sStore.setNamespaces(payload.namespaces)
}

const handlePodsUpdated = (payload: { pods: PodInfo[] }) => {
  k8sStore.setPods(payload.pods)
}

const handleDeploymentsUpdated = (payload: { deployments: DeploymentInfo[] }) => {
  k8sStore.setDeployments(payload.deployments)
}

const handleStatefulSetsUpdated = (payload: { stateful_sets: StatefulSetInfo[] }) => {
  k8sStore.setStatefulSets(payload.stateful_sets)
}

const handleDaemonSetsUpdated = (payload: { daemon_sets: DaemonSetInfo[] }) => {
  k8sStore.setDaemonSets(payload.daemon_sets)
}

const handleReplicaSetsUpdated = (payload: { replica_sets: ReplicaSetInfo[] }) => {
  k8sStore.setReplicaSets(payload.replica_sets)
}

const handleJobsUpdated = (payload: { jobs: JobInfo[] }) => {
  k8sStore.setJobs(payload.jobs)
}

const handleCronJobsUpdated = (payload: { cron_jobs: CronJobInfo[] }) => {
  k8sStore.setCronJobs(payload.cron_jobs)
}

const handleNodesUpdated = (payload: { nodes: NodeInfo[] }) => {
  k8sStore.setNodes(payload.nodes)
}

const handleClustersUpdated = (payload: { clusters: ClusterInfo[] }) => {
  k8sStore.setClusters(payload.clusters)
}

const handleActiveClusterChanged = (payload: { active_cluster_id: string | null }) => {
  k8sStore.setActiveClusterId(payload.active_cluster_id)
}

const handleErrorOccurred = (payload: { message: string }) => {
  toast.add({
    severity: 'error',
    summary: 'Error',
    detail: payload.message,
    life: 5000
  })
}

onMounted(() => {
  // Initialize dark mode by default
  document.documentElement.classList.add('my-app-dark')
  document.documentElement.setAttribute('data-theme', 'dark')

  events.on(OrbitEvents.EngineConnected, handleEngineConnected)
  events.on(OrbitEvents.NamespacesUpdated, handleNamespacesUpdated)
  events.on(OrbitEvents.PodsUpdated, handlePodsUpdated)
  events.on(OrbitEvents.DeploymentsUpdated, handleDeploymentsUpdated)
  events.on(OrbitEvents.StatefulSetsUpdated, handleStatefulSetsUpdated)
  events.on(OrbitEvents.DaemonSetsUpdated, handleDaemonSetsUpdated)
  events.on(OrbitEvents.ReplicaSetsUpdated, handleReplicaSetsUpdated)
  events.on(OrbitEvents.JobsUpdated, handleJobsUpdated)
  events.on(OrbitEvents.CronJobsUpdated, handleCronJobsUpdated)
  events.on(OrbitEvents.NodesUpdated, handleNodesUpdated)
  events.on(OrbitEvents.ClustersUpdated, handleClustersUpdated)
  events.on(OrbitEvents.ActiveClusterChanged, handleActiveClusterChanged)
  events.on(OrbitEvents.ErrorOccurred, handleErrorOccurred)
})

onUnmounted(() => {
  events.off(OrbitEvents.EngineConnected, handleEngineConnected)
  events.off(OrbitEvents.NamespacesUpdated, handleNamespacesUpdated)
  events.off(OrbitEvents.PodsUpdated, handlePodsUpdated)
  events.off(OrbitEvents.DeploymentsUpdated, handleDeploymentsUpdated)
  events.off(OrbitEvents.StatefulSetsUpdated, handleStatefulSetsUpdated)
  events.off(OrbitEvents.DaemonSetsUpdated, handleDaemonSetsUpdated)
  events.off(OrbitEvents.ReplicaSetsUpdated, handleReplicaSetsUpdated)
  events.off(OrbitEvents.JobsUpdated, handleJobsUpdated)
  events.off(OrbitEvents.CronJobsUpdated, handleCronJobsUpdated)
  events.off(OrbitEvents.NodesUpdated, handleNodesUpdated)
  events.off(OrbitEvents.ClustersUpdated, handleClustersUpdated)
  events.off(OrbitEvents.ActiveClusterChanged, handleActiveClusterChanged)
  events.off(OrbitEvents.ErrorOccurred, handleErrorOccurred)
})
</script>

<template>
  <AppLayout />
  <Toast />
  <ConfirmDialog />
</template>

<style>
/* Global scrollbar styling matching modern dark theme */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--border-strong);
}
</style>
