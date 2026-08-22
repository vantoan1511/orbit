<script setup lang="ts">
import NodeOverviewTab from '@/components/nodes/NodeOverviewTab.vue'
import WorkloadEventsTab from '@/components/workloads/WorkloadEventsTab.vue'
import WorkloadYamlTab from '@/components/workloads/WorkloadYamlTab.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import type { NodeInfo } from '@/types/kubernetes'
import { Activity, FileCode, Server } from '@lucide/vue'
import { storeToRefs } from 'pinia'
import Drawer from 'primevue/drawer'
import Tab from 'primevue/tab'
import TabList from 'primevue/tablist'
import TabPanel from 'primevue/tabpanel'
import TabPanels from 'primevue/tabpanels'
import Tabs from 'primevue/tabs'
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
  <Drawer
    :visible="visible"
    position="right"
    class="w-160! bg-(--bg-card)! border-l! border-(--border)!"
    :dismissable="true"
    @update:visible="(val) => emit('update:visible', val)"
  >
    <template #header>
      <div v-if="node" class="flex items-center justify-between w-full pr-4">
        <div class="flex items-center gap-3 min-w-0">
          <span
            class="w-3 h-3 rounded-full shrink-0"
            :class="getStatusBadgeClass(nodeStatus)"
          ></span>
          <div class="min-w-0">
            <div class="flex items-center gap-2">
              <h3
                class="text-base font-bold text-primary font-mono truncate max-w-70"
                :title="node.name"
              >
                {{ node.name }}
              </h3>
              <Tag rounded severity="info" class="font-mono" value="Node" />
            </div>
            <div class="flex items-center gap-2 text-xs text-muted-color font-mono mt-0.5">
              <span>role: {{ node.role }}</span>
              <span>•</span>
              <span>version: {{ node.version }}</span>
            </div>
          </div>
        </div>
      </div>
    </template>

    <div v-if="node" class="flex flex-col h-full">
      <Tabs v-model:value="activeTab" class="flex flex-col flex-1 min-h-0">
        <TabList class="bg-transparent! border-b! border-(--border)! px-2">
          <Tab value="overview" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
            <Server class="w-3.5 h-3.5" />
            <span>Overview</span>
          </Tab>
          <Tab value="events" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
            <Activity class="w-3.5 h-3.5" />
            <span>Events ({{ nodeEvents.length }})</span>
          </Tab>
          <Tab value="yaml" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
            <FileCode class="w-3.5 h-3.5" />
            <span>YAML</span>
          </Tab>
        </TabList>

        <TabPanels class="flex-1 overflow-y-auto p-6! bg-transparent!">
          <TabPanel value="overview">
            <NodeOverviewTab :node="node" />
          </TabPanel>

          <TabPanel value="events">
            <WorkloadEventsTab :events="nodeEvents" />
          </TabPanel>

          <TabPanel value="yaml">
            <WorkloadYamlTab
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
