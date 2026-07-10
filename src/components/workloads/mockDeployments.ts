export interface DeploymentInfo {
  name: string
  namespace: string
  status: 'Running' | 'Progressing' | 'Failed'
  replicas: {
    current: number
    desired: number
  }
  available: number
  upToDate: number
  age: string
  images: string[]
  strategy: string
  minReadySeconds: number
  revisionHistory: number
  labels: Record<string, string>
  annotations: Record<string, string>
}

export const mockDeployments: DeploymentInfo[] = [
  {
    name: 'api-gateway',
    namespace: 'default',
    status: 'Running',
    replicas: { current: 3, desired: 3 },
    available: 3,
    upToDate: 3,
    age: '12d',
    images: ['kong:3.4', 'redis:7.0-alpine'],
    strategy: 'RollingUpdate (maxUnavailable: 25%, maxSurge: 25%)',
    minReadySeconds: 0,
    revisionHistory: 10,
    labels: {
      app: 'api-gateway',
      version: 'v1.2.4',
      'app.kubernetes.io/managed-by': 'Helm'
    },
    annotations: {
      'deployment.kubernetes.io/revision': '4',
      'prometheus.io/scrape': 'true',
      'prometheus.io/port': '8000'
    }
  },
  {
    name: 'auth-service',
    namespace: 'default',
    status: 'Running',
    replicas: { current: 2, desired: 2 },
    available: 2,
    upToDate: 2,
    age: '4d 2h',
    images: ['orbit/auth-service:v1.0.1'],
    strategy: 'RollingUpdate (maxUnavailable: 0, maxSurge: 1)',
    minReadySeconds: 5,
    revisionHistory: 5,
    labels: {
      app: 'auth-service',
      environment: 'production'
    },
    annotations: {
      'deployment.kubernetes.io/revision': '2'
    }
  },
  {
    name: 'payment-processor',
    namespace: 'default',
    status: 'Progressing',
    replicas: { current: 3, desired: 4 },
    available: 2,
    upToDate: 3,
    age: '1h',
    images: ['orbit/payment-processor:v2.1.0-rc3'],
    strategy: 'RollingUpdate (maxUnavailable: 25%, maxSurge: 25%)',
    minReadySeconds: 0,
    revisionHistory: 10,
    labels: {
      app: 'payment-processor',
      tier: 'backend'
    },
    annotations: {
      'deployment.kubernetes.io/revision': '12',
      'checksum/config': 'a8f5b871c900e'
    }
  },
  {
    name: 'frontend-web',
    namespace: 'default',
    status: 'Running',
    replicas: { current: 5, desired: 5 },
    available: 5,
    upToDate: 5,
    age: '18d',
    images: ['nginx:1.25.1-alpine'],
    strategy: 'RollingUpdate (maxUnavailable: 20%, maxSurge: 20%)',
    minReadySeconds: 0,
    revisionHistory: 10,
    labels: {
      app: 'frontend-web',
      role: 'ui'
    },
    annotations: {
      'deployment.kubernetes.io/revision': '7'
    }
  },
  {
    name: 'prometheus-server',
    namespace: 'monitoring',
    status: 'Running',
    replicas: { current: 1, desired: 1 },
    available: 1,
    upToDate: 1,
    age: '30d',
    images: ['prom/prometheus:v2.45.0'],
    strategy: 'Recreate',
    minReadySeconds: 0,
    revisionHistory: 3,
    labels: {
      app: 'prometheus',
      component: 'server'
    },
    annotations: {
      'deployment.kubernetes.io/revision': '1'
    }
  },
  {
    name: 'grafana',
    namespace: 'monitoring',
    status: 'Running',
    replicas: { current: 2, desired: 2 },
    available: 2,
    upToDate: 2,
    age: '30d',
    images: ['grafana/grafana:10.0.3'],
    strategy: 'RollingUpdate (maxUnavailable: 25%, maxSurge: 25%)',
    minReadySeconds: 0,
    revisionHistory: 10,
    labels: {
      app: 'grafana'
    },
    annotations: {
      'deployment.kubernetes.io/revision': '3'
    }
  },
  {
    name: 'kube-state-metrics',
    namespace: 'kube-system',
    status: 'Running',
    replicas: { current: 1, desired: 1 },
    available: 1,
    upToDate: 1,
    age: '45d',
    images: ['registry.k8s.io/kube-state-metrics/kube-state-metrics:v2.9.2'],
    strategy: 'RollingUpdate (maxUnavailable: 25%, maxSurge: 25%)',
    minReadySeconds: 0,
    revisionHistory: 2,
    labels: {
      'app.kubernetes.io/name': 'kube-state-metrics'
    },
    annotations: {
      'deployment.kubernetes.io/revision': '1'
    }
  },
  {
    name: 'coredns',
    namespace: 'kube-system',
    status: 'Running',
    replicas: { current: 2, desired: 2 },
    available: 2,
    upToDate: 2,
    age: '45d',
    images: ['registry.k8s.io/dns/k8s-dns-node-cache:1.22.20'],
    strategy: 'RollingUpdate (maxUnavailable: 1, maxSurge: 25%)',
    minReadySeconds: 0,
    revisionHistory: 10,
    labels: {
      'k8s-app': 'kube-dns'
    },
    annotations: {
      'deployment.kubernetes.io/revision': '1'
    }
  },
  {
    name: 'notification-worker',
    namespace: 'default',
    status: 'Failed',
    replicas: { current: 0, desired: 2 },
    available: 0,
    upToDate: 0,
    age: '3h',
    images: ['orbit/notification-worker:v0.8.2'],
    strategy: 'RollingUpdate (maxUnavailable: 25%, maxSurge: 25%)',
    minReadySeconds: 0,
    revisionHistory: 10,
    labels: {
      app: 'notification-worker',
      tier: 'backend'
    },
    annotations: {
      'deployment.kubernetes.io/revision': '8'
    }
  }
]
