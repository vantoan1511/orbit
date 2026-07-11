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
}

export type OrbitEventName = keyof OrbitEventMap

export type OrbitEvent<K extends OrbitEventName = OrbitEventName> = {
  event: K
  data: OrbitEventMap[K]
}
