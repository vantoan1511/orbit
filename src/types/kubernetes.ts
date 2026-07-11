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

export interface Replicas {
  current: number
  desired: number
}

export interface DaemonSetReplicas {
  desired: number
  current: number
  ready: number
  upToDate: number
  available: number
}

export interface DeploymentInfo {
  name: string
  namespace: string
  status: string
  replicas: Replicas
  available: number
  upToDate: number
  age: string
  images: string[]
  strategy?: string
  minReadySeconds: number
  revisionHistory?: number
  labels: Record<string, string>
  annotations: Record<string, string>
}

export interface StatefulSetInfo {
  name: string
  namespace: string
  status: string
  replicas: Replicas
  age: string
  images: string[]
  labels: Record<string, string>
  annotations: Record<string, string>
}

export interface DaemonSetInfo {
  name: string
  namespace: string
  status: string
  replicas: DaemonSetReplicas
  age: string
  images: string[]
  labels: Record<string, string>
  annotations: Record<string, string>
}

export interface ReplicaSetInfo {
  name: string
  namespace: string
  status: string
  replicas: Replicas
  age: string
  images: string[]
  labels: Record<string, string>
  annotations: Record<string, string>
}

export interface JobInfo {
  name: string
  namespace: string
  status: string
  completions: string
  duration?: string
  age: string
  images: string[]
  labels: Record<string, string>
  annotations: Record<string, string>
}

export interface CronJobInfo {
  name: string
  namespace: string
  schedule: string
  suspend: boolean
  active: number
  lastSchedule?: string
  age: string
  images: string[]
  labels: Record<string, string>
  annotations: Record<string, string>
}

export type WorkloadInfo =
  DeploymentInfo | StatefulSetInfo | DaemonSetInfo | ReplicaSetInfo | JobInfo | CronJobInfo

export interface NodeInfo {
  name: string
  status: string
  role: string
  version: string
  cpuPct: number
  cpuUsed: string
  cpuTotal: string
  memPct: number
  memUsed: string
  memTotal: string
  podsCount: number
  podsLimit: number
  uptime: string
  labels: string[]
}

export interface ServicePort {
  port: number
  targetPort: number | string
  protocol: string
  nodePort?: number
}

export interface ServiceEvent {
  type: string
  reason: string
  message: string
  age: string
}

export interface ServiceInfo {
  name: string
  namespace: string
  type: string
  clusterIP: string
  externalIP: string
  ports: string
  endpoints: string
  age: string
  sessionAffinity: string
  internalTrafficPolicy?: string
  created: string
  uid: string
  selector: Record<string, string>
  labels: Record<string, string>
  portsList: ServicePort[]
  endpointsList: string[]
  events: ServiceEvent[]
}
