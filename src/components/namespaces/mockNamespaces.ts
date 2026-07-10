export interface NamespaceResourceQuota {
  cpuRequest: string
  cpuLimit: string
  cpuUsed: string
  cpuPercent: number
  memoryRequest: string
  memoryLimit: string
  memoryUsed: string
  memoryPercent: number
}

export interface NamespaceLimitRange {
  type: string
  resource: string
  min: string
  max: string
  default: string
  defaultRequest: string
}

export interface NamespaceInfo {
  name: string
  status: 'Active' | 'Terminating'
  isSystem: boolean
  pods: number
  podSparkline: number[]
  workloads: number
  services: number
  configMaps: number
  secrets: number
  age: string
  labels: Record<string, string>
  uid: string
  created: string
  cpuUsage: string
  cpuPercent: number
  cpuHistory: number[]
  memoryUsage: string
  memoryPercent: number
  memoryHistory: number[]
  annotations: Record<string, string>
  resourceQuota: NamespaceResourceQuota | null
  limitRanges: NamespaceLimitRange[]
}

const mainNamespaces: NamespaceInfo[] = [
  {
    name: 'default',
    status: 'Active',
    isSystem: true,
    pods: 18,
    podSparkline: [14, 15, 16, 17, 18, 18, 18],
    workloads: 6,
    services: 5,
    configMaps: 12,
    secrets: 9,
    age: '18d 4h',
    labels: {
      'kubernetes.io/metadata.name': 'default'
    },
    uid: 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    created: 'May 12, 2025, 10:21 AM',
    cpuUsage: '320m',
    cpuPercent: 16,
    cpuHistory: [280, 290, 305, 310, 315, 320, 320],
    memoryUsage: '512Mi',
    memoryPercent: 25,
    memoryHistory: [480, 490, 500, 505, 510, 512, 512],
    annotations: {
      'kubectl.kubernetes.io/last-applied-configuration': '...'
    },
    resourceQuota: {
      cpuRequest: '2 cores',
      cpuLimit: '4 cores',
      cpuUsed: '2 / 4 cores',
      cpuPercent: 50,
      memoryRequest: '4 GiB',
      memoryLimit: '8 GiB',
      memoryUsed: '4 / 8 GiB',
      memoryPercent: 50
    },
    limitRanges: [
      {
        type: 'Container',
        resource: 'cpu',
        min: '100m',
        max: '2',
        default: '500m',
        defaultRequest: '100m'
      },
      {
        type: 'Container',
        resource: 'memory',
        min: '128Mi',
        max: '2Gi',
        default: '512Mi',
        defaultRequest: '128Mi'
      }
    ]
  },
  {
    name: 'kube-system',
    status: 'Active',
    isSystem: true,
    pods: 72,
    podSparkline: [68, 70, 70, 71, 72, 72, 72],
    workloads: 24,
    services: 10,
    configMaps: 42,
    secrets: 28,
    age: '18d 4h',
    labels: {
      'kubernetes.io/metadata.name': 'kube-system'
    },
    uid: 'b2c3d4e5-f6a7-8901-bcde-f12345678901',
    created: 'May 12, 2025, 10:21 AM',
    cpuUsage: '1200m',
    cpuPercent: 60,
    cpuHistory: [1100, 1120, 1150, 1180, 1200, 1200, 1200],
    memoryUsage: '2.1 GiB',
    memoryPercent: 52,
    memoryHistory: [2000, 2040, 2060, 2080, 2100, 2100, 2100],
    annotations: {},
    resourceQuota: null,
    limitRanges: []
  },
  {
    name: 'kube-public',
    status: 'Active',
    isSystem: true,
    pods: 2,
    podSparkline: [2, 2, 2, 2, 2, 2, 2],
    workloads: 0,
    services: 1,
    configMaps: 1,
    secrets: 0,
    age: '18d 4h',
    labels: {
      'kubernetes.io/metadata.name': 'kube-public'
    },
    uid: 'c3d4e5f6-a7b8-9012-cdef-123456789012',
    created: 'May 12, 2025, 10:21 AM',
    cpuUsage: '5m',
    cpuPercent: 0,
    cpuHistory: [5, 5, 5, 5, 5, 5, 5],
    memoryUsage: '12Mi',
    memoryPercent: 1,
    memoryHistory: [12, 12, 12, 12, 12, 12, 12],
    annotations: {},
    resourceQuota: null,
    limitRanges: []
  },
  {
    name: 'kube-node-lease',
    status: 'Active',
    isSystem: true,
    pods: 2,
    podSparkline: [2, 2, 2, 2, 2, 2, 2],
    workloads: 0,
    services: 1,
    configMaps: 2,
    secrets: 0,
    age: '18d 4h',
    labels: {
      'kubernetes.io/metadata.name': 'kube-node-lease'
    },
    uid: 'd4e5f6a7-b8c9-0123-defa-234567890123',
    created: 'May 12, 2025, 10:21 AM',
    cpuUsage: '3m',
    cpuPercent: 0,
    cpuHistory: [3, 3, 3, 3, 3, 3, 3],
    memoryUsage: '8Mi',
    memoryPercent: 0,
    memoryHistory: [8, 8, 8, 8, 8, 8, 8],
    annotations: {},
    resourceQuota: null,
    limitRanges: []
  },
  {
    name: 'backend',
    status: 'Active',
    isSystem: false,
    pods: 45,
    podSparkline: [38, 40, 42, 43, 44, 45, 45],
    workloads: 12,
    services: 8,
    configMaps: 15,
    secrets: 11,
    age: '12d 10h',
    labels: {
      app: 'backend',
      team: 'platform',
      env: 'production',
      tier: 'backend'
    },
    uid: 'f3a7c9e2-2b1d-4cce-9a6a-0f2b8e7d9c11',
    created: 'May 12, 2025, 10:21 AM',
    cpuUsage: '680m',
    cpuPercent: 34,
    cpuHistory: [600, 620, 640, 660, 670, 680, 680],
    memoryUsage: '1.25 GiB',
    memoryPercent: 41,
    memoryHistory: [1100, 1150, 1200, 1220, 1240, 1250, 1250],
    annotations: {
      'kubectl.kubernetes.io/last-applied-configuration': '...',
      'meta.helm.sh/release-name': 'backend-services'
    },
    resourceQuota: {
      cpuRequest: '2 cores',
      cpuLimit: '4 cores',
      cpuUsed: '2 / 4 cores',
      cpuPercent: 50,
      memoryRequest: '4 GiB',
      memoryLimit: '8 GiB',
      memoryUsed: '4 / 8 GiB',
      memoryPercent: 50
    },
    limitRanges: [
      {
        type: 'Container',
        resource: 'cpu',
        min: '50m',
        max: '4',
        default: '200m',
        defaultRequest: '50m'
      },
      {
        type: 'Container',
        resource: 'memory',
        min: '64Mi',
        max: '4Gi',
        default: '256Mi',
        defaultRequest: '64Mi'
      }
    ]
  },
  {
    name: 'frontend',
    status: 'Active',
    isSystem: false,
    pods: 28,
    podSparkline: [24, 25, 26, 27, 28, 28, 28],
    workloads: 8,
    services: 6,
    configMaps: 9,
    secrets: 6,
    age: '12d 10h',
    labels: {
      app: 'frontend',
      team: 'platform'
    },
    uid: 'e5a7b2c1-3d4e-5f6a-7b8c-9d0e1f2a3b4c',
    created: 'May 12, 2025, 10:21 AM',
    cpuUsage: '380m',
    cpuPercent: 19,
    cpuHistory: [340, 350, 360, 370, 375, 380, 380],
    memoryUsage: '820Mi',
    memoryPercent: 32,
    memoryHistory: [780, 790, 800, 808, 815, 820, 820],
    annotations: {
      'meta.helm.sh/release-name': 'frontend-services'
    },
    resourceQuota: {
      cpuRequest: '1 core',
      cpuLimit: '2 cores',
      cpuUsed: '0.38 / 2 cores',
      cpuPercent: 19,
      memoryRequest: '2 GiB',
      memoryLimit: '4 GiB',
      memoryUsed: '0.8 / 4 GiB',
      memoryPercent: 20
    },
    limitRanges: []
  },
  {
    name: 'monitoring',
    status: 'Active',
    isSystem: false,
    pods: 35,
    podSparkline: [30, 32, 33, 34, 35, 35, 35],
    workloads: 10,
    services: 7,
    configMaps: 18,
    secrets: 14,
    age: '18d 4h',
    labels: {
      app: 'monitoring',
      team: 'observability'
    },
    uid: 'a1b2c3d4-4e5f-6a7b-8c9d-0e1f2a3b4c5d',
    created: 'May 12, 2025, 10:21 AM',
    cpuUsage: '950m',
    cpuPercent: 47,
    cpuHistory: [880, 900, 920, 935, 945, 950, 950],
    memoryUsage: '3.2 GiB',
    memoryPercent: 80,
    memoryHistory: [3000, 3050, 3100, 3150, 3180, 3200, 3200],
    annotations: {},
    resourceQuota: null,
    limitRanges: []
  },
  {
    name: 'logging',
    status: 'Active',
    isSystem: false,
    pods: 22,
    podSparkline: [18, 19, 20, 21, 22, 22, 22],
    workloads: 6,
    services: 4,
    configMaps: 11,
    secrets: 7,
    age: '18d 4h',
    labels: {
      app: 'logging',
      team: 'observability'
    },
    uid: 'b2c3d4e5-5f6a-7b8c-9d0e-1f2a3b4c5d6e',
    created: 'May 12, 2025, 10:21 AM',
    cpuUsage: '420m',
    cpuPercent: 21,
    cpuHistory: [380, 390, 400, 410, 415, 420, 420],
    memoryUsage: '1.5 GiB',
    memoryPercent: 37,
    memoryHistory: [1400, 1430, 1460, 1470, 1490, 1500, 1500],
    annotations: {},
    resourceQuota: null,
    limitRanges: []
  },
  {
    name: 'database',
    status: 'Active',
    isSystem: false,
    pods: 16,
    podSparkline: [14, 14, 15, 15, 16, 16, 16],
    workloads: 5,
    services: 3,
    configMaps: 6,
    secrets: 6,
    age: '18d 4h',
    labels: {
      app: 'database'
    },
    uid: 'c3d4e5f6-6a7b-8c9d-0e1f-2a3b4c5d6e7f',
    created: 'May 12, 2025, 10:21 AM',
    cpuUsage: '820m',
    cpuPercent: 41,
    cpuHistory: [760, 780, 800, 810, 818, 820, 820],
    memoryUsage: '4.8 GiB',
    memoryPercent: 60,
    memoryHistory: [4600, 4680, 4720, 4760, 4780, 4800, 4800],
    annotations: {},
    resourceQuota: null,
    limitRanges: []
  },
  {
    name: 'cache',
    status: 'Active',
    isSystem: false,
    pods: 12,
    podSparkline: [10, 10, 11, 11, 12, 12, 12],
    workloads: 4,
    services: 2,
    configMaps: 4,
    secrets: 2,
    age: '9d 1h',
    labels: {
      app: 'cache'
    },
    uid: 'd4e5f6a7-7b8c-9d0e-1f2a-3b4c5d6e7f8a',
    created: 'May 21, 2025, 09:15 AM',
    cpuUsage: '260m',
    cpuPercent: 13,
    cpuHistory: [230, 240, 250, 255, 258, 260, 260],
    memoryUsage: '980Mi',
    memoryPercent: 24,
    memoryHistory: [920, 940, 958, 965, 975, 980, 980],
    annotations: {},
    resourceQuota: null,
    limitRanges: []
  },
  {
    name: 'jobs',
    status: 'Active',
    isSystem: false,
    pods: 8,
    podSparkline: [5, 6, 7, 8, 8, 8, 8],
    workloads: 3,
    services: 1,
    configMaps: 3,
    secrets: 2,
    age: '5d 2h',
    labels: {
      app: 'jobs'
    },
    uid: 'e5f6a7b8-8c9d-0e1f-2a3b-4c5d6e7f8a9b',
    created: 'May 25, 2025, 08:30 AM',
    cpuUsage: '140m',
    cpuPercent: 7,
    cpuHistory: [100, 110, 120, 130, 135, 140, 140],
    memoryUsage: '320Mi',
    memoryPercent: 10,
    memoryHistory: [280, 290, 300, 308, 315, 320, 320],
    annotations: {},
    resourceQuota: null,
    limitRanges: []
  },
  {
    name: 'dev',
    status: 'Terminating',
    isSystem: false,
    pods: 6,
    podSparkline: [10, 9, 8, 7, 6, 6, 6],
    workloads: 2,
    services: 2,
    configMaps: 3,
    secrets: 1,
    age: '2h 15m',
    labels: {
      env: 'dev'
    },
    uid: 'f6a7b8c9-9d0e-1f2a-3b4c-5d6e7f8a9b0c',
    created: 'Jun 10, 2025, 02:00 PM',
    cpuUsage: '80m',
    cpuPercent: 4,
    cpuHistory: [150, 130, 110, 95, 85, 80, 80],
    memoryUsage: '180Mi',
    memoryPercent: 5,
    memoryHistory: [320, 290, 260, 230, 200, 180, 180],
    annotations: {},
    resourceQuota: null,
    limitRanges: []
  },
  {
    name: 'old',
    status: 'Terminating',
    isSystem: false,
    pods: 0,
    podSparkline: [3, 2, 1, 1, 0, 0, 0],
    workloads: 0,
    services: 0,
    configMaps: 0,
    secrets: 0,
    age: '1h 32m',
    labels: {
      env: 'old'
    },
    uid: 'a7b8c9d0-0e1f-2a3b-4c5d-6e7f8a9b0c1d',
    created: 'Jun 10, 2025, 03:00 PM',
    cpuUsage: '0m',
    cpuPercent: 0,
    cpuHistory: [20, 15, 5, 0, 0, 0, 0],
    memoryUsage: '0Mi',
    memoryPercent: 0,
    memoryHistory: [80, 50, 20, 0, 0, 0, 0],
    annotations: {},
    resourceQuota: null,
    limitRanges: []
  },
  {
    name: 'test',
    status: 'Active',
    isSystem: false,
    pods: 3,
    podSparkline: [2, 2, 3, 3, 3, 3, 3],
    workloads: 1,
    services: 1,
    configMaps: 1,
    secrets: 0,
    age: '2d 6h',
    labels: {
      env: 'test'
    },
    uid: 'b8c9d0e1-1f2a-3b4c-5d6e-7f8a9b0c1d2e',
    created: 'Jun 8, 2025, 11:00 AM',
    cpuUsage: '40m',
    cpuPercent: 2,
    cpuHistory: [30, 32, 35, 38, 40, 40, 40],
    memoryUsage: '80Mi',
    memoryPercent: 2,
    memoryHistory: [60, 65, 70, 75, 78, 80, 80],
    annotations: {},
    resourceQuota: null,
    limitRanges: []
  }
]

// Generate remaining namespaces to reach 38 total (14 named above, need 24 more)
const generatedNamespaces: NamespaceInfo[] = []

const appNames = [
  'api-gateway',
  'auth',
  'payments',
  'notifications',
  'search',
  'analytics',
  'reporting',
  'scheduler',
  'worker',
  'ingress',
  'proxy',
  'queue',
  'events',
  'audit',
  'metrics',
  'tracing',
  'ci',
  'staging',
  'release',
  'sandbox',
  'tools',
  'shared',
  'platform',
  'infra'
]

const teamNames = ['backend', 'frontend', 'platform', 'data', 'infra', 'security']

for (let i = 0; i < 24; i++) {
  const appName = appNames[i] ?? `app-${i}`
  const team = teamNames[i % teamNames.length] ?? 'platform'
  const podCount = Math.floor(Math.random() * 20) + 2

  generatedNamespaces.push({
    name: appName,
    status: 'Active',
    isSystem: false,
    pods: podCount,
    podSparkline: Array.from({ length: 7 }, (_, j) =>
      Math.max(0, podCount - (6 - j) + Math.floor(Math.random() * 3) - 1)
    ),
    workloads: Math.floor(podCount / 4),
    services: Math.floor(podCount / 5) + 1,
    configMaps: Math.floor(Math.random() * 8) + 1,
    secrets: Math.floor(Math.random() * 6) + 1,
    age: `${Math.floor(Math.random() * 20) + 1}d`,
    labels: {
      app: appName,
      team
    },
    uid: `${i.toString(16).padStart(8, '0')}-abcd-efab-cdef-${Math.random().toString(16).substring(2, 14).padEnd(12, '0')}`,
    created: 'May 12, 2025, 10:21 AM',
    cpuUsage: `${Math.floor(Math.random() * 400) + 50}m`,
    cpuPercent: Math.floor(Math.random() * 50) + 5,
    cpuHistory: Array.from({ length: 7 }, () => Math.floor(Math.random() * 400) + 50),
    memoryUsage: `${Math.floor(Math.random() * 800) + 100}Mi`,
    memoryPercent: Math.floor(Math.random() * 60) + 10,
    memoryHistory: Array.from({ length: 7 }, () => Math.floor(Math.random() * 800) + 100),
    annotations: {},
    resourceQuota: null,
    limitRanges: []
  })
}

export const mockNamespaces: NamespaceInfo[] = [...mainNamespaces, ...generatedNamespaces]
