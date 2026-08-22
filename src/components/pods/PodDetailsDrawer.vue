<script setup lang="ts">
import PodContainersTab from '@/components/pods/PodContainersTab.vue'
import PodOverviewTab from '@/components/pods/PodOverviewTab.vue'
import WorkloadEventsTab from '@/components/workloads/WorkloadEventsTab.vue'
import WorkloadYamlTab from '@/components/workloads/WorkloadYamlTab.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import type { PodInfo } from '@/types/kubernetes'
import { Activity, FileCode, Server, Shield, Terminal } from '@lucide/vue'
import { storeToRefs } from 'pinia'
import Button from 'primevue/button'
import Drawer from 'primevue/drawer'
import Tab from 'primevue/tab'
import TabList from 'primevue/tablist'
import TabPanel from 'primevue/tabpanel'
import TabPanels from 'primevue/tabpanels'
import Tabs from 'primevue/tabs'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import * as yaml from 'yaml'

const props = defineProps<{
  visible: boolean
  pod: PodInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const router = useRouter()
const k8sStore = useKubernetesStore()
const { events: clusterEvents } = storeToRefs(k8sStore)

const activeTab = ref('overview')

const podStatus = computed(() => {
  return props.pod?.status || 'Unknown'
})

const getStatusBadgeClass = (status: string) => {
  const s = status.toLowerCase()
  if (s === 'running' || s === 'succeeded' || s === 'completed') {
    return 'bg-emerald-500'
  }
  if (s === 'pending' || s === 'containercreating') {
    return 'bg-amber-500'
  }
  return 'bg-rose-500'
}

const podEvents = computed(() => {
  if (!props.pod) return []
  const ns = props.pod.namespace
  const name = props.pod.name
  return clusterEvents.value.filter((ev) => {
    return ev.namespace === ns && ev.objectName === name
  })
})

const rawYamlData = ref<string | null>(null)
const isYamlLoading = ref<boolean>(false)
const copied = ref<boolean>(false)

const fetchRawYaml = async () => {
  if (!props.pod || !props.visible) return
  isYamlLoading.value = true
  try {
    await kubernetesService.getResourceRaw({
      namespace: props.pod.namespace,
      kind: 'Pod',
      name: props.pod.name
    })
  } catch (e) {
    console.error('Failed to fetch raw Pod YAML:', e)
    isYamlLoading.value = false
  }
}

const handleRawData = (payload: { name?: string; namespace?: string; data?: unknown }) => {
  if (!props.visible || !props.pod) return
  const matchesName = payload?.name === props.pod.name
  const matchesNs = !payload?.namespace || payload?.namespace === props.pod.namespace
  if (matchesName && matchesNs) {
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
  () => [props.visible, props.pod?.name, props.pod?.namespace],
  ([newVisible]) => {
    if (newVisible && props.pod) {
      rawYamlData.value = null
      if (activeTab.value === 'yaml') {
        fetchRawYaml()
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
    console.error('Failed to copy Pod YAML:', e)
  }
}

const viewPodLogs = (containerName?: string) => {
  if (!props.pod) return
  router.push({
    name: 'logs',
    query: {
      namespace: props.pod.namespace,
      workload: props.pod.name,
      kind: 'Pod',
      pod: props.pod.name,
      container: containerName || 'All'
    }
  })
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
      <div v-if="pod" class="flex items-center justify-between w-full pr-4">
        <div class="flex items-center gap-3 min-w-0">
          <span
            class="w-3 h-3 rounded-full shrink-0 animate-pulse"
            :class="getStatusBadgeClass(podStatus)"
          ></span>
          <div class="min-w-0">
            <div class="flex items-center gap-2">
              <h3
                class="text-base font-bold text-primary font-mono truncate max-w-70"
                :title="pod.name"
              >
                {{ pod.name }}
              </h3>
              <Tag rounded severity="info" class="font-mono" value="Pod" />
            </div>
            <div class="flex items-center gap-2 text-xs text-muted-color font-mono mt-0.5">
              <span>ns: {{ pod.namespace }}</span>
            </div>
          </div>
        </div>

        <Button
          severity="secondary"
          size="small"
          variant="outlined"
          class="text-xs flex items-center gap-1.5"
          title="View Pod Logs"
          @click="viewPodLogs()"
        >
          <Terminal class="w-3.5 h-3.5" />
          <span>Logs</span>
        </Button>
      </div>
    </template>

    <div v-if="pod" class="flex flex-col h-full">
      <Tabs v-model:value="activeTab" class="flex flex-col flex-1 min-h-0">
        <TabList class="bg-transparent! border-b! border-(--border)! px-2">
          <Tab value="overview" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
            <Server class="w-3.5 h-3.5" />
            <span>Overview</span>
          </Tab>
          <Tab value="containers" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
            <Shield class="w-3.5 h-3.5" />
            <span>Containers ({{ pod.containers?.length || 0 }})</span>
          </Tab>
          <Tab value="events" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
            <Activity class="w-3.5 h-3.5" />
            <span>Events ({{ podEvents.length }})</span>
          </Tab>
          <Tab value="yaml" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
            <FileCode class="w-3.5 h-3.5" />
            <span>YAML</span>
          </Tab>
        </TabList>

        <TabPanels class="flex-1 overflow-y-auto p-6! bg-transparent!">
          <TabPanel value="overview">
            <PodOverviewTab :pod="pod" />
          </TabPanel>

          <TabPanel value="containers">
            <PodContainersTab :containers="pod.containers || []" @view-logs="viewPodLogs" />
          </TabPanel>

          <TabPanel value="events">
            <WorkloadEventsTab :events="podEvents" />
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
