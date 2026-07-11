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
    namespaces: string[]
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
  ErrorOccurred: 'errorOccurred'
} as const

export type OrbitEventName = keyof OrbitEventMap

export type OrbitEvent<K extends OrbitEventName = OrbitEventName> = {
  event: K
  data: OrbitEventMap[K]
}
