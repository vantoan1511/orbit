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

export interface UsedByPod {
  name: string
  status: 'Running' | 'Pending' | 'Failed' | 'Completed' | string
}

export interface ConfigMapInfo {
  name: string
  namespace: string
  labels: Record<string, string>
  annotations: number
  created: string
  age: string
  resourceVersion: string
  immutable: boolean
  keysCount: number
  size: string
  mountedPods: number
  usedBy: UsedByPod[]
  data: Record<string, string>
}

export interface SecretInfo {
  name: string
  namespace: string
  labels: Record<string, string>
  annotations: number
  type: string
  created: string
  age: string
  resourceVersion: string
  immutable: boolean
  keysCount: number
  size: string
  mountedPods: number
  usedBy: UsedByPod[]
  data: Record<string, string>
}

export interface PersistentVolumeInfo {
  name: string
  capacity: string
  accessMode: string
  reclaimPolicy: string
  status: 'Bound' | 'Available' | 'Released' | 'Failed' | string
  storageClass: string
  age: string
  volumeMode: 'Filesystem' | 'Block' | string
  reason?: string
}

export interface PersistentVolumeClaimInfo {
  name: string
  namespace: string
  status: 'Bound' | 'Lost' | 'Pending' | string
  volume: string
  capacity: string
  accessMode: string
  storageClass: string
  age: string
}

export interface StorageClassInfo {
  name: string
  provisioner: string
  reclaimPolicy: string
  volumeBindingMode: string
  allowVolumeExpansion: boolean
  age: string
}

export interface NamespaceInfo {
  name: string
  status: string
  isSystem: boolean
  age: string
  labels: Record<string, string>
  annotations: Record<string, string>
  uid: string
  created: string
}

export interface EventInfo {
  uid: string
  time: string
  type: string
  reason: string
  objectName: string
  objectKind: string
  message: string
  namespace: string
  source: string
  firstSeen: string
  lastSeen: string
  count: number
  labels: Record<string, string>
  annotations: Record<string, string>
}

export interface PolicyInfo {
  uid: string
  name: string
  type: string
  scope: string
  namespace: string
  status: string
  mode: string
  violations: number
  lastUpdated: string
  description: string
  rules: string
}

