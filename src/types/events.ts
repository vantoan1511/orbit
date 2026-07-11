import type { PodInfo } from './kubernetes'

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
}

export type OrbitEventName = keyof OrbitEventMap

export type OrbitEvent<K extends OrbitEventName = OrbitEventName> = {
  event: K
  data: OrbitEventMap[K]
}
