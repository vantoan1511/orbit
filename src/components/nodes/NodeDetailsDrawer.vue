<script setup lang="ts">
import NodeOverviewTab from '@/components/nodes/NodeOverviewTab.vue'
import WorkloadEventsTab from '@/components/workloads/WorkloadEventsTab.vue'
import ResourceYamlTab from '@/components/shared/ResourceYamlTab.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import type { NodeInfo } from '@/types/kubernetes'
import { Activity } from '@lucide/vue'
import { storeToRefs } from 'pinia'
import BaseResourceDrawer from '@/components/shared/BaseResourceDrawer.vue'
import Tab from 'primevue/tab'
import TabPanel from 'primevue/tabpanel'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import * as yaml from 'yaml'

const props = defineProps<{
  visible: boolean
  node: NodeInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const k8sStore = useKubernetesStore()
const { events: clusterEvents } = storeToRefs(k8sStore)

const activeTab = ref('overview')

const nodeStatus = computed(() => {
  return props.node?.status || 'Unknown'
})

const getStatusBadgeClass = (status: string) => {
  const s = status.toLowerCase()
  if (s === 'ready') {
    return 'bg-emerald-500'
  }
  return 'bg-rose-500'
}

const nodeEvents = computed(() => {
  if (!props.node) return []
  const name = props.node.name
  return clusterEvents.value.filter((ev) => {
    return ev.objectName === name || (ev.source && ev.source.includes(name))
  })
})

const rawYamlData = ref<string | null>(null)
const isYamlLoading = ref<boolean>(false)
const copied = ref<boolean>(false)

const fetchRawYaml = async () => {
  if (!props.node || !props.visible) return
  isYamlLoading.value = true
  try {
    await kubernetesService.getResourceRaw({
      namespace: '',
      kind: 'Node',
      name: props.node.name
    })
  } catch (e) {
    console.error('Failed to fetch raw Node YAML:', e)
    isYamlLoading.value = false
  }
}

const handleRawData = (payload: {
  kind?: string
  name?: string
  namespace?: string
  data?: unknown
}) => {
  if (!props.visible || !props.node) return
  const matchesKind = !payload?.kind || payload.kind.toLowerCase() === 'node'
  const matchesName = payload?.name === props.node.name
  if (matchesKind && matchesName) {
    if (payload.data) {
      rawYamlData.value =
        typeof payload.data === 'string' ? payload.data : JSON.stringify(payload.data)
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
  () => [props.visible, props.node?.name],
  ([newVisible, newName], [oldVisible, oldName]) => {
    if (newVisible && props.node) {
      if (newName !== oldName || !oldVisible) {
        rawYamlData.value = null
        if (activeTab.value === 'yaml') {
          fetchRawYaml()
        }
      }
    }
  }
)

watch(activeTab, (newTab) => {
  if (newTab === 'yaml' && !rawYamlData.value && !isYamlLoading.value) {
    fetchRawYaml()
  }
})

const displayedYaml = computed(() => {
  if (!rawYamlData.value) return ''
  try {
    const parsed = JSON.parse(rawYamlData.value)
    return yaml.stringify(parsed)
  } catch {
    return rawYamlData.value
  }
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
    console.error('Failed to copy Node YAML:', e)
  }
}
</script>

<template>
  <BaseResourceDrawer
    v-model:active-tab="activeTab"
    :visible="visible"
    :has-resource="!!node"
    :title="node?.name ?? ''"
    kind="Node"
    kind-severity="info"
    :status-badge-class="getStatusBadgeClass(nodeStatus)"
    @update:visible="(val) => emit('update:visible', val)"
  >
    <template #metadata>
      <div v-if="node" class="flex items-center gap-2 text-xs text-muted-color font-mono mt-0.5">
        <span>role: {{ node.role }}</span>
        <span>•</span>
        <span>version: {{ node.version }}</span>
      </div>
    </template>

    <!-- Extra Tabs -->
    <template #extra-tabs>
      <Tab value="events" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
        <Activity class="w-3.5 h-3.5" />
        <span>Events ({{ nodeEvents.length }})</span>
      </Tab>
    </template>

    <!-- Overview Panel -->
    <template #overview>
      <NodeOverviewTab v-if="node" :node="node" />
    </template>

    <!-- Extra Panels -->
    <template #extra-panels>
      <TabPanel value="events">
        <WorkloadEventsTab :events="nodeEvents" />
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
