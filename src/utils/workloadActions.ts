import { KUBERNETES_RESOURCE_KIND } from '../constants/kubernetes.ts'

export interface EditWorkloadTargetParams {
  kind: string
  namespace: string
  name: string
}

export function resolveEditTargetParams<
  T extends { name: string; namespace?: string; kind?: string }
>(row: T, defaultKind: string = KUBERNETES_RESOURCE_KIND.Deployment): EditWorkloadTargetParams {
  const kind = row.kind || defaultKind
  const namespace = row.namespace && row.namespace !== '-' ? row.namespace : 'default'
  return {
    kind,
    namespace,
    name: row.name
  }
}
