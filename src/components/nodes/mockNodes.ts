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

export const mockNodes: NodeInfo[] = [
  {
    name: 'prod-node-master-0',
    status: 'Ready',
    role: 'control-plane',
    version: 'v1.33.2',
    cpuPct: 35,
    cpuUsed: '2.8',
    cpuTotal: '8',
    memPct: 52,
    memUsed: '16.6',
    memTotal: '32',
    podsCount: 28,
    podsLimit: 110,
    uptime: '18d 4h',
    labels: ['os=linux', 'role=control-plane']
  },
  {
    name: 'prod-node-worker-01',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 62,
    cpuUsed: '9.9',
    cpuTotal: '16',
    memPct: 78,
    memUsed: '24.9',
    memTotal: '32',
    podsCount: 68,
    podsLimit: 110,
    uptime: '18d 3h',
    labels: ['os=linux', 'role=worker', 'gpu=true']
  },
  {
    name: 'prod-node-worker-02',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 48,
    cpuUsed: '7.6',
    cpuTotal: '16',
    memPct: 65,
    memUsed: '20.8',
    memTotal: '32',
    podsCount: 52,
    podsLimit: 110,
    uptime: '18d 3h',
    labels: ['os=linux', 'role=worker']
  },
  {
    name: 'prod-node-worker-03',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 75,
    cpuUsed: '12.0',
    cpuTotal: '16',
    memPct: 88,
    memUsed: '28.1',
    memTotal: '32',
    podsCount: 94,
    podsLimit: 110,
    uptime: '15d 1h',
    labels: ['os=linux', 'role=worker', 'storage=fast']
  },
  {
    name: 'prod-node-worker-04',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 15,
    cpuUsed: '1.2',
    cpuTotal: '8',
    memPct: 42,
    memUsed: '6.7',
    memTotal: '16',
    podsCount: 15,
    podsLimit: 110,
    uptime: '18d 4h',
    labels: ['os=linux', 'role=worker']
  },
  {
    name: 'prod-node-worker-05',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 54,
    cpuUsed: '4.3',
    cpuTotal: '8',
    memPct: 70,
    memUsed: '11.2',
    memTotal: '16',
    podsCount: 45,
    podsLimit: 110,
    uptime: '18d 2h',
    labels: ['os=linux', 'role=worker']
  },
  {
    name: 'prod-node-worker-06',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 41,
    cpuUsed: '3.2',
    cpuTotal: '8',
    memPct: 59,
    memUsed: '9.4',
    memTotal: '16',
    podsCount: 38,
    podsLimit: 110,
    uptime: '18d 4h',
    labels: ['os=linux', 'role=worker']
  },
  {
    name: 'prod-node-worker-07',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 82,
    cpuUsed: '13.1',
    cpuTotal: '16',
    memPct: 91,
    memUsed: '29.1',
    memTotal: '32',
    podsCount: 102,
    podsLimit: 110,
    uptime: '10d 12h',
    labels: ['os=linux', 'role=worker', 'gpu=true']
  },
  {
    name: 'prod-node-worker-08',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 29,
    cpuUsed: '2.3',
    cpuTotal: '8',
    memPct: 47,
    memUsed: '7.5',
    memTotal: '16',
    podsCount: 31,
    podsLimit: 110,
    uptime: '18d 4h',
    labels: ['os=linux', 'role=worker']
  },
  {
    name: 'prod-node-worker-09',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 51,
    cpuUsed: '8.1',
    cpuTotal: '16',
    memPct: 69,
    memUsed: '22.0',
    memTotal: '32',
    podsCount: 58,
    podsLimit: 110,
    uptime: '12d 8h',
    labels: ['os=linux', 'role=worker']
  },
  {
    name: 'prod-node-worker-10',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 44,
    cpuUsed: '3.5',
    cpuTotal: '8',
    memPct: 62,
    memUsed: '9.9',
    memTotal: '16',
    podsCount: 40,
    podsLimit: 110,
    uptime: '18d 4h',
    labels: ['os=linux', 'role=worker']
  },
  {
    name: 'prod-node-worker-11',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 58,
    cpuUsed: '4.6',
    cpuTotal: '8',
    memPct: 75,
    memUsed: '12.0',
    memTotal: '16',
    podsCount: 48,
    podsLimit: 110,
    uptime: '18d 1h',
    labels: ['os=linux', 'role=worker']
  }
]
