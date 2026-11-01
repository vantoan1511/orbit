import { KUBERNETES_RESOURCE_KIND } from '../constants/kubernetes.ts'

export interface EditWorkloadTargetParams {
  [key: string]: string
  kind: string
  namespace: string
  name: string
}

export function resolveEditTargetParams<
  T extends { name: string; namespace?: string; kind?: string; type?: string }
>(row: T, defaultKind: string = KUBERNETES_RESOURCE_KIND.Deployment): EditWorkloadTargetParams {
  const kind = row.kind || defaultKind
  const name = row.type === 'Pod Security' && row.namespace ? row.namespace : row.name
  const namespace =
    kind === 'Namespace' || !row.namespace || row.namespace === '-' ? 'default' : row.namespace
  return {
    kind,
    namespace,
    name
  }
}

export interface DeleteWorkloadTargetParams {
  kind: string
  namespace: string
  name: string
  isSuppressed: boolean
}

export function resolveDeleteTargetParams<
  T extends { name: string; namespace?: string; kind?: string; type?: string }
>(row: T, defaultKind: string = KUBERNETES_RESOURCE_KIND.Deployment): DeleteWorkloadTargetParams {
  const isSuppressed = row.type === 'Pod Security'
  const kind = row.kind || defaultKind
  const namespace = row.namespace && row.namespace !== '-' ? row.namespace : 'default'
  return {
    kind,
    namespace,
    name: row.name,
    isSuppressed
  }
}
