export interface PodContainer {
  name: string
  image: string
  status: string
  ready: string
  restarts: number
  ports?: string
}

export interface PodEvent {
  type: 'Normal' | 'Warning'
  reason: string
  message: string
  age: string
}

export interface PodInfo {
  name: string
  namespace: string
  status: string
  age: string
  cpu?: string
  cpuPct?: number
  memory?: string
  memoryPct?: number
  node?: string
  restarts?: number
  images?: string[]
  labels?: Record<string, string>
  annotations?: Record<string, string>
  ip?: string
  nodeIP?: string
  controlledBy?: string
  qosClass?: 'Guaranteed' | 'Burstable' | 'BestEffort'
  containers?: PodContainer[]
  events?: PodEvent[]
}

export interface ClusterInfo {
  id: string
  name: string
  status: 'healthy' | 'offline' | string
}
