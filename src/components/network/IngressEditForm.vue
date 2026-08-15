<script setup lang="ts">
import { Plus, Trash2 } from '@lucide/vue'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import Tab from 'primevue/tab'
import TabList from 'primevue/tablist'
import TabPanel from 'primevue/tabpanel'
import TabPanels from 'primevue/tabpanels'
import Tabs from 'primevue/tabs'
import { ref, toRaw, watch } from 'vue'

import KeyValueEditor from '@/components/shared/KeyValueEditor.vue'

const props = defineProps<{
  rawData: Record<string, unknown> | null
}>()

const emit = defineEmits<{
  (e: 'update:rawData', value: Record<string, unknown>): void
}>()

const activeTab = ref('general')

// General
const ingressClassName = ref<string>('')
const defaultBackendServiceName = ref<string>('')
const defaultBackendServicePort = ref<string>('')

// Rules
interface IngressRuleRow {
  host: string
  path: string
  pathType: 'Prefix' | 'Exact' | 'ImplementationSpecific'
  serviceName: string
  servicePort: string
}

const pathTypeOptions = ['Prefix', 'Exact', 'ImplementationSpecific']
const rules = ref<IngressRuleRow[]>([])

// TLS
interface IngressTlsRow {
  secretName: string
  hosts: string
}

const tlsConfigs = ref<IngressTlsRow[]>([])

// Metadata
const labels = ref<{ key: string; value: string }[]>([])
const annotations = ref<{ key: string; value: string }[]>([])

let isEmitting = false

const kvObjectToArray = (
  obj: Record<string, unknown> | undefined
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

const syncFromRawData = (data: Record<string, unknown> | null) => {
  if (!data || isEmitting) return

  // Metadata
  const metadata = (data.metadata as Record<string, unknown>) || {}
  labels.value = kvObjectToArray(metadata.labels as Record<string, unknown>)
  annotations.value = kvObjectToArray(metadata.annotations as Record<string, unknown>)

  // Spec
  const spec = (data.spec as Record<string, unknown>) || {}
  ingressClassName.value = typeof spec.ingressClassName === 'string' ? spec.ingressClassName : ''

  // Default Backend
  const defaultBackend = (spec.defaultBackend as Record<string, unknown>) || {}
  const defaultService = (defaultBackend.service as Record<string, unknown>) || {}
  defaultBackendServiceName.value =
    typeof defaultService.name === 'string' ? defaultService.name : ''
  const defaultPort = (defaultService.port as Record<string, unknown>) || {}
  defaultBackendServicePort.value =
    typeof defaultPort.number === 'number'
      ? String(defaultPort.number)
      : typeof defaultPort.name === 'string'
        ? defaultPort.name
        : ''

  // Rules
  const rawRules = Array.isArray(spec.rules) ? (spec.rules as Record<string, unknown>[]) : []
  const parsedRules: IngressRuleRow[] = []

  for (const rule of rawRules) {
    const host = typeof rule.host === 'string' ? rule.host : ''
    const http = (rule.http as Record<string, unknown>) || {}
    const paths = Array.isArray(http.paths) ? (http.paths as Record<string, unknown>[]) : []

    if (paths.length === 0) {
      parsedRules.push({
        host,
        path: '/',
        pathType: 'Prefix',
        serviceName: '',
        servicePort: '80'
      })
    } else {
      for (const p of paths) {
        const pathStr = typeof p.path === 'string' ? p.path : '/'
        const rawPathType = typeof p.pathType === 'string' ? p.pathType : 'Prefix'
        const pathType = pathTypeOptions.includes(rawPathType)
          ? (rawPathType as IngressRuleRow['pathType'])
          : 'Prefix'

        const backend = (p.backend as Record<string, unknown>) || {}
        const service = (backend.service as Record<string, unknown>) || {}
        const serviceName = typeof service.name === 'string' ? service.name : ''
        const portObj = (service.port as Record<string, unknown>) || {}
        const servicePort =
          typeof portObj.number === 'number'
            ? String(portObj.number)
            : typeof portObj.name === 'string'
              ? portObj.name
              : '80'

        parsedRules.push({
          host,
          path: pathStr,
          pathType,
          serviceName,
          servicePort
        })
      }
    }
  }

  rules.value = parsedRules

  // TLS
  const rawTls = Array.isArray(spec.tls) ? (spec.tls as Record<string, unknown>[]) : []
  tlsConfigs.value = rawTls.map((t) => {
    const secretName = typeof t.secretName === 'string' ? t.secretName : ''
    const hostsArr = Array.isArray(t.hosts) ? (t.hosts as string[]) : []
    return {
      secretName,
      hosts: hostsArr.join(', ')
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

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const rawObj = JSON.parse(JSON.stringify(toRaw(props.rawData))) as Record<string, any>
  if (!rawObj.metadata) rawObj.metadata = {}
  if (!rawObj.spec) rawObj.spec = {}

  // 1. Metadata
  rawObj.metadata.labels = kvArrayToObject(labels.value)
  rawObj.metadata.annotations = kvArrayToObject(annotations.value)

  // 2. General / IngressClassName
  if (ingressClassName.value.trim()) {
    rawObj.spec.ingressClassName = ingressClassName.value.trim()
  } else {
    delete rawObj.spec.ingressClassName
  }

  // 3. Default Backend
  if (defaultBackendServiceName.value.trim()) {
    const portVal = defaultBackendServicePort.value
    const portObj: Record<string, unknown> = {}
    if (typeof portVal === 'number' || (typeof portVal === 'string' && /^\d+$/.test(portVal))) {
      portObj.number = Number(portVal)
    } else if (typeof portVal === 'string' && portVal.trim()) {
      portObj.name = portVal.trim()
    } else {
      portObj.number = 80
    }

    rawObj.spec.defaultBackend = {
      service: {
        name: defaultBackendServiceName.value.trim(),
        port: portObj
      }
    }
  } else {
    delete rawObj.spec.defaultBackend
  }

  // 4. Rules
  if (rules.value.length > 0) {
    // Group paths by host
    const hostMap = new Map<string, IngressRuleRow[]>()
    for (const r of rules.value) {
      const h = r.host.trim()
      if (!hostMap.has(h)) {
        hostMap.set(h, [])
      }
      hostMap.get(h)!.push(r)
    }

    const generatedRules: Record<string, unknown>[] = []
    for (const [host, hostRules] of hostMap.entries()) {
      const paths = hostRules
        .filter((r) => r.serviceName.trim() !== '')
        .map((r) => {
          const portVal = r.servicePort
          const portObj: Record<string, unknown> = {}
          if (
            typeof portVal === 'number' ||
            (typeof portVal === 'string' && /^\d+$/.test(portVal))
          ) {
            portObj.number = Number(portVal)
          } else if (typeof portVal === 'string' && portVal.trim()) {
            portObj.name = portVal.trim()
          } else {
            portObj.number = 80
          }

          return {
            path: r.path.trim() || '/',
            pathType: r.pathType,
            backend: {
              service: {
                name: r.serviceName.trim(),
                port: portObj
              }
            }
          }
        })

      if (paths.length > 0) {
        const ruleItem: Record<string, unknown> = {
          http: {
            paths
          }
        }
        if (host) {
          ruleItem.host = host
        }
        generatedRules.push(ruleItem)
      }
    }

    if (generatedRules.length > 0) {
      rawObj.spec.rules = generatedRules
    } else {
      delete rawObj.spec.rules
    }
  } else {
    delete rawObj.spec.rules
  }

  // 5. TLS
  const validTls = tlsConfigs.value
    .filter((t) => t.secretName.trim() !== '' || t.hosts.trim() !== '')
    .map((t) => {
      const item: Record<string, unknown> = {}
      if (t.secretName.trim()) {
        item.secretName = t.secretName.trim()
      }
      const hostList = t.hosts
        .split(',')
        .map((h) => h.trim())
        .filter(Boolean)
      if (hostList.length > 0) {
        item.hosts = hostList
      }
      return item
    })

  if (validTls.length > 0) {
    rawObj.spec.tls = validTls
  } else {
    delete rawObj.spec.tls
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

// Rules operations
const addRuleRow = () => {
  rules.value.push({
    host: '',
    path: '/',
    pathType: 'Prefix',
    serviceName: '',
    servicePort: '80'
  })
  handleFieldChange()
}

const removeRuleRow = (index: number) => {
  rules.value.splice(index, 1)
  handleFieldChange()
}

// TLS operations
const addTlsRow = () => {
  tlsConfigs.value.push({
    secretName: '',
    hosts: ''
  })
  handleFieldChange()
}

const removeTlsRow = (index: number) => {
  tlsConfigs.value.splice(index, 1)
  handleFieldChange()
}
</script>

<template>
  <div class="flex flex-col h-full w-full">
    <Tabs v-model:value="activeTab" class="w-full flex flex-col h-full">
      <TabList>
        <Tab value="general" class="text-xs font-medium">General</Tab>
        <Tab value="rules" class="text-xs font-medium">Rules</Tab>
        <Tab value="tls" class="text-xs font-medium">TLS</Tab>
        <Tab value="metadata" class="text-xs font-medium">Metadata</Tab>
      </TabList>

      <TabPanels class="flex-1 overflow-y-auto pt-6 px-0">
        <!-- GENERAL TAB -->
        <TabPanel value="general" class="flex flex-col gap-10 max-w-7xl">
          <!-- Section 1: Ingress Class -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Ingress Class
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Specifies which Ingress controller should implement the rules (e.g.
                <code class="text-primary font-mono">nginx</code>,
                <code class="text-primary font-mono">traefik</code>).
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-3">
              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color">Ingress Class Name</label>
                <InputText
                  v-model="ingressClassName"
                  placeholder="e.g. nginx"
                  size="small"
                  class="w-full md:w-80"
                  @input="handleFieldChange"
                />
              </div>
            </div>
          </div>

          <!-- Section 2: Default Backend -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Default Backend
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Service that handles requests that do not match any specified rule paths.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-4">
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Service Name</label>
                  <InputText
                    v-model="defaultBackendServiceName"
                    placeholder="e.g. default-backend-svc"
                    size="small"
                    fluid
                    @input="handleFieldChange"
                  />
                </div>
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Service Port</label>
                  <InputText
                    v-model="defaultBackendServicePort"
                    placeholder="e.g. 80 or http"
                    size="small"
                    fluid
                    @input="handleFieldChange"
                  />
                </div>
              </div>
            </div>
          </div>
        </TabPanel>

        <!-- RULES TAB -->
        <TabPanel value="rules" class="flex flex-col gap-10 max-w-7xl">
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Routing Rules
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Configure HTTP routing paths mapping incoming domain hosts and URI paths to backend
                target services.
              </p>
            </div>

            <div class="md:col-span-8 flex flex-col gap-4">
              <div class="flex items-center justify-between">
                <span class="text-xs font-medium text-muted-color"
                  >{{ rules.length }} Rule{{ rules.length === 1 ? '' : 's' }}</span
                >
                <Button
                  size="small"
                  variant="text"
                  label="Add Path Rule"
                  class="text-xs"
                  @click="addRuleRow"
                >
                  <template #icon>
                    <Plus class="w-3.5 h-3.5 mr-1" />
                  </template>
                </Button>
              </div>

              <div v-if="rules.length === 0" class="text-xs text-muted-color italic py-2">
                No routing rules configured.
              </div>

              <div
                v-for="(rule, idx) in rules"
                :key="'rule-' + idx"
                class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3"
              >
                <div class="flex items-center justify-between border-b border-(--border) pb-2">
                  <span class="text-xs font-semibold text-primary font-mono"
                    >Rule #{{ idx + 1 }}</span
                  >
                  <Button
                    variant="text"
                    severity="danger"
                    size="small"
                    class="p-1! text-muted-color hover:text-rose-500 cursor-pointer"
                    aria-label="Remove Rule"
                    @click="removeRuleRow(idx)"
                  >
                    <template #icon>
                      <Trash2 class="w-4 h-4" />
                    </template>
                  </Button>
                </div>

                <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  <div class="flex flex-col gap-1">
                    <label class="text-[11px] font-medium text-muted-color"
                      >Host (Empty for all)</label
                    >
                    <InputText
                      v-model="rule.host"
                      placeholder="e.g. app.example.com"
                      size="small"
                      fluid
                      @input="handleFieldChange"
                    />
                  </div>
                  <div class="flex flex-col gap-1">
                    <label class="text-[11px] font-medium text-muted-color">Path</label>
                    <InputText
                      v-model="rule.path"
                      placeholder="e.g. / or /api"
                      size="small"
                      fluid
                      @input="handleFieldChange"
                    />
                  </div>
                </div>

                <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                  <div class="flex flex-col gap-1">
                    <label class="text-[11px] font-medium text-muted-color">Path Type</label>
                    <Select
                      v-model="rule.pathType"
                      :options="pathTypeOptions"
                      size="small"
                      fluid
                      @change="handleFieldChange"
                    />
                  </div>
                  <div class="flex flex-col gap-1">
                    <label class="text-[11px] font-medium text-muted-color">Service Name</label>
                    <InputText
                      v-model="rule.serviceName"
                      placeholder="e.g. my-service"
                      size="small"
                      fluid
                      @input="handleFieldChange"
                    />
                  </div>
                  <div class="flex flex-col gap-1">
                    <label class="text-[11px] font-medium text-muted-color">Port</label>
                    <InputText
                      v-model="rule.servicePort"
                      placeholder="e.g. 80 or http"
                      size="small"
                      fluid
                      @input="handleFieldChange"
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </TabPanel>

        <!-- TLS TAB -->
        <TabPanel value="tls" class="flex flex-col gap-10 max-w-7xl">
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                TLS Termination
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Specify TLS certificates and their associated host names for HTTPS termination.
              </p>
            </div>

            <div class="md:col-span-8 flex flex-col gap-4">
              <div class="flex items-center justify-between">
                <span class="text-xs font-medium text-muted-color"
                  >{{ tlsConfigs.length }} Certificate{{ tlsConfigs.length === 1 ? '' : 's' }}</span
                >
                <Button
                  size="small"
                  variant="text"
                  label="Add TLS Certificate"
                  class="text-xs"
                  @click="addTlsRow"
                >
                  <template #icon>
                    <Plus class="w-3.5 h-3.5 mr-1" />
                  </template>
                </Button>
              </div>

              <div v-if="tlsConfigs.length === 0" class="text-xs text-muted-color italic py-2">
                No TLS certificates configured.
              </div>

              <div
                v-for="(tls, idx) in tlsConfigs"
                :key="'tls-' + idx"
                class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3"
              >
                <div class="flex items-center justify-between border-b border-(--border) pb-2">
                  <span class="text-xs font-semibold text-primary font-mono"
                    >TLS Cert #{{ idx + 1 }}</span
                  >
                  <Button
                    variant="text"
                    severity="danger"
                    size="small"
                    class="p-1! text-muted-color hover:text-rose-500 cursor-pointer"
                    aria-label="Remove TLS"
                    @click="removeTlsRow(idx)"
                  >
                    <template #icon>
                      <Trash2 class="w-4 h-4" />
                    </template>
                  </Button>
                </div>

                <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  <div class="flex flex-col gap-1">
                    <label class="text-[11px] font-medium text-muted-color">Secret Name</label>
                    <InputText
                      v-model="tls.secretName"
                      placeholder="e.g. tls-secret"
                      size="small"
                      fluid
                      @input="handleFieldChange"
                    />
                  </div>
                  <div class="flex flex-col gap-1">
                    <label class="text-[11px] font-medium text-muted-color"
                      >Hosts (comma-separated)</label
                    >
                    <InputText
                      v-model="tls.hosts"
                      placeholder="e.g. example.com, api.example.com"
                      size="small"
                      fluid
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
                Ingress Metadata
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Labels and annotations attached to the Ingress resource (such as ingress-controller
                annotations).
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-6">
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="labels"
                  title="Labels"
                  add-label="Add Label"
                  @update:model-value="handleFieldChange"
                />
              </div>
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="annotations"
                  title="Annotations"
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
