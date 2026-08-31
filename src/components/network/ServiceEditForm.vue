<script setup lang="ts">
import { Plus, Trash2 } from '@lucide/vue'
import Button from 'primevue/button'
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import Tab from 'primevue/tab'
import TabList from 'primevue/tablist'
import TabPanel from 'primevue/tabpanel'
import TabPanels from 'primevue/tabpanels'
import Tabs from 'primevue/tabs'
import { computed, ref, toRaw, watch } from 'vue'

import KeyValueEditor from '@/components/shared/KeyValueEditor.vue'
import { KUBERNETES_SERVICE_TYPE, type KubernetesServiceType } from '@/constants/kubernetes'
import { isValidHost, isValidK8sLabel, isValidPort } from '@/utils/validators'
import type { Service, ServicePort, ServiceSpec } from 'kubernetes-types/core/v1'

const props = defineProps<{
  rawData: Service | null
}>()

const emit = defineEmits<{
  (e: 'update:rawData', value: Service): void
  (e: 'update:isValid', value: boolean): void
}>()

const activeTab = ref('general')

// Types and options
const serviceTypeOptions: KubernetesServiceType[] = Object.values(KUBERNETES_SERVICE_TYPE)
const sessionAffinityOptions = ['None', 'ClientIP']
const externalTrafficPolicyOptions = ['Cluster', 'Local']
const protocolOptions = ['TCP', 'UDP', 'SCTP']

// General Spec
const serviceType = ref<KubernetesServiceType | string>(KUBERNETES_SERVICE_TYPE.ClusterIP)
const clusterIP = ref<string>('')
const externalName = ref<string>('')
const sessionAffinity = ref<string>('None')
const sessionAffinityTimeout = ref<number | null>(null)
const externalTrafficPolicy = ref<string>('Cluster')
const loadBalancerIP = ref<string>('')

// Selector
const selectorList = ref<{ key: string; value: string }[]>([])

// Ports
interface ServicePortRow {
  id: string
  name: string
  protocol: 'TCP' | 'UDP' | 'SCTP'
  port: number | null
  targetPort: string
  nodePort: number | null
  appProtocol: string
}

const portsList = ref<ServicePortRow[]>([])

// Metadata
const labels = ref<{ key: string; value: string }[]>([])
const annotations = ref<{ key: string; value: string }[]>([])

let isEmitting = false

// Validation helpers
const getPortNameError = (name: string, index: number): string | null => {
  const trimmed = name.trim()
  if (!trimmed) {
    if (portsList.value.length > 1) {
      return 'Port name is required when multiple ports are defined.'
    }
    return null
  }
  if (!isValidK8sLabel(trimmed)) {
    return 'Port name must be a valid DNS label (lowercase alphanumeric, max 63 chars).'
  }
  const firstIndex = portsList.value.findIndex(
    (p) => p.name.trim().toLowerCase() === trimmed.toLowerCase()
  )
  if (firstIndex !== -1 && firstIndex !== index) {
    return `Duplicate port name "${trimmed}".`
  }
  return null
}

const getPortNumberError = (
  portVal: number | null,
  protocol: string,
  index: number
): string | null => {
  if (portVal === null || portVal === undefined) {
    return 'Port is required.'
  }
  if (!isValidPort(portVal)) {
    return 'Port must be between 1 and 65535.'
  }
  const firstIndex = portsList.value.findIndex((p) => p.port === portVal && p.protocol === protocol)
  if (firstIndex !== -1 && firstIndex !== index) {
    return `Duplicate port ${portVal}/${protocol}.`
  }
  return null
}

const getTargetPortError = (targetPortVal: string): string | null => {
  const trimmed = targetPortVal.trim()
  if (!trimmed) return null
  if (/^\d+$/.test(trimmed)) {
    const num = parseInt(trimmed, 10)
    if (!isValidPort(num)) {
      return 'Target port number must be between 1 and 65535.'
    }
    return null
  }
  if (!isValidK8sLabel(trimmed)) {
    return 'Named target port must be a valid IANA service name/DNS label.'
  }
  return null
}

const getNodePortError = (nodePortVal: number | null, index: number): string | null => {
  if (
    serviceType.value !== KUBERNETES_SERVICE_TYPE.NodePort &&
    serviceType.value !== KUBERNETES_SERVICE_TYPE.LoadBalancer
  ) {
    return null
  }
  if (nodePortVal === null || nodePortVal === undefined) {
    return null
  }
  if (!isValidPort(nodePortVal)) {
    return 'NodePort must be between 1 and 65535.'
  }
  const firstIndex = portsList.value.findIndex((p) => p.nodePort === nodePortVal)
  if (firstIndex !== -1 && firstIndex !== index) {
    return `Duplicate NodePort ${nodePortVal}.`
  }
  return null
}

const externalNameError = computed(() => {
  if (serviceType.value !== KUBERNETES_SERVICE_TYPE.ExternalName) return null
  const trimmed = externalName.value.trim()
  if (!trimmed) {
    return 'External name is required for ExternalName services.'
  }
  if (!isValidHost(trimmed)) {
    return 'Must be a valid hostname (e.g. my.database.example.com).'
  }
  return null
})

const isFormValid = computed(() => {
  if (externalNameError.value) return false

  if (sessionAffinity.value === 'ClientIP' && sessionAffinityTimeout.value !== null) {
    if (sessionAffinityTimeout.value <= 0 || sessionAffinityTimeout.value > 86400) {
      return false
    }
  }

  for (let i = 0; i < portsList.value.length; i++) {
    const p = portsList.value[i]
    if (!p) continue
    if (getPortNameError(p.name, i)) return false
    if (getPortNumberError(p.port, p.protocol, i)) return false
    if (getTargetPortError(p.targetPort)) return false
    if (getNodePortError(p.nodePort, i)) return false
  }

  return true
})

watch(
  isFormValid,
  (val) => {
    emit('update:isValid', val)
  },
  { immediate: true }
)

const kvObjectToArray = (
  obj: Record<string, string> | undefined
): { key: string; value: string }[] => {
  if (!obj || typeof obj !== 'object') return []
  return Object.entries(obj).map(([key, value]) => ({ key, value: String(value ?? '') }))
}

const kvArrayToObject = (arr: { key: string; value: string }[]): Record<string, string> => {
  const res: Record<string, string> = {}
  for (const item of arr) {
    if (item.key.trim()) {
      res[item.key.trim()] = item.value
    }
  }
  return res
}

const syncFromRawData = (data: Service | null) => {
  if (!data || isEmitting) return

  // Metadata
  labels.value = kvObjectToArray(data.metadata?.labels)
  annotations.value = kvObjectToArray(data.metadata?.annotations)

  // Spec
  const spec = data.spec || {}
  serviceType.value = spec.type || KUBERNETES_SERVICE_TYPE.ClusterIP
  clusterIP.value = spec.clusterIP || ''
  externalName.value = spec.externalName || ''
  sessionAffinity.value = spec.sessionAffinity || 'None'
  sessionAffinityTimeout.value = spec.sessionAffinityConfig?.clientIP?.timeoutSeconds ?? null
  externalTrafficPolicy.value = spec.externalTrafficPolicy || 'Cluster'
  loadBalancerIP.value = spec.loadBalancerIP || ''

  // Selector
  selectorList.value = kvObjectToArray(spec.selector)

  // Ports
  const rawPorts = spec.ports || []
  portsList.value = rawPorts.map((p) => {
    let tpStr = ''
    if (typeof p.targetPort === 'number') {
      tpStr = String(p.targetPort)
    } else if (typeof p.targetPort === 'string') {
      tpStr = p.targetPort
    }

    return {
      id: crypto.randomUUID(),
      name: p.name || '',
      protocol: (p.protocol as 'TCP' | 'UDP' | 'SCTP') || 'TCP',
      port: typeof p.port === 'number' ? p.port : null,
      targetPort: tpStr,
      nodePort: typeof p.nodePort === 'number' ? p.nodePort : null,
      appProtocol: p.appProtocol || ''
    }
  })
}

watch(
  () => props.rawData,
  (newData) => {
    syncFromRawData(newData)
  },
  { immediate: true, deep: true }
)

const emitUpdate = () => {
  if (!props.rawData) return
  isEmitting = true

  const rawObj = JSON.parse(JSON.stringify(toRaw(props.rawData))) as Service
  if (!rawObj.metadata) rawObj.metadata = {}
  if (!rawObj.spec) rawObj.spec = {}

  // 1. Metadata
  rawObj.metadata.labels = kvArrayToObject(labels.value)
  rawObj.metadata.annotations = kvArrayToObject(annotations.value)

  // 2. General Spec
  rawObj.spec.type = serviceType.value as ServiceSpec['type']

  if (serviceType.value === KUBERNETES_SERVICE_TYPE.ExternalName) {
    if (externalName.value.trim()) {
      rawObj.spec.externalName = externalName.value.trim()
    } else {
      delete rawObj.spec.externalName
    }
    delete rawObj.spec.clusterIP
    delete rawObj.spec.clusterIPs
    delete rawObj.spec.loadBalancerIP
    delete rawObj.spec.externalTrafficPolicy
  } else {
    delete rawObj.spec.externalName

    if (clusterIP.value.trim()) {
      rawObj.spec.clusterIP = clusterIP.value.trim()
    }

    if (
      serviceType.value === KUBERNETES_SERVICE_TYPE.NodePort ||
      serviceType.value === KUBERNETES_SERVICE_TYPE.LoadBalancer
    ) {
      rawObj.spec.externalTrafficPolicy = externalTrafficPolicy.value
    } else {
      delete rawObj.spec.externalTrafficPolicy
    }

    if (serviceType.value === KUBERNETES_SERVICE_TYPE.LoadBalancer && loadBalancerIP.value.trim()) {
      rawObj.spec.loadBalancerIP = loadBalancerIP.value.trim()
    } else {
      delete rawObj.spec.loadBalancerIP
    }
  }

  // Session affinity
  rawObj.spec.sessionAffinity = sessionAffinity.value
  if (sessionAffinity.value === 'ClientIP' && sessionAffinityTimeout.value !== null) {
    rawObj.spec.sessionAffinityConfig = {
      clientIP: {
        timeoutSeconds: sessionAffinityTimeout.value
      }
    }
  } else {
    delete rawObj.spec.sessionAffinityConfig
  }

  // 3. Selector
  const selectorObj = kvArrayToObject(selectorList.value)
  if (
    Object.keys(selectorObj).length > 0 &&
    serviceType.value !== KUBERNETES_SERVICE_TYPE.ExternalName
  ) {
    rawObj.spec.selector = selectorObj
  } else {
    delete rawObj.spec.selector
  }

  // 4. Ports
  if (portsList.value.length > 0) {
    const validPorts: ServicePort[] = portsList.value
      .filter((p) => p.port !== null && isValidPort(p.port))
      .map((p) => {
        const item: ServicePort = {
          port: p.port as number,
          protocol: p.protocol || 'TCP'
        }
        if (p.name.trim()) {
          item.name = p.name.trim()
        }

        const tpTrimmed = p.targetPort.trim()
        if (tpTrimmed) {
          if (/^\d+$/.test(tpTrimmed)) {
            item.targetPort = parseInt(tpTrimmed, 10)
          } else {
            item.targetPort = tpTrimmed
          }
        }

        if (
          (serviceType.value === KUBERNETES_SERVICE_TYPE.NodePort ||
            serviceType.value === KUBERNETES_SERVICE_TYPE.LoadBalancer) &&
          p.nodePort !== null &&
          isValidPort(p.nodePort)
        ) {
          item.nodePort = p.nodePort
        }

        if (p.appProtocol.trim()) {
          item.appProtocol = p.appProtocol.trim()
        }

        return item
      })

    if (validPorts.length > 0) {
      rawObj.spec.ports = validPorts
    } else {
      delete rawObj.spec.ports
    }
  } else {
    delete rawObj.spec.ports
  }

  emit('update:rawData', rawObj)

  setTimeout(() => {
    isEmitting = false
  }, 50)
}

const handleFieldChange = () => {
  if (!isEmitting) {
    emitUpdate()
  }
}

// Ports operations
const addPort = () => {
  portsList.value.push({
    id: crypto.randomUUID(),
    name: portsList.value.length > 0 ? `port-${portsList.value.length + 1}` : 'http',
    protocol: 'TCP',
    port: 80,
    targetPort: '80',
    nodePort: null,
    appProtocol: ''
  })
  handleFieldChange()
}

const removePort = (index: number) => {
  portsList.value.splice(index, 1)
  handleFieldChange()
}
</script>

<template>
  <div class="flex flex-col h-full w-full">
    <Tabs v-model:value="activeTab" class="w-full flex flex-col h-full">
      <TabList>
        <Tab value="general" class="text-xs font-medium">General</Tab>
        <Tab value="selector" class="text-xs font-medium">Selector</Tab>
        <Tab value="ports" class="text-xs font-medium">Ports</Tab>
        <Tab value="metadata" class="text-xs font-medium">Metadata</Tab>
      </TabList>

      <TabPanels class="flex-1 overflow-y-auto pt-6 px-0">
        <!-- GENERAL TAB -->
        <TabPanel value="general" class="flex flex-col gap-10 max-w-7xl">
          <!-- Section 1: Service Type & Networking -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Service Type
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Determines how the Service is exposed to cluster internal or external traffic.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-4">
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Type</label>
                  <Select
                    v-model="serviceType"
                    :options="serviceTypeOptions"
                    size="small"
                    fluid
                    class="text-xs"
                    @change="handleFieldChange"
                  />
                </div>

                <div
                  v-if="serviceType !== KUBERNETES_SERVICE_TYPE.ExternalName"
                  class="flex flex-col gap-1.5"
                >
                  <label class="text-xs font-medium text-muted-color">Cluster IP</label>
                  <InputText
                    v-model="clusterIP"
                    placeholder="e.g. 10.96.0.1 or None for headless"
                    size="small"
                    fluid
                    class="text-xs"
                    @input="handleFieldChange"
                  />
                </div>

                <div v-else class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">
                    External Name <span class="text-(--danger)">*</span>
                  </label>
                  <InputText
                    v-model="externalName"
                    placeholder="e.g. db.example.com"
                    :invalid="Boolean(externalNameError)"
                    size="small"
                    fluid
                    class="text-xs"
                    @input="handleFieldChange"
                  />
                  <small v-if="externalNameError" class="text-(--danger) text-[11px] leading-tight">
                    {{ externalNameError }}
                  </small>
                </div>
              </div>

              <!-- Extra fields for LoadBalancer / NodePort -->
              <div
                v-if="
                  serviceType === KUBERNETES_SERVICE_TYPE.NodePort ||
                  serviceType === KUBERNETES_SERVICE_TYPE.LoadBalancer
                "
                class="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-2"
              >
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color"
                    >External Traffic Policy</label
                  >
                  <Select
                    v-model="externalTrafficPolicy"
                    :options="externalTrafficPolicyOptions"
                    size="small"
                    fluid
                    class="text-xs"
                    @change="handleFieldChange"
                  />
                </div>

                <div
                  v-if="serviceType === KUBERNETES_SERVICE_TYPE.LoadBalancer"
                  class="flex flex-col gap-1.5"
                >
                  <label class="text-xs font-medium text-muted-color">LoadBalancer IP</label>
                  <InputText
                    v-model="loadBalancerIP"
                    placeholder="e.g. 198.51.100.1"
                    size="small"
                    fluid
                    class="text-xs"
                    @input="handleFieldChange"
                  />
                </div>
              </div>
            </div>
          </div>

          <!-- Section 2: Session Affinity -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Session Affinity
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Sticky sessions configuration based on client IP addresses.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-4">
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Affinity Mode</label>
                  <Select
                    v-model="sessionAffinity"
                    :options="sessionAffinityOptions"
                    size="small"
                    fluid
                    class="text-xs"
                    @change="handleFieldChange"
                  />
                </div>

                <div v-if="sessionAffinity === 'ClientIP'" class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">
                    Timeout Seconds (1-86400)
                  </label>
                  <InputNumber
                    v-model="sessionAffinityTimeout"
                    :min="1"
                    :max="86400"
                    placeholder="e.g. 10800 (default 3h)"
                    size="small"
                    fluid
                    @change="handleFieldChange"
                  />
                </div>
              </div>
            </div>
          </div>
        </TabPanel>

        <!-- SELECTOR TAB -->
        <TabPanel value="selector" class="flex flex-col gap-10 max-w-7xl">
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Pod Selector
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Routes network traffic to Pods matching these label key-value pairs. Services
                without selectors route to custom Endpoints.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-3">
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="selectorList"
                  title="Selector Labels"
                  key-placeholder="e.g. app"
                  value-placeholder="e.g. web"
                  add-label="Add Selector"
                  @update:model-value="handleFieldChange"
                />
              </div>
            </div>
          </div>
        </TabPanel>

        <!-- PORTS TAB -->
        <TabPanel value="ports" class="flex flex-col gap-10 max-w-7xl">
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Port Mappings
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Expose service ports and map incoming traffic to target container ports.
              </p>
            </div>

            <div class="md:col-span-8 flex flex-col gap-4">
              <div class="flex items-center justify-between">
                <span class="text-xs font-medium text-muted-color">
                  {{ portsList.length }} Port Definition{{ portsList.length === 1 ? '' : 's' }}
                </span>
                <Button
                  size="small"
                  variant="text"
                  label="Add Port"
                  class="text-xs"
                  @click="addPort"
                >
                  <template #icon>
                    <Plus class="w-3.5 h-3.5 mr-1" />
                  </template>
                </Button>
              </div>

              <div v-if="portsList.length === 0" class="text-xs text-muted-color italic py-2">
                No ports configured.
              </div>

              <div
                v-for="(portItem, pIdx) in portsList"
                :key="portItem.id"
                class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-4"
              >
                <div class="grid grid-cols-1 sm:grid-cols-12 gap-3 items-start">
                  <!-- Name -->
                  <div class="sm:col-span-3 flex flex-col gap-1">
                    <label class="text-[11px] font-medium text-muted-color">Port Name</label>
                    <InputText
                      v-model="portItem.name"
                      placeholder="e.g. http"
                      :invalid="Boolean(getPortNameError(portItem.name, pIdx))"
                      size="small"
                      fluid
                      class="text-xs"
                      @input="handleFieldChange"
                    />
                    <small
                      v-if="getPortNameError(portItem.name, pIdx)"
                      class="text-(--danger) text-[11px] leading-tight"
                    >
                      {{ getPortNameError(portItem.name, pIdx) }}
                    </small>
                  </div>

                  <!-- Protocol -->
                  <div class="sm:col-span-2 flex flex-col gap-1">
                    <label class="text-[11px] font-medium text-muted-color">Protocol</label>
                    <Select
                      v-model="portItem.protocol"
                      :options="protocolOptions"
                      size="small"
                      fluid
                      class="text-xs"
                      @change="handleFieldChange"
                    />
                  </div>

                  <!-- Port -->
                  <div class="sm:col-span-3 flex flex-col gap-1">
                    <label class="text-[11px] font-medium text-muted-color">
                      Port <span class="text-(--danger)">*</span>
                    </label>
                    <InputNumber
                      v-model="portItem.port"
                      :min="1"
                      :max="65535"
                      placeholder="e.g. 80"
                      :invalid="Boolean(getPortNumberError(portItem.port, portItem.protocol, pIdx))"
                      size="small"
                      fluid
                      @change="handleFieldChange"
                    />
                    <small
                      v-if="getPortNumberError(portItem.port, portItem.protocol, pIdx)"
                      class="text-(--danger) text-[11px] leading-tight"
                    >
                      {{ getPortNumberError(portItem.port, portItem.protocol, pIdx) }}
                    </small>
                  </div>

                  <!-- Target Port -->
                  <div class="sm:col-span-3 flex flex-col gap-1">
                    <label class="text-[11px] font-medium text-muted-color">Target Port</label>
                    <InputText
                      v-model="portItem.targetPort"
                      placeholder="e.g. 8080 or http"
                      :invalid="Boolean(getTargetPortError(portItem.targetPort))"
                      size="small"
                      fluid
                      class="text-xs"
                      @input="handleFieldChange"
                    />
                    <small
                      v-if="getTargetPortError(portItem.targetPort)"
                      class="text-(--danger) text-[11px] leading-tight"
                    >
                      {{ getTargetPortError(portItem.targetPort) }}
                    </small>
                  </div>

                  <!-- Delete button -->
                  <div class="sm:col-span-1 flex items-center justify-end sm:pt-5">
                    <Button
                      variant="text"
                      severity="danger"
                      size="small"
                      class="p-1! text-muted-color hover:text-(--danger) cursor-pointer"
                      aria-label="Remove Port"
                      @click="removePort(pIdx)"
                    >
                      <template #icon>
                        <Trash2 class="w-4 h-4" />
                      </template>
                    </Button>
                  </div>
                </div>

                <!-- Secondary row: NodePort & AppProtocol -->
                <div
                  class="grid grid-cols-1 sm:grid-cols-12 gap-3 pt-2 border-t border-(--border)/50"
                >
                  <div
                    v-if="
                      serviceType === KUBERNETES_SERVICE_TYPE.NodePort ||
                      serviceType === KUBERNETES_SERVICE_TYPE.LoadBalancer
                    "
                    class="sm:col-span-4 flex flex-col gap-1"
                  >
                    <label class="text-[11px] font-medium text-muted-color"
                      >NodePort (Optional)</label
                    >
                    <InputNumber
                      v-model="portItem.nodePort"
                      :min="1"
                      :max="65535"
                      placeholder="e.g. 30080"
                      :invalid="Boolean(getNodePortError(portItem.nodePort, pIdx))"
                      size="small"
                      fluid
                      @change="handleFieldChange"
                    />
                    <small
                      v-if="getNodePortError(portItem.nodePort, pIdx)"
                      class="text-(--danger) text-[11px] leading-tight"
                    >
                      {{ getNodePortError(portItem.nodePort, pIdx) }}
                    </small>
                  </div>

                  <div class="sm:col-span-4 flex flex-col gap-1">
                    <label class="text-[11px] font-medium text-muted-color"
                      >App Protocol (Optional)</label
                    >
                    <InputText
                      v-model="portItem.appProtocol"
                      placeholder="e.g. http, https, grpc"
                      size="small"
                      fluid
                      class="text-xs"
                      @input="handleFieldChange"
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </TabPanel>

        <!-- METADATA TAB -->
        <TabPanel value="metadata" class="flex flex-col gap-10 max-w-7xl">
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Service Metadata
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Labels and annotations attached to this Service resource.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-6">
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="labels"
                  title="Service Labels"
                  add-label="Add Label"
                  @update:model-value="handleFieldChange"
                />
              </div>
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="annotations"
                  title="Service Annotations"
                  add-label="Add Annotation"
                  @update:model-value="handleFieldChange"
                />
              </div>
            </div>
          </div>
        </TabPanel>
      </TabPanels>
    </Tabs>
  </div>
</template>
