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
  status: 'Running' | 'Pending' | 'Failed' | 'Unknown' | 'CrashLoopBackOff' | 'Completed'
  node: string
  restarts: number
  cpu: string
  cpuPct: number
  memory: string
  memoryPct: number
  age: string
  images: string[]
  labels: Record<string, string>
  annotations: Record<string, string>
  ip: string
  nodeIP: string
  controlledBy: string
  qosClass: 'Guaranteed' | 'Burstable' | 'BestEffort'
  containers: PodContainer[]
  events: PodEvent[]
  cpuHistory: number[]
  memoryHistory: number[]
}

// Generate the specific 16 pods mentioned in mockup or typical workloads
const mainPods: PodInfo[] = [
  {
    name: 'api-gateway-7f4c59b6d8-abc12',
    namespace: 'default',
    status: 'Running',
    node: 'ip-10-0-1-24.ec2.internal',
    restarts: 0,
    cpu: '120m',
    cpuPct: 24,
    memory: '256Mi',
    memoryPct: 50,
    age: '7d',
    images: ['kong:3.4', 'redis:7.0-alpine'],
    labels: { app: 'api-gateway', version: 'v1.2.4', tier: 'frontend' },
    annotations: { 'prometheus.io/scrape': 'true', 'prometheus.io/port': '8000' },
    ip: '10.244.1.45',
    nodeIP: '10.0.1.24',
    controlledBy: 'ReplicaSet/api-gateway-7f4c59b6d8',
    qosClass: 'Burstable',
    containers: [
      {
        name: 'kong',
        image: 'kong:3.4',
        status: 'Running',
        ready: '1/1',
        restarts: 0,
        ports: '8000/TCP, 8443/TCP'
      },
      {
        name: 'redis-sidecar',
        image: 'redis:7.0-alpine',
        status: 'Running',
        ready: '1/1',
        restarts: 0,
        ports: '6379/TCP'
      }
    ],
    events: [
      {
        type: 'Normal',
        reason: 'Scheduled',
        message:
          'Successfully assigned default/api-gateway-7f4c59b6d8-abc12 to ip-10-0-1-24.ec2.internal',
        age: '7d'
      },
      {
        type: 'Normal',
        reason: 'Pulled',
        message: 'Container image "kong:3.4" already present on machine',
        age: '7d'
      },
      { type: 'Normal', reason: 'Created', message: 'Created container kong', age: '7d' },
      { type: 'Normal', reason: 'Started', message: 'Started container kong', age: '7d' }
    ],
    cpuHistory: [95, 102, 110, 115, 120, 118, 120],
    memoryHistory: [240, 245, 250, 255, 256, 256, 256]
  },
  {
    name: 'auth-service-589df4fc7c-xyz98',
    namespace: 'default',
    status: 'Running',
    node: 'ip-10-0-2-11.ec2.internal',
    restarts: 2,
    cpu: '45m',
    cpuPct: 9,
    memory: '128Mi',
    memoryPct: 25,
    age: '4d 2h',
    images: ['orbit/auth-service:v1.0.1'],
    labels: { app: 'auth-service', environment: 'production' },
    annotations: { 'vault.hashicorp.com/agent-inject': 'true' },
    ip: '10.244.2.19',
    nodeIP: '10.0.2.11',
    controlledBy: 'ReplicaSet/auth-service-589df4fc7c',
    qosClass: 'Burstable',
    containers: [
      {
        name: 'auth-service',
        image: 'orbit/auth-service:v1.0.1',
        status: 'Running',
        ready: '1/1',
        restarts: 2,
        ports: '8080/TCP'
      }
    ],
    events: [
      {
        type: 'Normal',
        reason: 'Scheduled',
        message:
          'Successfully assigned default/auth-service-589df4fc7c-xyz98 to ip-10-0-2-11.ec2.internal',
        age: '4d'
      },
      {
        type: 'Warning',
        reason: 'Unhealthy',
        message: 'Liveness probe failed: HTTP probe failed with statuscode: 500',
        age: '3d 12h'
      },
      {
        type: 'Normal',
        reason: 'Killing',
        message: 'Container auth-service failed liveness probe, will be restarted',
        age: '3d 12h'
      }
    ],
    cpuHistory: [30, 35, 42, 40, 48, 44, 45],
    memoryHistory: [110, 115, 120, 122, 125, 128, 128]
  },
  {
    name: 'payment-processor-cdf8d4d7-12345',
    namespace: 'default',
    status: 'Pending',
    node: 'ip-10-0-3-15.ec2.internal',
    restarts: 0,
    cpu: '-',
    cpuPct: 0,
    memory: '-',
    memoryPct: 0,
    age: '12m',
    images: ['orbit/payment-processor:v2.1.0-rc3'],
    labels: { app: 'payment-processor', tier: 'backend' },
    annotations: {},
    ip: '-',
    nodeIP: '10.0.3.15',
    controlledBy: 'ReplicaSet/payment-processor-cdf8d4d7',
    qosClass: 'BestEffort',
    containers: [
      {
        name: 'payment-processor',
        image: 'orbit/payment-processor:v2.1.0-rc3',
        status: 'Waiting',
        ready: '0/1',
        restarts: 0
      }
    ],
    events: [
      {
        type: 'Warning',
        reason: 'FailedScheduling',
        message: '0/6 nodes are available: 3 Insufficient cpu, 3 node(s) had untolerated taint.',
        age: '12m'
      },
      {
        type: 'Normal',
        reason: 'TriggeredScaleUp',
        message: 'pod triggered scale-up: [{egress-gateway 0->1}]',
        age: '10m'
      }
    ],
    cpuHistory: [0, 0, 0, 0, 0, 0, 0],
    memoryHistory: [0, 0, 0, 0, 0, 0, 0]
  },
  {
    name: 'notification-worker-0',
    namespace: 'default',
    status: 'CrashLoopBackOff',
    node: 'ip-10-0-2-11.ec2.internal',
    restarts: 14,
    cpu: '15m',
    cpuPct: 3,
    memory: '64Mi',
    memoryPct: 12,
    age: '3h',
    images: ['orbit/notification-worker:v0.8.2'],
    labels: { app: 'notification-worker', controller: 'statefulset' },
    annotations: {},
    ip: '10.244.2.88',
    nodeIP: '10.0.2.11',
    controlledBy: 'StatefulSet/notification-worker',
    qosClass: 'Burstable',
    containers: [
      {
        name: 'worker',
        image: 'orbit/notification-worker:v0.8.2',
        status: 'CrashLoopBackOff',
        ready: '0/1',
        restarts: 14
      }
    ],
    events: [
      {
        type: 'Warning',
        reason: 'BackOff',
        message: 'Back-off restarting failed container worker in pod notification-worker-0_default',
        age: '1m'
      },
      { type: 'Normal', reason: 'Created', message: 'Created container worker', age: '3h' }
    ],
    cpuHistory: [40, 5, 0, 30, 2, 0, 15],
    memoryHistory: [60, 62, 10, 61, 12, 10, 64]
  },
  {
    name: 'prometheus-server-84f98d9c5b-pqr72',
    namespace: 'monitoring',
    status: 'Running',
    node: 'ip-10-0-4-99.ec2.internal',
    restarts: 0,
    cpu: '450m',
    cpuPct: 90,
    memory: '1024Mi',
    memoryPct: 64,
    age: '30d',
    images: ['prom/prometheus:v2.45.0'],
    labels: { app: 'prometheus', component: 'server' },
    annotations: {},
    ip: '10.244.4.102',
    nodeIP: '10.0.4.99',
    controlledBy: 'ReplicaSet/prometheus-server-84f98d9c5b',
    qosClass: 'Guaranteed',
    containers: [
      {
        name: 'prometheus',
        image: 'prom/prometheus:v2.45.0',
        status: 'Running',
        ready: '1/1',
        restarts: 0,
        ports: '9090/TCP'
      }
    ],
    events: [
      {
        type: 'Normal',
        reason: 'Scheduled',
        message:
          'Successfully assigned monitoring/prometheus-server-84f98d9c5b-pqr72 to ip-10-0-4-99.ec2.internal',
        age: '30d'
      }
    ],
    cpuHistory: [410, 420, 435, 440, 448, 445, 450],
    memoryHistory: [1000, 1010, 1020, 1024, 1024, 1024, 1024]
  },
  {
    name: 'coredns-78fcdf6894-mnb12',
    namespace: 'kube-system',
    status: 'Running',
    node: 'ip-10-0-0-12.ec2.internal',
    restarts: 1,
    cpu: '18m',
    cpuPct: 18,
    memory: '32Mi',
    memoryPct: 32,
    age: '45d',
    images: ['registry.k8s.io/dns/k8s-dns-node-cache:1.22.20'],
    labels: { 'k8s-app': 'kube-dns' },
    annotations: {},
    ip: '10.244.0.2',
    nodeIP: '10.0.0.12',
    controlledBy: 'ReplicaSet/coredns-78fcdf6894',
    qosClass: 'Burstable',
    containers: [
      {
        name: 'coredns',
        image: 'registry.k8s.io/dns/k8s-dns-node-cache:1.22.20',
        status: 'Running',
        ready: '1/1',
        restarts: 1,
        ports: '53/UDP, 53/TCP'
      }
    ],
    events: [],
    cpuHistory: [15, 16, 17, 18, 16, 18, 18],
    memoryHistory: [30, 31, 32, 32, 32, 32, 32]
  }
]

// Generate other mock pods to reach exactly 264 pods (236 Running, 18 Pending, 6 Failed, 4 Unknown)
// Currently mainPods has:
// Running: 4 (api-gateway, auth-service, prometheus-server, coredns)
// Pending: 1 (payment-processor)
// Failed (CrashLoopBackOff count as Failed or Running? Let's treat CrashLoopBackOff as Failed status count): 1 (notification-worker-0)
// Unknown: 0
// Let's generate additional pods to match target counts:
// Running: 236 - 4 = 232
// Pending: 18 - 1 = 17
// Failed: 6 - 1 = 5
// Unknown: 4

const generatedPods: PodInfo[] = []

const namespaces = ['default', 'backend', 'frontend', 'monitoring', 'kube-system', 'logging']
const nodes = [
  'ip-10-0-1-24.ec2.internal',
  'ip-10-0-2-11.ec2.internal',
  'ip-10-0-3-15.ec2.internal',
  'ip-10-0-4-99.ec2.internal',
  'ip-10-0-0-12.ec2.internal',
  'ip-10-0-5-56.ec2.internal'
]

// 232 Running
for (let i = 0; i < 232; i++) {
  const ns = namespaces[i % namespaces.length] || 'default'
  const isSystem = ['kube-system', 'monitoring', 'logging'].includes(ns)
  const namePrefix = isSystem ? 'sys' : 'app'
  const name = `${namePrefix}-worker-${i + 100}-${Math.random().toString(36).substring(2, 6)}`
  const node = nodes[i % nodes.length] || 'unknown-node'
  const cpuVal = Math.floor(Math.random() * 80) + 10
  const memVal = Math.floor(Math.random() * 192) + 32

  generatedPods.push({
    name,
    namespace: ns,
    status: 'Running',
    node,
    restarts: Math.random() > 0.96 ? Math.floor(Math.random() * 3) + 1 : 0,
    cpu: `${cpuVal}m`,
    cpuPct: Math.round((cpuVal / 200) * 100),
    memory: `${memVal}Mi`,
    memoryPct: Math.round((memVal / 512) * 100),
    age: `${Math.floor(Math.random() * 14) + 1}d`,
    images: [`docker.io/library/worker-node:v2.1.${i % 4}`],
    labels: { app: 'worker', environment: isSystem ? 'system' : 'production', tier: 'worker' },
    annotations: {},
    ip: `10.244.${i % 5}.${10 + (i % 200)}`,
    nodeIP: `10.0.${i % 6}.10`,
    controlledBy: `ReplicaSet/worker-deployment-${Math.random().toString(36).substring(2, 5)}`,
    qosClass: 'Burstable',
    containers: [
      {
        name: 'worker-con',
        image: `docker.io/library/worker-node:v2.1.${i % 4}`,
        status: 'Running',
        ready: '1/1',
        restarts: 0
      }
    ],
    events: [],
    cpuHistory: Array.from({ length: 7 }, () => cpuVal + Math.floor(Math.random() * 10) - 5),
    memoryHistory: Array.from({ length: 7 }, () => memVal + Math.floor(Math.random() * 16) - 8)
  })
}

// 17 Pending
for (let i = 0; i < 17; i++) {
  const ns = namespaces[i % namespaces.length] || 'default'
  const name = `pending-job-${i + 100}-${Math.random().toString(36).substring(2, 6)}`

  generatedPods.push({
    name,
    namespace: ns,
    status: 'Pending',
    node: 'None (Unscheduled)',
    restarts: 0,
    cpu: '-',
    cpuPct: 0,
    memory: '-',
    memoryPct: 0,
    age: `${i + 2}m`,
    images: [`docker.io/library/batch-job:v1.0.${i % 2}`],
    labels: { app: 'batch-job', schedule: 'daily' },
    annotations: {},
    ip: '-',
    nodeIP: '-',
    controlledBy: `Job/batch-job-${Math.random().toString(36).substring(2, 5)}`,
    qosClass: 'BestEffort',
    containers: [
      {
        name: 'job-con',
        image: `docker.io/library/batch-job:v1.0.${i % 2}`,
        status: 'Waiting',
        ready: '0/1',
        restarts: 0
      }
    ],
    events: [
      {
        type: 'Warning',
        reason: 'FailedScheduling',
        message: '0/6 nodes are available: 6 Insufficient memory',
        age: `${i + 2}m`
      }
    ],
    cpuHistory: [0, 0, 0, 0, 0, 0, 0],
    memoryHistory: [0, 0, 0, 0, 0, 0, 0]
  })
}

// 5 Failed (e.g. Completed or CrashLoopBackOff)
for (let i = 0; i < 5; i++) {
  const ns = namespaces[i % namespaces.length] || 'default'
  const status = i % 2 === 0 ? 'CrashLoopBackOff' : 'Failed'
  const name = `failed-task-${i + 100}-${Math.random().toString(36).substring(2, 6)}`
  const node = nodes[i % nodes.length] || 'unknown-node'
  const restarts = status === 'CrashLoopBackOff' ? 8 : 1

  generatedPods.push({
    name,
    namespace: ns,
    status,
    node,
    restarts,
    cpu: '0m',
    cpuPct: 0,
    memory: '12Mi',
    memoryPct: 2,
    age: `${i + 1}h`,
    images: [`docker.io/library/faulty-task:v0.1`],
    labels: { app: 'faulty-task' },
    annotations: {},
    ip: `10.244.3.${50 + i}`,
    nodeIP: `10.0.${i % 6}.10`,
    controlledBy: `ReplicaSet/faulty-task-${Math.random().toString(36).substring(2, 5)}`,
    qosClass: 'BestEffort',
    containers: [
      {
        name: 'task-con',
        image: 'docker.io/library/faulty-task:v0.1',
        status: 'Terminated',
        ready: '0/1',
        restarts
      }
    ],
    events: [
      {
        type: 'Warning',
        reason: 'BackOff',
        message: 'Back-off restarting failed container task-con',
        age: '4m'
      }
    ],
    cpuHistory: [50, 10, 0, 0, 0, 0, 0],
    memoryHistory: [12, 12, 12, 12, 12, 12, 12]
  })
}

// 4 Unknown
for (let i = 0; i < 4; i++) {
  const ns = namespaces[i % namespaces.length] || 'default'
  const name = `ghost-pod-${i + 100}-${Math.random().toString(36).substring(2, 6)}`
  const node = nodes[i % nodes.length] || 'unknown-node'

  generatedPods.push({
    name,
    namespace: ns,
    status: 'Unknown',
    node,
    restarts: 0,
    cpu: '-',
    cpuPct: 0,
    memory: '-',
    memoryPct: 0,
    age: '2d',
    images: ['docker.io/library/ghost:latest'],
    labels: { app: 'ghost' },
    annotations: {},
    ip: '10.244.5.99',
    nodeIP: `10.0.5.56`,
    controlledBy: 'None',
    qosClass: 'BestEffort',
    containers: [
      {
        name: 'ghost-con',
        image: 'docker.io/library/ghost:latest',
        status: 'Unknown',
        ready: '0/1',
        restarts: 0
      }
    ],
    events: [
      {
        type: 'Warning',
        reason: 'NodeLost',
        message: 'Node ip-10-0-5-56.ec2.internal which host pod was lost',
        age: '10m'
      }
    ],
    cpuHistory: [0, 0, 0, 0, 0, 0, 0],
    memoryHistory: [0, 0, 0, 0, 0, 0, 0]
  })
}

export const mockPods: PodInfo[] = [...mainPods, ...generatedPods]
