import type {
  ClusterInfo,
  ConfigMapInfo,
  CronJobInfo,
  DaemonSetInfo,
  DeploymentInfo,
  JobInfo,
  NamespaceInfo,
  NodeInfo,
  PersistentVolumeClaimInfo,
  PersistentVolumeInfo,
  PodInfo,
  ReplicaSetInfo,
  SecretInfo,
  ServiceInfo,
  StatefulSetInfo,
  StorageClassInfo
} from './kubernetes'

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
  persistentVolumesUpdated: {
    persistent_volumes: PersistentVolumeInfo[]
  }
  persistentVolumeClaimsUpdated: {
    persistent_volume_claims: PersistentVolumeClaimInfo[]
  }
  storageClassesUpdated: {
    storage_classes: StorageClassInfo[]
  }
  errorOccurred: {
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
  PersistentVolumesUpdated: 'persistentVolumesUpdated',
  PersistentVolumeClaimsUpdated: 'persistentVolumeClaimsUpdated',
  StorageClassesUpdated: 'storageClassesUpdated',
  ErrorOccurred: 'errorOccurred'
} as const

export type OrbitEventName = keyof OrbitEventMap

export type OrbitEvent<K extends OrbitEventName = OrbitEventName> = {
  event: K
  data: OrbitEventMap[K]
}
