import {
  KUBERNETES_EVENT_TYPE,
  KUBERNETES_NAMESPACE_STATUS,
  KUBERNETES_NODE_STATUS,
  KUBERNETES_POD_STATUS,
  KUBERNETES_VOLUME_STATUS,
  KUBERNETES_WORKLOAD_STATUS
} from '../constants/kubernetes.ts'

export interface StatusVisualStyle {
  readonly dotClass: string
  readonly textClass: string
}

const STYLE_SUCCESS: StatusVisualStyle = Object.freeze({
  dotClass: 'bg-(--success)',
  textClass: 'text-(--success)'
})

const STYLE_WARNING: StatusVisualStyle = Object.freeze({
  dotClass: 'bg-(--warning)',
  textClass: 'text-(--warning)'
})

const STYLE_DANGER: StatusVisualStyle = Object.freeze({
  dotClass: 'bg-(--danger)',
  textClass: 'text-(--danger)'
})

const STYLE_MUTED: StatusVisualStyle = Object.freeze({
  dotClass: 'bg-(--text-muted)',
  textClass: 'text-(--text-muted)'
})

const STATUS_STYLE_MAP: Readonly<Record<string, StatusVisualStyle>> = Object.freeze({
  [KUBERNETES_POD_STATUS.Running]: STYLE_SUCCESS,
  [KUBERNETES_POD_STATUS.Completed]: STYLE_SUCCESS,
  [KUBERNETES_NAMESPACE_STATUS.Active]: STYLE_SUCCESS,
  [KUBERNETES_NODE_STATUS.Ready]: STYLE_SUCCESS,
  [KUBERNETES_VOLUME_STATUS.Bound]: STYLE_SUCCESS,
  [KUBERNETES_VOLUME_STATUS.Available]: STYLE_SUCCESS,
  [KUBERNETES_POD_STATUS.Succeeded]: STYLE_SUCCESS,

  [KUBERNETES_POD_STATUS.Pending]: STYLE_WARNING,
  [KUBERNETES_WORKLOAD_STATUS.Progressing]: STYLE_WARNING,
  [KUBERNETES_VOLUME_STATUS.Released]: STYLE_WARNING,
  [KUBERNETES_POD_STATUS.ContainerCreating]: STYLE_WARNING,
  [KUBERNETES_EVENT_TYPE.Warning]: STYLE_WARNING,

  [KUBERNETES_POD_STATUS.Failed]: STYLE_DANGER,
  [KUBERNETES_POD_STATUS.CrashLoopBackOff]: STYLE_DANGER,
  [KUBERNETES_POD_STATUS.Terminating]: STYLE_DANGER,
  [KUBERNETES_POD_STATUS.Error]: STYLE_DANGER
})

/**
 * Returns pre-allocated frozen visual styles (dot and text classes) for a Kubernetes status string.
 * Uses an O(1) table lookup without allocating or recomputing style strings.
 */
export function getStatusVisualStyle(status?: string): StatusVisualStyle {
  if (!status) {
    return STYLE_MUTED
  }
  return STATUS_STYLE_MAP[status] ?? STYLE_MUTED
}

export type NamespaceSeverity = 'secondary' | 'warn' | 'info'

const NAMESPACE_SEVERITY_MAP: Readonly<Record<string, NamespaceSeverity>> = Object.freeze({
  'kube-system': 'secondary',
  monitoring: 'warn',
  logging: 'info'
})

/**
 * Maps a Kubernetes namespace name to its PrimeVue Tag severity via an O(1) table lookup.
 */
export function getNamespaceSeverity(namespace?: string): NamespaceSeverity {
  if (!namespace) {
    return 'secondary'
  }
  return NAMESPACE_SEVERITY_MAP[namespace] ?? 'secondary'
}
