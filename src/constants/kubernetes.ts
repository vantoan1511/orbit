/**
 * Kubernetes resource kinds supported across Orbit.
 */
export const KUBERNETES_RESOURCE_KIND = {
  Deployment: 'Deployment',
  DaemonSet: 'DaemonSet',
  StatefulSet: 'StatefulSet',
  ReplicaSet: 'ReplicaSet',
  Job: 'Job',
  CronJob: 'CronJob',
  Pod: 'Pod',
  Service: 'Service',
  Ingress: 'Ingress',
  ConfigMap: 'ConfigMap',
  Secret: 'Secret',
  Node: 'Node',
  Namespace: 'Namespace',
  PersistentVolume: 'PersistentVolume',
  PersistentVolumeClaim: 'PersistentVolumeClaim',
  StorageClass: 'StorageClass',
  Event: 'Event',
  Policy: 'Policy'
} as const

export type KubernetesResourceKind =
  (typeof KUBERNETES_RESOURCE_KIND)[keyof typeof KUBERNETES_RESOURCE_KIND]

/**
 * Kubernetes Service types.
 */
export const KUBERNETES_SERVICE_TYPE = {
  ClusterIP: 'ClusterIP',
  NodePort: 'NodePort',
  LoadBalancer: 'LoadBalancer',
  ExternalName: 'ExternalName'
} as const

export type KubernetesServiceType =
  (typeof KUBERNETES_SERVICE_TYPE)[keyof typeof KUBERNETES_SERVICE_TYPE]

/**
 * Common Kubernetes Pod phases / statuses.
 */
export const KUBERNETES_POD_STATUS = {
  Running: 'Running',
  Pending: 'Pending',
  Failed: 'Failed',
  Succeeded: 'Succeeded',
  Completed: 'Completed',
  CrashLoopBackOff: 'CrashLoopBackOff',
  ContainerCreating: 'ContainerCreating',
  Error: 'Error',
  Unknown: 'Unknown',
  Terminating: 'Terminating'
} as const

export type KubernetesPodStatus = (typeof KUBERNETES_POD_STATUS)[keyof typeof KUBERNETES_POD_STATUS]

/**
 * Common Kubernetes Pod restart policies.
 */
export const KUBERNETES_RESTART_POLICY = {
  Always: 'Always',
  OnFailure: 'OnFailure',
  Never: 'Never'
} as const

export type KubernetesRestartPolicy =
  (typeof KUBERNETES_RESTART_POLICY)[keyof typeof KUBERNETES_RESTART_POLICY]

export const KUBERNETES_RESTART_POLICIES: KubernetesRestartPolicy[] =
  Object.values(KUBERNETES_RESTART_POLICY)

/**
 * Common Kubernetes Workload statuses (Deployment, DaemonSet, StatefulSet, ReplicaSet).
 */
export const KUBERNETES_WORKLOAD_STATUS = {
  Running: 'Running',
  Progressing: 'Progressing',
  Failed: 'Failed',
  Paused: 'Paused',
  Completed: 'Completed'
} as const

export type KubernetesWorkloadStatus =
  (typeof KUBERNETES_WORKLOAD_STATUS)[keyof typeof KUBERNETES_WORKLOAD_STATUS]

/**
 * Common Kubernetes Job statuses.
 */
export const KUBERNETES_JOB_STATUS = {
  Active: 'Active',
  Succeeded: 'Succeeded',
  Failed: 'Failed',
  Unknown: 'Unknown',
  Suspended: 'Suspended'
} as const

export type KubernetesJobStatus = (typeof KUBERNETES_JOB_STATUS)[keyof typeof KUBERNETES_JOB_STATUS]

/**
 * Common Kubernetes Namespace phases.
 */
export const KUBERNETES_NAMESPACE_STATUS = {
  Active: 'Active',
  Terminating: 'Terminating'
} as const

export type KubernetesNamespaceStatus =
  (typeof KUBERNETES_NAMESPACE_STATUS)[keyof typeof KUBERNETES_NAMESPACE_STATUS]

/**
 * Common Kubernetes Node statuses.
 */
export const KUBERNETES_NODE_STATUS = {
  Ready: 'Ready',
  NotReady: 'NotReady',
  Unknown: 'Unknown'
} as const

export type KubernetesNodeStatus =
  (typeof KUBERNETES_NODE_STATUS)[keyof typeof KUBERNETES_NODE_STATUS]

/**
 * Kubernetes Event types.
 */
export const KUBERNETES_EVENT_TYPE = {
  Normal: 'Normal',
  Warning: 'Warning',
  Error: 'Error'
} as const

export type KubernetesEventType = (typeof KUBERNETES_EVENT_TYPE)[keyof typeof KUBERNETES_EVENT_TYPE]

/**
 * Kubernetes QoS classes.
 */
export const KUBERNETES_QOS_CLASS = {
  Guaranteed: 'Guaranteed',
  Burstable: 'Burstable',
  BestEffort: 'BestEffort'
} as const

export type KubernetesQosClass = (typeof KUBERNETES_QOS_CLASS)[keyof typeof KUBERNETES_QOS_CLASS]

/**
 * Kubernetes PersistentVolume / PersistentVolumeClaim statuses.
 */
export const KUBERNETES_VOLUME_STATUS = {
  Bound: 'Bound',
  Available: 'Available',
  Released: 'Released',
  Failed: 'Failed',
  Lost: 'Lost',
  Pending: 'Pending'
} as const

export type KubernetesVolumeStatus =
  (typeof KUBERNETES_VOLUME_STATUS)[keyof typeof KUBERNETES_VOLUME_STATUS]

/**
 * Kubernetes Volume modes.
 */
export const KUBERNETES_VOLUME_MODE = {
  Filesystem: 'Filesystem',
  Block: 'Block'
} as const

export type KubernetesVolumeMode =
  (typeof KUBERNETES_VOLUME_MODE)[keyof typeof KUBERNETES_VOLUME_MODE]

/**
 * Kubernetes PersistentVolume reclaim policies.
 */
export const KUBERNETES_RECLAIM_POLICY = {
  Retain: 'Retain',
  Delete: 'Delete',
  Recycle: 'Recycle'
} as const

export type KubernetesReclaimPolicy =
  (typeof KUBERNETES_RECLAIM_POLICY)[keyof typeof KUBERNETES_RECLAIM_POLICY]

/**
 * Resource update actions from backend.
 */
export const KUBERNETES_ACTION = {
  Applied: 'Applied',
  Deleted: 'Deleted'
} as const

export type KubernetesAction = (typeof KUBERNETES_ACTION)[keyof typeof KUBERNETES_ACTION]

/**
 * Cluster health status.
 */
export const KUBERNETES_CLUSTER_STATUS = {
  Healthy: 'healthy',
  Offline: 'offline'
} as const

export type KubernetesClusterStatus =
  (typeof KUBERNETES_CLUSTER_STATUS)[keyof typeof KUBERNETES_CLUSTER_STATUS]
