<script setup lang="ts">
import ResourceYamlTab from '@/components/shared/ResourceYamlTab.vue'
import ServiceEndpointsTab from '@/components/services/ServiceEndpointsTab.vue'
import ServiceOverviewTab from '@/components/services/ServiceOverviewTab.vue'
import ServicePortsTab from '@/components/services/ServicePortsTab.vue'
import ReactiveAge from '@/components/shared/ReactiveAge.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { OrbitEvents } from '@/types/events'
import type { ServiceInfo } from '@/types/kubernetes'
import { Activity, Clock, FileCode, Server, Shield } from '@lucide/vue'
import Drawer from 'primevue/drawer'
import Tab from 'primevue/tab'
import TabList from 'primevue/tablist'
import TabPanel from 'primevue/tabpanel'
import TabPanels from 'primevue/tabpanels'
import Tabs from 'primevue/tabs'
import Tag from 'primevue/tag'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import * as yaml from 'yaml'

const props = defineProps<{
  visible: boolean
  service: ServiceInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const activeTab = ref('overview')

const getTypeSeverity = (type: string) => {
  switch (type) {
    case 'LoadBalancer':
      return 'info'
    case 'ClusterIP':
      return 'success'
    case 'NodePort':
      return 'warn'
    case 'ExternalName':
      return 'contrast'
    default:
      return 'secondary'
  }
}

// Live Raw YAML & JSON Fetching
const rawYamlData = ref<string | null>(null)
const isYamlLoading = ref<boolean>(false)
const copied = ref<boolean>(false)

const fetchRawData = async () => {
  if (!props.service || !props.visible) return
  isYamlLoading.value = true
  try {
    await kubernetesService.getResourceRaw({
      namespace: props.service.namespace,
      kind: 'Service',
      name: props.service.name
    })
  } catch (e) {
    console.error('Failed to fetch raw service YAML:', e)
    isYamlLoading.value = false
  }
}

const handleRawData = (data: { kind?: string; name?: string; data?: unknown }) => {
  if (data && data.kind === 'Service' && data.name === props.service?.name) {
    if (data.data && typeof data.data === 'object') {
      rawYamlData.value = yaml.stringify(data.data)
    } else if (typeof data.data === 'string') {
      rawYamlData.value = data.data
    }
    isYamlLoading.value = false
  }
}

onMounted(() => {
  events.on(OrbitEvents.ResourceRawData, handleRawData)
})

onUnmounted(() => {
  events.off(OrbitEvents.ResourceRawData, handleRawData)
})

watch(
  () => [props.visible, props.service],
  ([visible, service]) => {
    if (visible && service) {
      rawYamlData.value = null
      fetchRawData()
    } else if (!visible) {
      rawYamlData.value = null
      copied.value = false
    }
  },
  { immediate: true }
)

const generateYaml = (s: ServiceInfo) => {
  return `apiVersion: v1
kind: Service
metadata:
  name: ${s.name}
  namespace: ${s.namespace}
  uid: ${s.uid}
  creationTimestamp: "${s.created}"
  labels:
${Object.entries(s.labels)
  .map(([k, v]) => `    ${k}: ${v}`)
  .join('\n')}
spec:
  clusterIP: ${s.clusterIP}
  type: ${s.type}
  sessionAffinity: ${s.sessionAffinity}
  internalTrafficPolicy: ${s.internalTrafficPolicy}
  selector:
${Object.entries(s.selector)
  .map(([k, v]) => `    ${k}: ${v}`)
  .join('\n')}
  ports:
${s.portsList
  .map(
    (p) => `  - port: ${p.port}
    protocol: ${p.protocol}
    targetPort: ${p.targetPort}${p.nodePort ? `\n    nodePort: ${p.nodePort}` : ''}`
  )
  .join('\n')}
status:
  loadBalancer: ${s.type === 'LoadBalancer' ? `\n    ingress:\n    - ip: ${s.externalIP}` : '{}'}
`
}

const displayedYaml = computed(() => {
  if (rawYamlData.value) return rawYamlData.value
  return props.service ? generateYaml(props.service) : ''
})

const copyYaml = async () => {
  if (!displayedYaml.value) return
  try {
    await navigator.clipboard.writeText(displayedYaml.value)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (e) {
    console.error('Failed to copy YAML:', e)
  }
}
</script>

<template>
  <Drawer
    :visible="props.visible"
    position="right"
    class="w-160! bg-(--bg-card)! border-l! border-(--border)!"
    :dismissable="true"
    @update:visible="emit('update:visible', $event)"
  >
    <template #header>
      <div v-if="props.service" class="flex items-center justify-between w-full pr-4">
        <div class="flex items-center gap-3 min-w-0">
          <span class="w-3 h-3 rounded-full shrink-0 animate-pulse bg-emerald-500"></span>
          <div class="min-w-0">
            <div class="flex items-center gap-2">
              <h3
                class="text-base font-bold text-primary font-mono truncate max-w-70"
                :title="props.service.name"
              >
                {{ props.service.name }}
              </h3>
              <Tag
                rounded
                class="font-mono"
                :severity="getTypeSeverity(props.service.type)"
                :value="props.service.type"
              />
            </div>
            <div class="flex items-center gap-2 text-xs text-muted-color font-mono mt-0.5">
              <span>ns: {{ props.service.namespace }}</span>
              <span class="text-muted-color/60">•</span>
              <span class="flex items-center gap-1">
                <Clock class="w-3 h-3" />
                <ReactiveAge :age="props.service.age" />
              </span>
            </div>
          </div>
        </div>
      </div>
    </template>

    <div v-if="props.service" class="flex flex-col h-full">
      <!-- Tab Layout -->
      <Tabs v-model:value="activeTab" class="flex flex-col flex-1 min-h-0">
        <TabList class="bg-transparent! border-b! border-(--border)! px-2">
          <Tab value="overview" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
            <Server class="w-3.5 h-3.5" />
            <span>Overview</span>
          </Tab>
          <Tab value="endpoints" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
            <Shield class="w-3.5 h-3.5" />
            <span>Endpoints ({{ props.service.endpointsList.length }})</span>
          </Tab>
          <Tab value="ports" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
            <Activity class="w-3.5 h-3.5" />
            <span>Ports ({{ props.service.portsList.length }})</span>
          </Tab>
          <Tab value="yaml" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
            <FileCode class="w-3.5 h-3.5" />
            <span>YAML</span>
          </Tab>
        </TabList>

        <TabPanels class="flex-1 overflow-y-auto p-6! bg-transparent!">
          <!-- OVERVIEW PANEL -->
          <TabPanel value="overview">
            <ServiceOverviewTab :service="props.service" />
          </TabPanel>

          <!-- ENDPOINTS PANEL -->
          <TabPanel value="endpoints">
            <ServiceEndpointsTab :service="props.service" />
          </TabPanel>

          <!-- PORTS PANEL -->
          <TabPanel value="ports">
            <ServicePortsTab :service="props.service" />
          </TabPanel>

          <!-- YAML PANEL -->
          <TabPanel value="yaml" class="h-full">
            <ResourceYamlTab
              :displayed-yaml="displayedYaml"
              :is-yaml-loading="isYamlLoading"
              :copied="copied"
              @copy-yaml="copyYaml"
            />
          </TabPanel>
        </TabPanels>
      </Tabs>
    </div>
  </Drawer>
</template>
