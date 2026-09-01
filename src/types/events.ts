import type {
  ClusterInfo,
  ConfigMapInfo,
  CronJobInfo,
  DaemonSetInfo,
  DeploymentInfo,
  EventInfo,
  IngressInfo,
  JobInfo,
  KubernetesAction,
  KubernetesResourceKind,
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
} from './kubernetes'

import type { UserProfileInfo } from './profile'
import type { OrbitConfig } from './settings'

export interface ComponentVersion {
  version: string
  url: string
}

export interface UpdateManifest {
  version: string
  url: string
  release_notes?: string
}

export type KubernetesResourceInfo =
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

export interface ResourceUpdateItem {
  action: KubernetesAction
  data: KubernetesResourceInfo
}

export interface OrbitEventMap {
  engineConnected: {
    status: 'ready' | 'error'
    message: string
  }
  engineTimeout: Record<string, never>
  ping: {
    message: string
  }
  pong: {
    reply: string
  }
  appSettingsUpdated: {
    settings: OrbitConfig
  }
  userProfileUpdated: {
    profile: UserProfileInfo
  }
  namespacesUpdated: {
    namespaces: NamespaceInfo[]
  }
  podsUpdated: {
    pods: PodInfo[]
  }
  deploymentsUpdated: {
    deployments: DeploymentInfo[]
  }
  statefulSetsUpdated: {
    stateful_sets: StatefulSetInfo[]
  }
  daemonSetsUpdated: {
    daemon_sets: DaemonSetInfo[]
  }
  replicaSetsUpdated: {
    replica_sets: ReplicaSetInfo[]
  }
  jobsUpdated: {
    jobs: JobInfo[]
  }
  cronJobsUpdated: {
    cron_jobs: CronJobInfo[]
  }
  clustersUpdated: {
    clusters: ClusterInfo[]
  }
  activeClusterChanged: {
    active_cluster_id: string | null
  }
  nodesUpdated: {
    nodes: NodeInfo[]
  }
  servicesUpdated: {
    services: ServiceInfo[]
  }
  ingressesUpdated: {
    ingresses: IngressInfo[]
  }
  configMapsUpdated: {
    config_maps: ConfigMapInfo[]
  }
  secretsUpdated: {
    secrets: SecretInfo[]
  }
  eventsUpdated: {
    events: EventInfo[]
  }
  persistentVolumesUpdated: {
    persistent_volumes: PersistentVolumeInfo[]
  }
  persistentVolumeClaimsUpdated: {
    persistent_volume_claims: PersistentVolumeClaimInfo[]
  }
  storageClassesUpdated: {
    storage_classes: StorageClassInfo[]
  }
  policiesUpdated: {
    policies: PolicyInfo[]
  }
  resourceBatchUpdated: {
    kind: KubernetesResourceKind | string
    updates: ResourceUpdateItem[]
  }
  podMetricsUpdated: {
    metrics: Array<{ name: string; namespace: string; cpu: string; memory: string }>
  }
  errorOccurred: {
    message: string
  }
  logLineReceived: {
    pod: string
    container: string
    line: string
  }
  logLinesChunkReceived: {
    pod: string
    container: string
    lines: string[]
  }
  updateCheckFinished: {
    has_update: boolean
    manifest: UpdateManifest
  }
  updateDownloadProgress: {
    component: string
    progress_percentage: number
  }
  updateReady: {
    component: string
  }
  commandSucceeded: {
    message: string
  }
  resourceRawData: {
    kind: string
    name: string
    data: Record<string, unknown>
  }
  portForwardStarted: {
    id: string
    namespace: string
    kind: string
    name: string
    localPort: number
    remotePort: number
  }
  portForwardStopped: {
    id: string
  }
}

/** Sentinel value for the tailLines IPC field meaning "fetch all log lines". */
export const TAIL_ALL_LINES = -1 as const

export const OrbitEvents = {
  EngineConnected: 'engineConnected',
  EngineTimeout: 'engineTimeout',
  Ping: 'ping',
  Pong: 'pong',
  AppSettingsUpdated: 'appSettingsUpdated',
  NamespacesUpdated: 'namespacesUpdated',
  PodsUpdated: 'podsUpdated',
  DeploymentsUpdated: 'deploymentsUpdated',
  StatefulSetsUpdated: 'statefulSetsUpdated',
  DaemonSetsUpdated: 'daemonSetsUpdated',
  ReplicaSetsUpdated: 'replicaSetsUpdated',
  JobsUpdated: 'jobsUpdated',
  CronJobsUpdated: 'cronJobsUpdated',
  ClustersUpdated: 'clustersUpdated',
  ActiveClusterChanged: 'activeClusterChanged',
  UserProfileUpdated: 'userProfileUpdated',
  NodesUpdated: 'nodesUpdated',
  ServicesUpdated: 'servicesUpdated',
  IngressesUpdated: 'ingressesUpdated',
  ConfigMapsUpdated: 'configMapsUpdated',
  SecretsUpdated: 'secretsUpdated',
  EventsUpdated: 'eventsUpdated',
  PersistentVolumesUpdated: 'persistentVolumesUpdated',
  PersistentVolumeClaimsUpdated: 'persistentVolumeClaimsUpdated',
  StorageClassesUpdated: 'storageClassesUpdated',
  PoliciesUpdated: 'policiesUpdated',
  ResourceBatchUpdated: 'resourceBatchUpdated',
  PodMetricsUpdated: 'podMetricsUpdated',
  ErrorOccurred: 'errorOccurred',
  LogLineReceived: 'logLineReceived',
  LogLinesChunkReceived: 'logLinesChunkReceived',
  UpdateCheckFinished: 'updateCheckFinished',
  UpdateDownloadProgress: 'updateDownloadProgress',
  UpdateReady: 'updateReady',
  CommandSucceeded: 'commandSucceeded',
  ResourceRawData: 'resourceRawData',
  PortForwardStarted: 'portForwardStarted',
  PortForwardStopped: 'portForwardStopped'
} as const

export type OrbitEventName = keyof OrbitEventMap

export type OrbitEvent<K extends OrbitEventName = OrbitEventName> = {
  event: K
  data: OrbitEventMap[K]
}
