<script setup lang="ts">
import PodContainersTab from '@/components/pods/PodContainersTab.vue'
import PodOverviewTab from '@/components/pods/PodOverviewTab.vue'
import WorkloadEventsTab from '@/components/workloads/WorkloadEventsTab.vue'
import ResourceYamlTab from '@/components/shared/ResourceYamlTab.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import type { PodInfo } from '@/types/kubernetes'
import { getPodStatusBadgeClass } from '@/utils/severity'
import { Activity, Shield, Terminal } from '@lucide/vue'
import { storeToRefs } from 'pinia'
import BaseResourceDrawer from '@/components/shared/BaseResourceDrawer.vue'
import Button from 'primevue/button'
import Tab from 'primevue/tab'
import TabPanel from 'primevue/tabpanel'
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

const podStatus = computed(() => props.pod?.status || 'Unknown')

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
      kind: KUBERNETES_RESOURCE_KIND.Pod,
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
      kind: KUBERNETES_RESOURCE_KIND.Pod,
      pod: props.pod.name,
      container: containerName || 'All'
    }
  })
}
</script>

<template>
  <BaseResourceDrawer
    v-model:active-tab="activeTab"
    :visible="visible"
    :has-resource="!!pod"
    :title="pod?.name ?? ''"
    :kind="KUBERNETES_RESOURCE_KIND.Pod"
    kind-severity="info"
    :status-badge-class="getPodStatusBadgeClass(podStatus)"
    :namespace="pod?.namespace"
    @update:visible="(val) => emit('update:visible', val)"
  >
    <template #actions>
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
    </template>

    <!-- Extra Tabs -->
    <template #extra-tabs>
      <Tab value="containers" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
        <Shield class="w-3.5 h-3.5" />
        <span>Containers ({{ pod?.containers?.length || 0 }})</span>
      </Tab>
      <Tab value="events" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
        <Activity class="w-3.5 h-3.5" />
        <span>Events ({{ podEvents.length }})</span>
      </Tab>
    </template>

    <!-- Overview Panel -->
    <template #overview>
      <PodOverviewTab v-if="pod" :pod="pod" />
    </template>

    <!-- Extra Panels -->
    <template #extra-panels>
      <TabPanel value="containers">
        <PodContainersTab :containers="pod?.containers || []" @view-logs="viewPodLogs" />
      </TabPanel>

      <TabPanel value="events">
        <WorkloadEventsTab :events="podEvents" />
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
