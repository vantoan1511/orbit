import type { PodInfo, ClusterInfo } from './kubernetes'

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
  clustersUpdated: {
    clusters: ClusterInfo[]
  }
  activeClusterChanged: {
    active_cluster_id: string | null
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
  ClustersUpdated: 'clustersUpdated',
  ActiveClusterChanged: 'activeClusterChanged',
  ErrorOccurred: 'errorOccurred'
} as const

export type OrbitEventName = keyof OrbitEventMap

export type OrbitEvent<K extends OrbitEventName = OrbitEventName> = {
  event: K
  data: OrbitEventMap[K]
}
