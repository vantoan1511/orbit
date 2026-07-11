export interface EventInfo {
  uid: string
  time: string // e.g. "10s", "2m", "1h"
  type: 'Normal' | 'Warning' | 'Error'
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

const baseEvents: EventInfo[] = [
  {
    uid: 'e1b2c3d4-5e6f-7a8b-9c0d-1e2f3a4b5c6d',
    time: '10s',
    type: 'Normal',
    reason: 'ScaledUp',
    objectName: 'api-gateway',
    objectKind: 'Deployment',
    message: 'Scaled up replica set api-gateway-5f7d9c11 to 3',
    namespace: 'backend',
    source: 'deployment-controller',
    firstSeen: 'May 12, 2025, 10:21 AM',
    lastSeen: 'May 12, 2025, 10:21 AM',
    count: 1,
    labels: { app: 'api-gateway' },
    annotations: { 'kubernetes.io/created-by': 'deployment-controller' }
  },
  {
    uid: 'f2c3d4e5-6f7a-8b9c-0d1e-2f3a4b5c6d7e',
    time: '32s',
    type: 'Warning',
    reason: 'FailedScheduling',
    objectName: 'user-service-xyz-12345',
    objectKind: 'Pod',
    message: '0/12 nodes are available: 3 Insufficient cpu, 9 node(s) had untolerated taint.',
    namespace: 'backend',
    source: 'default-scheduler',
    firstSeen: 'May 12, 2025, 10:22 AM',
    lastSeen: 'May 12, 2025, 10:25 AM',
    count: 8,
    labels: { app: 'user-service' },
    annotations: {}
  },
  {
    uid: 'a3d4e5f6-7a8b-9c0d-1e2f-3a4b5c6d7e8f',
    time: '1m',
    type: 'Error',
    reason: 'BackOff',
    objectName: 'payment-processor-pod-777',
    objectKind: 'Pod',
    message:
      'Back-off restarting failed container payment-processor in pod payment-processor-pod-777_backend',
    namespace: 'backend',
    source: 'kubelet',
    firstSeen: 'May 12, 2025, 10:15 AM',
    lastSeen: 'May 12, 2025, 10:28 AM',
    count: 14,
    labels: { app: 'payment-processor' },
    annotations: {}
  },
  {
    uid: 'b4e5f6a7-8b9c-0d1e-2f3a-4b5c6d7e8f9a',
    time: '2m',
    type: 'Normal',
    reason: 'Killing',
    objectName: 'redis-cache-0',
    objectKind: 'Pod',
    message: 'Stopping container redis',
    namespace: 'cache',
    source: 'kubelet',
    firstSeen: 'May 12, 2025, 10:18 AM',
    lastSeen: 'May 12, 2025, 10:18 AM',
    count: 1,
    labels: { app: 'redis' },
    annotations: {}
  },
  {
    uid: 'c5f6a7b8-9c0d-1e2f-3a4b-5c6d7e8f9a0b',
    time: '5m',
    type: 'Warning',
    reason: 'Unhealthy',
    objectName: 'web-frontend-55f6a',
    objectKind: 'Pod',
    message: 'Readiness probe failed: HTTP probe failed with statuscode: 502',
    namespace: 'frontend',
    source: 'kubelet',
    firstSeen: 'May 12, 2025, 10:10 AM',
    lastSeen: 'May 12, 2025, 10:20 AM',
    count: 5,
    labels: { app: 'web-frontend' },
    annotations: {}
  },
  {
    uid: 'd6a7b8c9-0d1e-2f3a-4b5c-6d7e8f9a0b1c',
    time: '12m',
    type: 'Normal',
    reason: 'Pulled',
    objectName: 'auth-service-v2',
    objectKind: 'Pod',
    message: 'Successfully pulled image "auth-service:v2.0.1" in 3.12s',
    namespace: 'backend',
    source: 'kubelet',
    firstSeen: 'May 12, 2025, 10:05 AM',
    lastSeen: 'May 12, 2025, 10:05 AM',
    count: 1,
    labels: { app: 'auth-service' },
    annotations: {}
  }
]

// Generate 60 extra mock events for robust pagination and sorting
const namespaces = [
  'default',
  'backend',
  'frontend',
  'cache',
  'database',
  'kube-system',
  'monitoring'
]
const eventTypes: ('Normal' | 'Warning' | 'Error')[] = [
  'Normal',
  'Warning',
  'Normal',
  'Normal',
  'Warning',
  'Error'
]
const reasons = {
  Normal: ['Pulled', 'Created', 'Started', 'ScaledUp', 'Killing', 'SuccessfulCreate'],
  Warning: ['FailedScheduling', 'Unhealthy', 'FailedMount', 'FailedValidation'],
  Error: ['BackOff', 'FailedCreatePod', 'CrashLoopBackOff']
}
const kinds = [
  'Pod',
  'Deployment',
  'ReplicaSet',
  'PersistentVolumeClaim',
  'DaemonSet',
  'StatefulSet'
]

const generatedEvents: EventInfo[] = Array.from({ length: 60 }, (_, i) => {
  const id = i + 7
  const type = eventTypes[id % eventTypes.length]!
  const list = reasons[type]
  const reason = list[id % list.length]!
  const namespace = namespaces[id % namespaces.length]!
  const objectKind = kinds[id % kinds.length]!
  const objectName = `${objectKind.toLowerCase()}-${id}-${Math.random().toString(36).substring(2, 6)}`

  let message = ''
  if (type === 'Normal') {
    message = `Event ${reason} for ${objectKind} ${objectName} completed successfully.`
  } else if (type === 'Warning') {
    message = `Warning: ${reason} on ${objectKind} ${objectName}. System performance may be degraded.`
  } else {
    message = `Error: ${reason} on ${objectKind} ${objectName}. Process exited with code 1.`
  }

  const count = (id % 10) + 1

  return {
    uid: `3c${id}2a4e-4f5a-6b7c-8d9e-${id}f1a2b3c4d5`,
    time: `${id}m`,
    type,
    reason,
    objectName,
    objectKind,
    message,
    namespace,
    source: id % 3 === 0 ? 'kubelet' : id % 3 === 1 ? 'default-scheduler' : 'deployment-controller',
    firstSeen: `June ${(id % 30) + 1}, 2026, 12:${id % 60} PM`,
    lastSeen: `June ${(id % 30) + 1}, 2026, 12:${(id + 2) % 60} PM`,
    count,
    labels: { app: `app-${id}` },
    annotations: {}
  }
})

export const mockEvents: EventInfo[] = [...baseEvents, ...generatedEvents]
