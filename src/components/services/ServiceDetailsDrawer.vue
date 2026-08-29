<script setup lang="ts">
import ResourceYamlTab from '@/components/shared/ResourceYamlTab.vue'
import ServiceEndpointsTab from '@/components/services/ServiceEndpointsTab.vue'
import ServiceOverviewTab from '@/components/services/ServiceOverviewTab.vue'
import ServicePortsTab from '@/components/services/ServicePortsTab.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { OrbitEvents } from '@/types/events'
import type { ServiceInfo } from '@/types/kubernetes'
import BaseResourceDrawer from '@/components/shared/BaseResourceDrawer.vue'
import { Activity, Shield } from '@lucide/vue'
import Tab from 'primevue/tab'
import TabPanel from 'primevue/tabpanel'
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
  <BaseResourceDrawer
    v-model:active-tab="activeTab"
    :visible="props.visible"
    :has-resource="!!props.service"
    :title="props.service?.name ?? ''"
    :kind="props.service?.type ?? ''"
    :kind-severity="props.service ? getTypeSeverity(props.service.type) : 'info'"
    status-badge-class="bg-emerald-500"
    :namespace="props.service?.namespace"
    :age="props.service?.age"
    @update:visible="emit('update:visible', $event)"
  >
    <!-- Extra Tabs -->
    <template #extra-tabs>
      <Tab value="endpoints" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
        <Shield class="w-3.5 h-3.5" />
        <span>Endpoints ({{ props.service?.endpointsList?.length || 0 }})</span>
      </Tab>
      <Tab value="ports" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
        <Activity class="w-3.5 h-3.5" />
        <span>Ports ({{ props.service?.portsList?.length || 0 }})</span>
      </Tab>
    </template>

    <!-- Overview Panel -->
    <template #overview>
      <ServiceOverviewTab v-if="props.service" :service="props.service" />
    </template>

    <!-- Extra Panels -->
    <template #extra-panels>
      <TabPanel value="endpoints">
        <ServiceEndpointsTab v-if="props.service" :service="props.service" />
      </TabPanel>

      <TabPanel value="ports">
        <ServicePortsTab v-if="props.service" :service="props.service" />
      </TabPanel>
    </template>

    <!-- YAML Panel -->
    <template #yaml>
      <ResourceYamlTab
        :displayed-yaml="displayedYaml"
        :is-yaml-loading="isYamlLoading"
        :copied="copied"
        @copy-yaml="copyYaml"
      />
    </template>
  </BaseResourceDrawer>
</template>
