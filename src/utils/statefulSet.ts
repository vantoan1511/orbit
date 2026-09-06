import type { StatefulSet } from 'kubernetes-types/apps/v1'
import type {
  Container,
  EnvFromSource,
  EnvVar,
  PersistentVolumeClaim,
  ResourceRequirements
} from 'kubernetes-types/core/v1'
import {
  KUBERNETES_STATEFULSET_POD_MANAGEMENT_POLICY,
  KUBERNETES_STATEFULSET_UPDATE_STRATEGY
} from '../constants/kubernetes.ts'
import { isValidK8sLabel, isValidK8sName, isValidPath } from './validators.ts'

export interface FormattedVolumeClaim {
  name: string
  storageClassName: string
  accessModes: string
  storageCapacity: string
  volumeMode: string
}

export interface ContainerEnvFormItem {
  type:
    | 'Literal'
    | 'ConfigMapKey'
    | 'SecretKey'
    | 'FieldRef'
    | 'ResourceFieldRef'
    | 'ConfigMapEnvFrom'
    | 'SecretEnvFrom'
  name?: string
  value?: string
  refName?: string
  refKey?: string
  fieldPath?: string
  prefix?: string
}

export interface StatefulSetContainerFormState {
  name: string
  image: string
  imagePullPolicy: string
  workingDir: string
  command: string[]
  args: string[]
  environments: ContainerEnvFormItem[]
  ports: { name: string; containerPort: number; protocol: string }[]
  cpuRequest: string
  memoryRequest: string
  cpuLimit: string
  memoryLimit: string
  rawContainer: Container
}

export interface StatefulSetContainerValidationItem {
  name?: string
  workingDir?: string
  ports?: Array<{ name?: string }>
}

export interface StatefulSetFormValidationInput {
  serviceName?: string
  serviceAccountName?: string
  strategyType?: string
  maxUnavailable?: string
  containers?: StatefulSetContainerValidationItem[]
}

/**
 * Validates Kubernetes maxUnavailable string (percentage 0-100% or non-negative integer).
 */
export function isValidMaxUnavailable(value?: string | number | null): boolean {
  if (value === undefined || value === null) return true
  const str = String(value).trim()
  if (!str) return true

  if (/^\d+%$/.test(str)) {
    const pct = parseInt(str.slice(0, -1), 10)
    return pct >= 0 && pct <= 100
  }

  if (/^\d+$/.test(str)) {
    return true
  }

  return false
}

/**
 * Validates StatefulSet form fields against Kubernetes constraints.
 */
export function validateStatefulSetForm(input: StatefulSetFormValidationInput): boolean {
  if (!input.serviceName?.trim() || !isValidK8sName(input.serviceName.trim())) {
    return false
  }

  if (input.serviceAccountName && !isValidK8sName(input.serviceAccountName)) {
    return false
  }

  if (
    input.strategyType === KUBERNETES_STATEFULSET_UPDATE_STRATEGY.RollingUpdate &&
    input.maxUnavailable &&
    !isValidMaxUnavailable(input.maxUnavailable)
  ) {
    return false
  }

  if (input.containers) {
    for (const c of input.containers) {
      if (c.name && !isValidK8sLabel(c.name)) {
        return false
      }
      if (c.workingDir && !isValidPath(c.workingDir)) {
        return false
      }
      if (c.ports) {
        for (const p of c.ports) {
          if (p.name && !isValidK8sLabel(p.name)) {
            return false
          }
        }
      }
    }
  }

  return true
}

/**
 * Parses raw Kubernetes containers into form state for Container editors.
 */
export function parseContainersFromPodSpec(
  containers?: Container[]
): StatefulSetContainerFormState[] {
  if (!containers || !Array.isArray(containers)) return []

  return containers.map((c: Container) => {
    const environments: ContainerEnvFormItem[] = []

    const envList = c.env || []
    for (const e of envList) {
      if (e && typeof e === 'object' && e.name) {
        if ('value' in e) {
          environments.push({
            type: 'Literal',
            name: e.name,
            value: String(e.value ?? '')
          })
        } else if (e.valueFrom) {
          const vf = e.valueFrom
          if (vf.configMapKeyRef) {
            environments.push({
              type: 'ConfigMapKey',
              name: e.name,
              refName: String(vf.configMapKeyRef.name || ''),
              refKey: String(vf.configMapKeyRef.key || '')
            })
          } else if (vf.secretKeyRef) {
            environments.push({
              type: 'SecretKey',
              name: e.name,
              refName: String(vf.secretKeyRef.name || ''),
              refKey: String(vf.secretKeyRef.key || '')
            })
          } else if (vf.fieldRef) {
            environments.push({
              type: 'FieldRef',
              name: e.name,
              fieldPath: String(vf.fieldRef.fieldPath || '')
            })
          } else if (vf.resourceFieldRef) {
            environments.push({
              type: 'ResourceFieldRef',
              name: e.name,
              fieldPath: String(vf.resourceFieldRef.resource || '')
            })
          }
        }
      }
    }

    const envFromList = c.envFrom || []
    for (const ef of envFromList) {
      if (ef) {
        const prefix = ef.prefix || ''
        if (ef.configMapRef) {
          environments.push({
            type: 'ConfigMapEnvFrom',
            prefix,
            refName: String(ef.configMapRef.name || '')
          })
        } else if (ef.secretRef) {
          environments.push({
            type: 'SecretEnvFrom',
            prefix,
            refName: String(ef.secretRef.name || '')
          })
        }
      }
    }

    const portList = c.ports || []
    const parsedPorts = portList.map((p) => ({
      name: p.name || '',
      containerPort: typeof p.containerPort === 'number' ? p.containerPort : 80,
      protocol: p.protocol || 'TCP'
    }))

    const resources = c.resources || {}
    const reqs = resources.requests || {}
    const lims = resources.limits || {}

    return {
      name: c.name || '',
      image: c.image || '',
      imagePullPolicy: c.imagePullPolicy || 'IfNotPresent',
      workingDir: c.workingDir || '',
      command: c.command ? [...c.command] : [],
      args: c.args ? [...c.args] : [],
      environments,
      ports: parsedPorts,
      cpuRequest: reqs.cpu ? String(reqs.cpu) : '',
      memoryRequest: reqs.memory ? String(reqs.memory) : '',
      cpuLimit: lims.cpu ? String(lims.cpu) : '',
      memoryLimit: lims.memory ? String(lims.memory) : '',
      rawContainer: c
    }
  })
}

/**
 * Serializes form state back into raw Kubernetes container specifications.
 */
export function formatContainersToPodSpec(
  formContainers: StatefulSetContainerFormState[]
): Container[] {
  return formContainers.map((c) => {
    const containerObj: Container = { ...c.rawContainer }
    containerObj.name = c.name
    containerObj.image = c.image
    containerObj.imagePullPolicy = c.imagePullPolicy

    if (c.workingDir) {
      containerObj.workingDir = c.workingDir
    } else {
      delete containerObj.workingDir
    }

    if (c.command.length > 0) {
      containerObj.command = c.command.filter((cmd) => cmd.trim() !== '')
    } else {
      delete containerObj.command
    }

    if (c.args.length > 0) {
      containerObj.args = c.args.filter((arg) => arg.trim() !== '')
    } else {
      delete containerObj.args
    }

    const envObjList: EnvVar[] = []
    const envFromObjList: EnvFromSource[] = []

    for (const e of c.environments) {
      if (e.type === 'ConfigMapEnvFrom' || e.type === 'SecretEnvFrom') {
        if (!e.refName?.trim()) continue
        const item: EnvFromSource = {}
        if (e.prefix?.trim()) item.prefix = e.prefix.trim()
        if (e.type === 'ConfigMapEnvFrom') {
          item.configMapRef = { name: e.refName.trim() }
        } else {
          item.secretRef = { name: e.refName.trim() }
        }
        envFromObjList.push(item)
      } else {
        if (!e.name?.trim()) continue
        const item: EnvVar = { name: e.name.trim() }
        if (e.type === 'Literal') {
          item.value = e.value || ''
        } else if (e.type === 'ConfigMapKey' && e.refName?.trim() && e.refKey?.trim()) {
          item.valueFrom = { configMapKeyRef: { name: e.refName.trim(), key: e.refKey.trim() } }
        } else if (e.type === 'SecretKey' && e.refName?.trim() && e.refKey?.trim()) {
          item.valueFrom = { secretKeyRef: { name: e.refName.trim(), key: e.refKey.trim() } }
        } else if (e.type === 'FieldRef' && e.fieldPath?.trim()) {
          item.valueFrom = { fieldRef: { fieldPath: e.fieldPath.trim() } }
        } else if (e.type === 'ResourceFieldRef' && e.fieldPath?.trim()) {
          item.valueFrom = { resourceFieldRef: { resource: e.fieldPath.trim() } }
        } else {
          continue
        }
        envObjList.push(item)
      }
    }

    if (envObjList.length > 0) {
      containerObj.env = envObjList
    } else {
      delete containerObj.env
    }

    if (envFromObjList.length > 0) {
      containerObj.envFrom = envFromObjList
    } else {
      delete containerObj.envFrom
    }

    const validPorts = c.ports.map((p) => {
      const portItem: { containerPort: number; protocol: string; name?: string } = {
        containerPort: p.containerPort,
        protocol: p.protocol || 'TCP'
      }
      if (p.name.trim()) portItem.name = p.name.trim()
      return portItem
    })
    if (validPorts.length > 0) {
      containerObj.ports = validPorts
    } else {
      delete containerObj.ports
    }

    const reqs: Record<string, string> = {}
    if (c.cpuRequest.trim()) reqs.cpu = c.cpuRequest.trim()
    if (c.memoryRequest.trim()) reqs.memory = c.memoryRequest.trim()

    const lims: Record<string, string> = {}
    if (c.cpuLimit.trim()) lims.cpu = c.cpuLimit.trim()
    if (c.memoryLimit.trim()) lims.memory = c.memoryLimit.trim()

    if (Object.keys(reqs).length > 0 || Object.keys(lims).length > 0) {
      const resObj: ResourceRequirements = { ...containerObj.resources }
      if (Object.keys(reqs).length > 0) resObj.requests = reqs
      else delete resObj.requests

      if (Object.keys(lims).length > 0) resObj.limits = lims
      else delete resObj.limits

      containerObj.resources = resObj
    } else {
      delete containerObj.resources
    }

    return containerObj
  })
}

/**
 * Formats VolumeClaimTemplates from a StatefulSet into an informative, displayable structure.
 */
export function parseVolumeClaimTemplates(
  templates?: PersistentVolumeClaim[]
): FormattedVolumeClaim[] {
  if (!templates || !Array.isArray(templates)) return []

  return templates.map((pvc) => {
    const name = pvc.metadata?.name || 'unnamed'
    const spec = pvc.spec || {}
    const storageClassName = spec.storageClassName || 'default'
    const accessModes = (spec.accessModes || []).join(', ') || 'ReadWriteOnce'
    const storageCapacity = spec.resources?.requests?.storage || 'N/A'
    const volumeMode = spec.volumeMode || 'Filesystem'

    return {
      name,
      storageClassName,
      accessModes,
      storageCapacity,
      volumeMode
    }
  })
}

export interface NormalizedStatefulSetSpec {
  replicas: number
  serviceName: string
  podManagementPolicy: string
  strategyType: string
  partition: number
  maxUnavailable: string
  minReadySeconds: number
  revisionHistoryLimit: number
  selectorLabels: Array<{ key: string; value: string }>
}

/**
 * Normalizes StatefulSet specification parameters for form controls.
 */
export function normalizeStatefulSetSpec(data: StatefulSet | null): NormalizedStatefulSetSpec {
  const spec = data?.spec

  const replicas = typeof spec?.replicas === 'number' ? spec.replicas : 1
  const serviceName = spec?.serviceName || ''
  const podManagementPolicy =
    spec?.podManagementPolicy || KUBERNETES_STATEFULSET_POD_MANAGEMENT_POLICY.OrderedReady

  const strat = spec?.updateStrategy
  const strategyType = strat?.type || KUBERNETES_STATEFULSET_UPDATE_STRATEGY.RollingUpdate
  const rolling = strat?.rollingUpdate

  const partition = typeof rolling?.partition === 'number' ? rolling.partition : 0
  const maxUnavailable =
    rolling?.maxUnavailable !== undefined && rolling?.maxUnavailable !== null
      ? String(rolling.maxUnavailable)
      : '1'

  const minReadySeconds = typeof spec?.minReadySeconds === 'number' ? spec.minReadySeconds : 0
  const revisionHistoryLimit =
    typeof spec?.revisionHistoryLimit === 'number' ? spec.revisionHistoryLimit : 10

  const matchLabels = spec?.selector?.matchLabels || {}
  const selectorLabels = Object.entries(matchLabels).map(([key, value]) => ({
    key,
    value: String(value ?? '')
  }))

  return {
    replicas,
    serviceName,
    podManagementPolicy,
    strategyType,
    partition,
    maxUnavailable,
    minReadySeconds,
    revisionHistoryLimit,
    selectorLabels
  }
}
