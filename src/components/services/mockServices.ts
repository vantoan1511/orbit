export interface ServicePort {
  port: number
  targetPort: number | string
  protocol: 'TCP' | 'UDP'
  nodePort?: number
}

export interface ServiceEvent {
  type: 'Normal' | 'Warning'
  reason: string
  message: string
  age: string
}

export interface ServiceInfo {
  name: string
  namespace: string
  type: 'ClusterIP' | 'NodePort' | 'LoadBalancer' | 'ExternalName'
  clusterIP: string
  externalIP: string
  ports: string
  endpoints: string
  age: string
  sessionAffinity: 'None' | 'ClientIP'
  internalTrafficPolicy: 'Cluster' | 'Local'
  created: string
  uid: string
  selector: Record<string, string>
  labels: Record<string, string>
  portsList: ServicePort[]
  endpointsList: string[]
  events: ServiceEvent[]
}

const baseServices: ServiceInfo[] = [
  {
    name: 'api-gateway',
    namespace: 'backend',
    type: 'LoadBalancer',
    clusterIP: '10.0.45.12',
    externalIP: '34.198.12.45',
    ports: '80:32180/TCP\n443:32443/TCP',
    endpoints: '3 / 3',
    age: '7d 4h',
    sessionAffinity: 'None',
    internalTrafficPolicy: 'Cluster',
    created: 'May 12, 2025, 10:21 AM',
    uid: 'f5a7c9e2-2b1d-4cce-9a6a-0f2b8e7d9c11',
    selector: { app: 'api-gateway', tier: 'backend' },
    labels: {
      app: 'api-gateway',
      tier: 'backend',
      env: 'production',
      team: 'platform'
    },
    portsList: [
      { port: 80, targetPort: 8080, protocol: 'TCP', nodePort: 32180 },
      { port: 443, targetPort: 8443, protocol: 'TCP', nodePort: 32443 }
    ],
    endpointsList: ['10.244.1.45:8080', '10.244.2.12:8080', '10.244.3.8:8080'],
    events: [
      {
        type: 'Normal',
        reason: 'EnsuredLoadBalancer',
        message: 'Successfully created cloud load balancer',
        age: '7d'
      }
    ]
  },
  {
    name: 'user-service',
    namespace: 'backend',
    type: 'ClusterIP',
    clusterIP: '10.0.96.10',
    externalIP: '-',
    ports: '8080/TCP',
    endpoints: '4 / 4',
    age: '7d 4h',
    sessionAffinity: 'None',
    internalTrafficPolicy: 'Cluster',
    created: 'May 12, 2025, 10:25 AM',
    uid: 'a4b2c1d3-3e4f-5a6b-7c8d-9e0f1a2b3c4d',
    selector: { app: 'user-service' },
    labels: { app: 'user-service', tier: 'backend', env: 'production' },
    portsList: [{ port: 8080, targetPort: 8080, protocol: 'TCP' }],
    endpointsList: ['10.244.1.15:8080', '10.244.2.22:8080', '10.244.3.11:8080', '10.244.4.5:8080'],
    events: []
  },
  {
    name: 'order-service',
    namespace: 'backend',
    type: 'ClusterIP',
    clusterIP: '10.0.199.18',
    externalIP: '-',
    ports: '8080/TCP',
    endpoints: '3 / 3',
    age: '2d 6h',
    sessionAffinity: 'None',
    internalTrafficPolicy: 'Cluster',
    created: 'May 17, 2025, 08:15 AM',
    uid: 'b5c3d2e4-4f5a-6b7c-8d9e-0f1a2b3c4d5e',
    selector: { app: 'order-service' },
    labels: { app: 'order-service', tier: 'backend', env: 'production' },
    portsList: [{ port: 8080, targetPort: 8080, protocol: 'TCP' }],
    endpointsList: ['10.244.1.16:8080', '10.244.2.23:8080', '10.244.3.12:8080'],
    events: []
  },
  {
    name: 'payment-service',
    namespace: 'backend',
    type: 'ClusterIP',
    clusterIP: '10.0.156.23',
    externalIP: '-',
    ports: '8080/TCP',
    endpoints: '2 / 2',
    age: '5d 2h',
    sessionAffinity: 'None',
    internalTrafficPolicy: 'Cluster',
    created: 'May 14, 2025, 02:40 PM',
    uid: 'c6d4e3f5-5a6b-7c8d-9e0f-1a2b3c4d5e6f',
    selector: { app: 'payment-service' },
    labels: { app: 'payment-service', tier: 'backend', env: 'production' },
    portsList: [{ port: 8080, targetPort: 8080, protocol: 'TCP' }],
    endpointsList: ['10.244.1.17:8080', '10.244.2.24:8080'],
    events: []
  },
  {
    name: 'inventory-service',
    namespace: 'frontend',
    type: 'NodePort',
    clusterIP: '10.0.52.34',
    externalIP: '-',
    ports: '8080:30080/TCP',
    endpoints: '3 / 3',
    age: '12d 10h',
    sessionAffinity: 'None',
    internalTrafficPolicy: 'Cluster',
    created: 'May 7, 2025, 11:30 AM',
    uid: 'd7e5f4a6-6b7c-8d9e-0f1a-2b3c4d5e6f7a',
    selector: { app: 'inventory-service' },
    labels: { app: 'inventory-service', tier: 'frontend', env: 'production' },
    portsList: [{ port: 8080, targetPort: 8080, protocol: 'TCP', nodePort: 30080 }],
    endpointsList: ['10.244.1.18:8080', '10.244.2.25:8080', '10.244.3.13:8080'],
    events: []
  },
  {
    name: 'web-frontend',
    namespace: 'frontend',
    type: 'LoadBalancer',
    clusterIP: '10.0.11.21',
    externalIP: '35.221.88.19',
    ports: '80:30080/TCP\n443:30443/TCP',
    endpoints: '3 / 3',
    age: '9d 1h',
    sessionAffinity: 'None',
    internalTrafficPolicy: 'Cluster',
    created: 'May 10, 2025, 09:12 AM',
    uid: 'e8f6a5b7-7c8d-9e0f-1a2b-3c4d5e6f7a8b',
    selector: { app: 'web-frontend' },
    labels: { app: 'web-frontend', tier: 'frontend', env: 'production' },
    portsList: [
      { port: 80, targetPort: 80, protocol: 'TCP', nodePort: 30080 },
      { port: 443, targetPort: 443, protocol: 'TCP', nodePort: 30443 }
    ],
    endpointsList: ['10.244.1.19:80', '10.244.2.26:80', '10.244.3.14:80'],
    events: []
  },
  {
    name: 'checkout-web',
    namespace: 'frontend',
    type: 'NodePort',
    clusterIP: '10.0.83.77',
    externalIP: '-',
    ports: '80:30081/TCP',
    endpoints: '2 / 2',
    age: '9d 1h',
    sessionAffinity: 'None',
    internalTrafficPolicy: 'Cluster',
    created: 'May 10, 2025, 09:15 AM',
    uid: 'f9a7b6c8-8d9e-0f1a-2b3c-4d5e6f7a8b9c',
    selector: { app: 'checkout-web' },
    labels: { app: 'checkout-web', tier: 'frontend', env: 'production' },
    portsList: [{ port: 80, targetPort: 8080, protocol: 'TCP', nodePort: 30081 }],
    endpointsList: ['10.244.1.20:8080', '10.244.2.27:8080'],
    events: []
  },
  {
    name: 'redis',
    namespace: 'cache',
    type: 'ClusterIP',
    clusterIP: '10.0.88.5',
    externalIP: '-',
    ports: '6379/TCP',
    endpoints: '1 / 1',
    age: '18d 2h',
    sessionAffinity: 'None',
    internalTrafficPolicy: 'Cluster',
    created: 'May 1, 2025, 08:00 AM',
    uid: 'a0b8c7d9-9e0f-1a2b-3c4d-5e6f7a8b9c0d',
    selector: { app: 'redis' },
    labels: { app: 'redis', tier: 'cache', env: 'production' },
    portsList: [{ port: 6379, targetPort: 6379, protocol: 'TCP' }],
    endpointsList: ['10.244.1.5:6379'],
    events: []
  },
  {
    name: 'postgres',
    namespace: 'database',
    type: 'ClusterIP',
    clusterIP: '10.0.70.9',
    externalIP: '-',
    ports: '5432/TCP',
    endpoints: '1 / 1',
    age: '18d 2h',
    sessionAffinity: 'None',
    internalTrafficPolicy: 'Cluster',
    created: 'May 1, 2025, 08:05 AM',
    uid: 'b1c9d8e0-0f1a-2b3c-4d5e-6f7a8b9c0d1e',
    selector: { app: 'postgres' },
    labels: { app: 'postgres', tier: 'database', env: 'production' },
    portsList: [{ port: 5432, targetPort: 5432, protocol: 'TCP' }],
    endpointsList: ['10.244.1.6:5432'],
    events: []
  },
  {
    name: 'kafka',
    namespace: 'messaging',
    type: 'NodePort',
    clusterIP: '10.0.20.15',
    externalIP: '-',
    ports: '9092:30092/TCP',
    endpoints: '3 / 3',
    age: '18d 2h',
    sessionAffinity: 'None',
    internalTrafficPolicy: 'Cluster',
    created: 'May 1, 2025, 08:10 AM',
    uid: 'c2d0e9f1-1a2b-3c4d-5e6f-7a8b9c0d1e2f',
    selector: { app: 'kafka' },
    labels: { app: 'kafka', tier: 'messaging', env: 'production' },
    portsList: [{ port: 9092, targetPort: 9092, protocol: 'TCP', nodePort: 30092 }],
    endpointsList: ['10.244.1.7:9092', '10.244.2.8:9092', '10.244.3.9:9092'],
    events: []
  },
  {
    name: 'metrics-server',
    namespace: 'kube-system',
    type: 'ClusterIP',
    clusterIP: '10.0.101.50',
    externalIP: '-',
    ports: '443/TCP',
    endpoints: '1 / 1',
    age: '18d 4h',
    sessionAffinity: 'None',
    internalTrafficPolicy: 'Cluster',
    created: 'May 1, 2025, 06:00 AM',
    uid: 'd3e1f0a2-2b3c-4d5e-6f7a-8b9c0d1e2f3a',
    selector: { 'k8s-app': 'metrics-server' },
    labels: { 'k8s-app': 'metrics-server', 'kubernetes.io/name': 'Metrics-Server' },
    portsList: [{ port: 443, targetPort: 443, protocol: 'TCP' }],
    endpointsList: ['10.244.0.12:443'],
    events: []
  },
  {
    name: 'kube-dns',
    namespace: 'kube-system',
    type: 'ClusterIP',
    clusterIP: '10.0.0.10',
    externalIP: '-',
    ports: '53/UDP\n53/TCP',
    endpoints: '2 / 2',
    age: '18d 4h',
    sessionAffinity: 'None',
    internalTrafficPolicy: 'Cluster',
    created: 'May 1, 2025, 05:55 AM',
    uid: 'e4f2a1b3-3c4d-5e6f-7a8b-9c0d1e2f3a4b',
    selector: { 'k8s-app': 'kube-dns' },
    labels: { 'k8s-app': 'kube-dns', 'kubernetes.io/cluster-service': 'true' },
    portsList: [
      { port: 53, targetPort: 53, protocol: 'UDP' },
      { port: 53, targetPort: 53, protocol: 'TCP' }
    ],
    endpointsList: ['10.244.0.2:53', '10.244.0.3:53'],
    events: []
  }
]

// Generate additional 84 mock services to make total 96
const namespacesList = [
  'default',
  'backend',
  'frontend',
  'cache',
  'database',
  'messaging',
  'kube-system',
  'monitoring',
  'logging',
  'analytics'
]

const generatedServices: ServiceInfo[] = Array.from({ length: 84 }, (_, i) => {
  const id = i + 13
  const namespace = namespacesList[id % namespacesList.length] || 'default'
  // Weighted types: mostly ClusterIP (approx 75%), then NodePort, then LoadBalancer, then ExternalName
  const typeRand = Math.random()
  const type =
    typeRand < 0.75
      ? 'ClusterIP'
      : typeRand < 0.9
        ? 'NodePort'
        : typeRand < 0.98
          ? 'LoadBalancer'
          : 'ExternalName'

  const clusterIP = type !== 'ExternalName' ? `10.0.${(id * 7) % 255}.${(id * 13) % 255}` : '-'
  const externalIP =
    type === 'LoadBalancer'
      ? `35.221.${(id * 3) % 255}.${(id * 11) % 255}`
      : type === 'ExternalName'
        ? `my-ext-service-${id}.database.windows.net`
        : '-'

  const port = 8000 + (id % 100)
  const targetPort = 80 + (id % 10)
  const ports =
    type === 'NodePort'
      ? `${port}:3${id % 1000}/TCP`
      : type === 'LoadBalancer'
        ? `80:30${id % 100}/TCP\n443:30${100 + (id % 100)}/TCP`
        : `${port}/TCP`

  const replicaCount = (id % 3) + 1
  const endpoints = type === 'ExternalName' ? '-' : `${replicaCount} / ${replicaCount}`

  const endpointsList =
    type === 'ExternalName'
      ? []
      : Array.from({ length: replicaCount }, (_, r) => `10.244.${id % 255}.${r + 10}:${targetPort}`)

  const portsList: ServicePort[] =
    type === 'LoadBalancer'
      ? [
          { port: 80, targetPort: 80, protocol: 'TCP', nodePort: 30000 + (id % 1000) },
          { port: 443, targetPort: 443, protocol: 'TCP', nodePort: 30100 + (id % 1000) }
        ]
      : type === 'NodePort'
        ? [{ port, targetPort, protocol: 'TCP', nodePort: 30000 + (id % 1000) }]
        : [{ port, targetPort, protocol: 'TCP' }]

  return {
    name: `service-${id}-${Math.random().toString(36).substring(2, 6)}`,
    namespace,
    type,
    clusterIP,
    externalIP,
    ports,
    endpoints,
    age: `${Math.floor(id / 5) + 1}d ${id % 24}h`,
    sessionAffinity: 'None',
    internalTrafficPolicy: 'Cluster',
    created: `June ${(id % 30) + 1}, 2026, 12:${id % 60} PM`,
    uid: `3c${id}2a4e-4f5a-6b7c-8d9e-${id}f1a2b3c4d5`,
    selector: { app: `app-${id}` },
    labels: { app: `app-${id}`, env: id % 2 === 0 ? 'production' : 'staging' },
    portsList,
    endpointsList,
    events: []
  }
})

export const mockServices: ServiceInfo[] = [...baseServices, ...generatedServices]
