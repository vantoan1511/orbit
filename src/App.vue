<script setup lang="ts">
import NotificationDrawer from '@/components/layout/NotificationDrawer.vue'
import ProfileDrawer from '@/components/layout/ProfileDrawer.vue'
import UpdaterDialog from '@/components/UpdaterDialog.vue'
import UpdaterNotifications from '@/components/UpdaterNotifications.vue'
import { app, events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { useNotificationStore } from '@/stores/notificationStore'
import { useProfileStore } from '@/stores/profileStore'
import { useSettingsStore } from '@/stores/settingsStore'
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
import type { UserProfileInfo } from '@/types/profile'
import type { OrbitConfig } from '@/types/settings'
import ConfirmDialog from 'primevue/confirmdialog'
import DynamicDialog from 'primevue/dynamicdialog'
import Toast from 'primevue/toast'
import { useConfirm } from 'primevue/useconfirm'
import { useToast } from 'primevue/usetoast'
import { onMounted, onUnmounted } from 'vue'
import AppLayout from './components/layout/AppLayout.vue'

const k8sStore = useKubernetesStore()
const notificationStore = useNotificationStore()
const profileStore = useProfileStore()
const settingsStore = useSettingsStore()
const toast = useToast()
const confirm = useConfirm()

const handleEngineConnected = (payload: { status: 'ready' | 'error'; message: string }) => {
  if (payload.status === 'ready') {
    k8sStore.setEngineReady(true)
    profileStore.fetchProfile()
    settingsStore.fetchSettings()
  } else {
    k8sStore.setEngineReady(false)
    notificationStore.addNotification({
      title: 'Engine Error',
      message: payload.message || 'Failed to connect to backend engine',
      severity: 'error',
      category: 'system'
    })
  }
}

const handleAppSettingsUpdated = (payload: { settings: OrbitConfig }) => {
  settingsStore.setSettings(payload.settings)
}

const handleUserProfileUpdated = (payload: { profile: UserProfileInfo }) => {
  profileStore.setProfile(payload.profile)
}

const handleNamespacesUpdated = (payload: { namespaces: NamespaceInfo[] }) => {
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

const handleServicesUpdated = (payload: { services: ServiceInfo[] }) => {
  k8sStore.setServices(payload.services)
}

const handleIngressesUpdated = (payload: { ingresses: IngressInfo[] }) => {
  k8sStore.setIngresses(payload.ingresses)
}

const handleConfigMapsUpdated = (payload: { config_maps: ConfigMapInfo[] }) => {
  k8sStore.setConfigMaps(payload.config_maps)
}

const handleSecretsUpdated = (payload: { secrets: SecretInfo[] }) => {
  k8sStore.setSecrets(payload.secrets)
}

const handleEventsUpdated = (payload: { events: EventInfo[] }) => {
  k8sStore.setEvents(payload.events)
}

const handlePersistentVolumesUpdated = (payload: {
  persistent_volumes: PersistentVolumeInfo[]
}) => {
  k8sStore.setPersistentVolumes(payload.persistent_volumes)
}

const handlePersistentVolumeClaimsUpdated = (payload: {
  persistent_volume_claims: PersistentVolumeClaimInfo[]
}) => {
  k8sStore.setPersistentVolumeClaims(payload.persistent_volume_claims)
}

const handleStorageClassesUpdated = (payload: { storage_classes: StorageClassInfo[] }) => {
  k8sStore.setStorageClasses(payload.storage_classes)
}

const handleClustersUpdated = (payload: { clusters: ClusterInfo[] }) => {
  k8sStore.setClusters(payload.clusters)
}

const handlePoliciesUpdated = (payload: { policies: PolicyInfo[] }) => {
  k8sStore.setPolicies(payload.policies)
}

const handleActiveClusterChanged = (payload: { active_cluster_id: string | null }) => {
  k8sStore.setActiveClusterId(payload.active_cluster_id)
}

let lastConnectionErrorTime = 0

const handleErrorOccurred = (payload: { message: string }) => {
  k8sStore.resetAllLoading()

  const isConnectionError =
    payload.message.includes('client error (Connect)') ||
    payload.message.includes('error trying to connect') ||
    payload.message.toLowerCase().includes('connection refused')

  let displayMessage = payload.message

  if (isConnectionError) {
    const now = Date.now()
    if (now - lastConnectionErrorTime < 5000) {
      return // Ignore duplicate connection error within 5 seconds
    }
    lastConnectionErrorTime = now
    displayMessage = 'Connection not ready. Please verify your cluster status.'
  }

  toast.add({
    severity: 'error',
    summary: 'Error',
    detail: displayMessage,
    life: 5000
  })
  notificationStore.addNotification({
    title: 'Error',
    message: displayMessage,
    severity: 'error',
    category: 'system'
  })
}

const handleCommandSucceeded = (payload: { message: string }) => {
  toast.add({
    severity: 'success',
    summary: 'Success',
    detail: payload.message,
    life: 3000
  })
  notificationStore.addNotification({
    title: 'Command Succeeded',
    message: payload.message,
    severity: 'success',
    category: 'command'
  })
}

const handleEngineTimeout = () => {
  confirm.require({
    header: 'Connection Timeout',
    message: 'Cannot connect to Orbit Engine. Please restart the application.',
    icon: 'pi pi-exclamation-triangle',
    rejectProps: {
      style: { display: 'none' }
    },
    acceptProps: {
      label: 'Restart',
      severity: 'primary'
    },
    accept: () => {
      app.restartProcess()
    }
  })
}

const handleContextMenu = (e: MouseEvent) => {
  const target = e.target as HTMLElement | null
  if (!target) {
    e.preventDefault()
    return
  }

  const isEditable =
    (target.tagName === 'INPUT' &&
      ['text', 'search', 'password', 'email', 'number', 'url'].includes(
        (target as HTMLInputElement).type
      )) ||
    target.tagName === 'TEXTAREA' ||
    target.isContentEditable

  if (!isEditable) {
    e.preventDefault()
  }
}

onMounted(() => {
  // Disable default browser context menu globally for non-input elements
  window.addEventListener('contextmenu', handleContextMenu)

  events.on(OrbitEvents.EngineConnected, handleEngineConnected)
  events.on(OrbitEvents.AppSettingsUpdated, handleAppSettingsUpdated)
  events.on(OrbitEvents.NamespacesUpdated, handleNamespacesUpdated)
  events.on(OrbitEvents.PodsUpdated, handlePodsUpdated)
  events.on(OrbitEvents.DeploymentsUpdated, handleDeploymentsUpdated)
  events.on(OrbitEvents.StatefulSetsUpdated, handleStatefulSetsUpdated)
  events.on(OrbitEvents.DaemonSetsUpdated, handleDaemonSetsUpdated)
  events.on(OrbitEvents.ReplicaSetsUpdated, handleReplicaSetsUpdated)
  events.on(OrbitEvents.JobsUpdated, handleJobsUpdated)
  events.on(OrbitEvents.CronJobsUpdated, handleCronJobsUpdated)
  events.on(OrbitEvents.NodesUpdated, handleNodesUpdated)
  events.on(OrbitEvents.ServicesUpdated, handleServicesUpdated)
  events.on(OrbitEvents.IngressesUpdated, handleIngressesUpdated)
  events.on(OrbitEvents.ConfigMapsUpdated, handleConfigMapsUpdated)
  events.on(OrbitEvents.SecretsUpdated, handleSecretsUpdated)
  events.on(OrbitEvents.EventsUpdated, handleEventsUpdated)
  events.on(OrbitEvents.PersistentVolumesUpdated, handlePersistentVolumesUpdated)
  events.on(OrbitEvents.PersistentVolumeClaimsUpdated, handlePersistentVolumeClaimsUpdated)
  events.on(OrbitEvents.StorageClassesUpdated, handleStorageClassesUpdated)
  events.on(OrbitEvents.PoliciesUpdated, handlePoliciesUpdated)
  events.on(OrbitEvents.ClustersUpdated, handleClustersUpdated)
  events.on(OrbitEvents.ActiveClusterChanged, handleActiveClusterChanged)
  events.on(OrbitEvents.UserProfileUpdated, handleUserProfileUpdated)
  events.on(OrbitEvents.ErrorOccurred, handleErrorOccurred)
  events.on(OrbitEvents.CommandSucceeded, handleCommandSucceeded)
  events.on(OrbitEvents.EngineTimeout, handleEngineTimeout)
})

onUnmounted(() => {
  window.removeEventListener('contextmenu', handleContextMenu)

  events.off(OrbitEvents.EngineConnected, handleEngineConnected)
  events.off(OrbitEvents.AppSettingsUpdated, handleAppSettingsUpdated)
  events.off(OrbitEvents.NamespacesUpdated, handleNamespacesUpdated)
  events.off(OrbitEvents.PodsUpdated, handlePodsUpdated)
  events.off(OrbitEvents.DeploymentsUpdated, handleDeploymentsUpdated)
  events.off(OrbitEvents.StatefulSetsUpdated, handleStatefulSetsUpdated)
  events.off(OrbitEvents.DaemonSetsUpdated, handleDaemonSetsUpdated)
  events.off(OrbitEvents.ReplicaSetsUpdated, handleReplicaSetsUpdated)
  events.off(OrbitEvents.JobsUpdated, handleJobsUpdated)
  events.off(OrbitEvents.CronJobsUpdated, handleCronJobsUpdated)
  events.off(OrbitEvents.NodesUpdated, handleNodesUpdated)
  events.off(OrbitEvents.ServicesUpdated, handleServicesUpdated)
  events.off(OrbitEvents.IngressesUpdated, handleIngressesUpdated)
  events.off(OrbitEvents.ConfigMapsUpdated, handleConfigMapsUpdated)
  events.off(OrbitEvents.SecretsUpdated, handleSecretsUpdated)
  events.off(OrbitEvents.EventsUpdated, handleEventsUpdated)
  events.off(OrbitEvents.PersistentVolumesUpdated, handlePersistentVolumesUpdated)
  events.off(OrbitEvents.PersistentVolumeClaimsUpdated, handlePersistentVolumeClaimsUpdated)
  events.off(OrbitEvents.StorageClassesUpdated, handleStorageClassesUpdated)
  events.off(OrbitEvents.PoliciesUpdated, handlePoliciesUpdated)
  events.off(OrbitEvents.ClustersUpdated, handleClustersUpdated)
  events.off(OrbitEvents.ActiveClusterChanged, handleActiveClusterChanged)
  events.off(OrbitEvents.UserProfileUpdated, handleUserProfileUpdated)
  events.off(OrbitEvents.ErrorOccurred, handleErrorOccurred)
  events.off(OrbitEvents.CommandSucceeded, handleCommandSucceeded)
  events.off(OrbitEvents.EngineTimeout, handleEngineTimeout)
})
</script>

<template>
  <AppLayout />
  <Toast />
  <ConfirmDialog />
  <DynamicDialog />
  <UpdaterNotifications />
  <UpdaterDialog />
  <NotificationDrawer />
  <ProfileDrawer />
</template>
