export interface WorkloadInfo {
  name: string
  type: string
  namespace: string
  node: string
  status: string
  cpu: string
  memory: string
}

export const mockWorkloads: WorkloadInfo[] = [
  {
    name: 'api-gateway-7f4d8c-2b8s',
    type: 'Deployment',
    namespace: 'default',
    node: 'prod-node-worker-01',
    status: 'Running',
    cpu: '180m',
    memory: '256 MiB'
  },
  {
    name: 'auth-service-6b4f7a-1a2b',
    type: 'Deployment',
    namespace: 'default',
    node: 'prod-node-worker-02',
    status: 'Running',
    cpu: '95m',
    memory: '128 MiB'
  },
  {
    name: 'payment-processor-5c8e9d-9z8y',
    type: 'Deployment',
    namespace: 'default',
    node: 'prod-node-worker-03',
    status: 'Running',
    cpu: '310m',
    memory: '512 MiB'
  },
  {
    name: 'postgres-db-0',
    type: 'StatefulSet',
    namespace: 'database',
    node: 'prod-node-worker-07',
    status: 'Running',
    cpu: '450m',
    memory: '1.2 GiB'
  },
  {
    name: 'postgres-db-1',
    type: 'StatefulSet',
    namespace: 'database',
    node: 'prod-node-worker-03',
    status: 'Pending',
    cpu: '0m',
    memory: '0 MiB'
  },
  {
    name: 'node-exporter-4z8x9',
    type: 'DaemonSet',
    namespace: 'monitoring',
    node: 'prod-node-worker-01',
    status: 'Running',
    cpu: '45m',
    memory: '64 MiB'
  },
  {
    name: 'node-exporter-2s1w8',
    type: 'DaemonSet',
    namespace: 'monitoring',
    node: 'prod-node-worker-02',
    status: 'Running',
    cpu: '45m',
    memory: '64 MiB'
  },
  {
    name: 'fluentbit-logger-8v9c2',
    type: 'DaemonSet',
    namespace: 'logging',
    node: 'prod-node-worker-04',
    status: 'Failed',
    cpu: '15m',
    memory: '32 MiB'
  }
]
