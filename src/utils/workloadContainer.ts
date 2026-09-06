import type {
  Container,
  EnvFromSource,
  EnvVar,
  ResourceRequirements
} from 'kubernetes-types/core/v1'

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

export interface WorkloadContainerPort {
  name: string
  containerPort: number
  protocol: string
}

export interface WorkloadContainerFormState {
  name: string
  image: string
  imagePullPolicy: string
  workingDir: string
  command: string[]
  args: string[]
  environments: ContainerEnvFormItem[]
  ports: WorkloadContainerPort[]
  cpuRequest: string
  memoryRequest: string
  cpuLimit: string
  memoryLimit: string
  rawContainer: Container
}

export interface WorkloadContainerValidationItem {
  name?: string
  workingDir?: string
  ports?: Array<{ name?: string }>
}

/**
 * Parses raw Kubernetes containers into form state for Container editors.
 */
export function parseContainersFromPodSpec(containers?: Container[]): WorkloadContainerFormState[] {
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
    const parsedPorts: WorkloadContainerPort[] = portList.map((p) => ({
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
  formContainers: WorkloadContainerFormState[]
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

    // Environments serialization
    const regularEnvs: EnvVar[] = []
    const envFroms: EnvFromSource[] = []

    for (const item of c.environments) {
      if (item.type === 'Literal' && item.name) {
        regularEnvs.push({
          name: item.name,
          value: item.value ?? ''
        })
      } else if (item.type === 'ConfigMapKey' && item.name && item.refName && item.refKey) {
        regularEnvs.push({
          name: item.name,
          valueFrom: {
            configMapKeyRef: {
              name: item.refName,
              key: item.refKey
            }
          }
        })
      } else if (item.type === 'SecretKey' && item.name && item.refName && item.refKey) {
        regularEnvs.push({
          name: item.name,
          valueFrom: {
            secretKeyRef: {
              name: item.refName,
              key: item.refKey
            }
          }
        })
      } else if (item.type === 'FieldRef' && item.name && item.fieldPath) {
        regularEnvs.push({
          name: item.name,
          valueFrom: {
            fieldRef: {
              fieldPath: item.fieldPath
            }
          }
        })
      } else if (item.type === 'ResourceFieldRef' && item.name && item.fieldPath) {
        regularEnvs.push({
          name: item.name,
          valueFrom: {
            resourceFieldRef: {
              resource: item.fieldPath
            }
          }
        })
      } else if (item.type === 'ConfigMapEnvFrom' && item.refName) {
        envFroms.push({
          prefix: item.prefix || undefined,
          configMapRef: {
            name: item.refName
          }
        })
      } else if (item.type === 'SecretEnvFrom' && item.refName) {
        envFroms.push({
          prefix: item.prefix || undefined,
          secretRef: {
            name: item.refName
          }
        })
      }
    }

    if (regularEnvs.length > 0) {
      containerObj.env = regularEnvs
    } else {
      delete containerObj.env
    }

    if (envFroms.length > 0) {
      containerObj.envFrom = envFroms
    } else {
      delete containerObj.envFrom
    }

    // Ports serialization
    if (c.ports.length > 0) {
      containerObj.ports = c.ports.map((p) => {
        const portObj: { name?: string; containerPort: number; protocol?: string } = {
          containerPort: p.containerPort
        }
        if (p.name && p.name.trim()) portObj.name = p.name.trim()
        if (p.protocol) portObj.protocol = p.protocol
        return portObj
      })
    } else {
      delete containerObj.ports
    }

    // Compute resources serialization
    const resourcesObj: ResourceRequirements = {}
    let hasResources = false

    if (c.cpuRequest.trim() || c.memoryRequest.trim()) {
      resourcesObj.requests = {}
      if (c.cpuRequest.trim()) resourcesObj.requests.cpu = c.cpuRequest.trim()
      if (c.memoryRequest.trim()) resourcesObj.requests.memory = c.memoryRequest.trim()
      hasResources = true
    }

    if (c.cpuLimit.trim() || c.memoryLimit.trim()) {
      resourcesObj.limits = {}
      if (c.cpuLimit.trim()) resourcesObj.limits.cpu = c.cpuLimit.trim()
      if (c.memoryLimit.trim()) resourcesObj.limits.memory = c.memoryLimit.trim()
      hasResources = true
    }

    if (hasResources) {
      containerObj.resources = resourcesObj
    } else {
      delete containerObj.resources
    }

    return containerObj
  })
}
