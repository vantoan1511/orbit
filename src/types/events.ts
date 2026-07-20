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
} from './kubernetes'

export interface ComponentVersion {
  version: string
  url: string
}

export interface UpdateManifest {
  version: string
  url: string
}

export interface OrbitEventMap {
  engineConnected: {
    status: 'ready' | 'error'
    message: string
  }
  ping: {
    message: string
  }
  pong: {
    reply: string
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
  resourceUpdated: {
    kind: string
    action: 'Applied' | 'Deleted'
    data: ServiceInfo | DeploymentInfo | PodInfo
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
}

export const OrbitEvents = {
  EngineConnected: 'engineConnected',
  Ping: 'ping',
  Pong: 'pong',
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
  NodesUpdated: 'nodesUpdated',
  ServicesUpdated: 'servicesUpdated',
  ConfigMapsUpdated: 'configMapsUpdated',
  SecretsUpdated: 'secretsUpdated',
  EventsUpdated: 'eventsUpdated',
  PersistentVolumesUpdated: 'persistentVolumesUpdated',
  PersistentVolumeClaimsUpdated: 'persistentVolumeClaimsUpdated',
  StorageClassesUpdated: 'storageClassesUpdated',
  PoliciesUpdated: 'policiesUpdated',
  ResourceUpdated: 'resourceUpdated',
  PodMetricsUpdated: 'podMetricsUpdated',
  ErrorOccurred: 'errorOccurred',
  LogLineReceived: 'logLineReceived',
  UpdateCheckFinished: 'updateCheckFinished',
  UpdateDownloadProgress: 'updateDownloadProgress',
  UpdateReady: 'updateReady',
  CommandSucceeded: 'commandSucceeded'
} as const

export type OrbitEventName = keyof OrbitEventMap

export type OrbitEvent<K extends OrbitEventName = OrbitEventName> = {
  event: K
  data: OrbitEventMap[K]
}
