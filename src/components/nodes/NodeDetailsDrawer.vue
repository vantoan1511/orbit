<script setup lang="ts">
import NodeOverviewTab from '@/components/nodes/NodeOverviewTab.vue'
import WorkloadEventsTab from '@/components/workloads/WorkloadEventsTab.vue'
import ResourceYamlTab from '@/components/shared/ResourceYamlTab.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import type { NodeInfo } from '@/types/kubernetes'
import { sortEventsDesc } from '@/utils/events'
import { getNodeStatusBadgeClass } from '@/utils/severity'
import { Activity, Ban, CheckCircle, FileCode, LogOut } from '@lucide/vue'
import { storeToRefs } from 'pinia'
import BaseResourceDrawer from '@/components/shared/BaseResourceDrawer.vue'
import Button from 'primevue/button'
import Tab from 'primevue/tab'
import TabPanel from 'primevue/tabpanel'
import Tag from 'primevue/tag'
import { useConfirm } from 'primevue/useconfirm'
import { useToast } from 'primevue/usetoast'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
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
const confirm = useConfirm()
const toast = useToast()
const router = useRouter()

const liveNode = computed<NodeInfo | null>(() => {
  if (!props.node) return null
  return k8sStore.nodes.find((n) => n.name === props.node?.name) ?? props.node
})

const activeTab = ref('overview')

const nodeStatus = computed(() => liveNode.value?.status || 'Unknown')

const nodeEvents = computed(() => {
  if (!liveNode.value) return []
  const name = liveNode.value.name
  const filtered = clusterEvents.value.filter((ev) => {
    return ev.objectName === name || ev.message?.includes(name)
  })
  return sortEventsDesc(filtered)
})

const rawYamlData = ref<string | null>(null)
const isYamlLoading = ref<boolean>(false)
const copied = ref<boolean>(false)

const fetchRawYaml = async () => {
  if (!liveNode.value || !props.visible) return
  isYamlLoading.value = true
  try {
    await kubernetesService.getResourceRaw({
      namespace: '',
      kind: KUBERNETES_RESOURCE_KIND.Node,
      name: liveNode.value.name
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
  if (!props.visible || !liveNode.value) return
  const matchesKind =
    !payload?.kind || payload.kind.toLowerCase() === KUBERNETES_RESOURCE_KIND.Node.toLowerCase()
  const matchesName = payload?.name === liveNode.value.name
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
  () => [props.visible, liveNode.value?.name],
  ([newVisible, newName], [oldVisible, oldName]) => {
    if (newVisible && liveNode.value) {
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

const handleCordon = () => {
  if (!liveNode.value) return
  const nodeName = liveNode.value.name
  confirm.require({
    message: `Are you sure you want to cordon node "${nodeName}"? New pods will not be scheduled on this node.`,
    header: 'Confirm Cordon Node',
    icon: 'pi pi-exclamation-triangle',
    rejectProps: {
      label: 'Cancel',
      severity: 'secondary',
      variant: 'text',
      size: 'small'
    },
    acceptProps: {
      label: 'Cordon',
      severity: 'primary',
      size: 'small'
    },
    accept: async () => {
      try {
        await kubernetesService.cordonNode({ name: nodeName })
        toast.add({
          severity: 'warn',
          summary: 'Node Cordoned',
          detail: `Node ${nodeName} marked as unschedulable`,
          life: 3000
        })
      } catch (e) {
        toast.add({
          severity: 'error',
          summary: 'Error',
          detail: e instanceof Error ? e.message : 'Failed to cordon node',
          life: 5000
        })
      }
    }
  })
}

const handleUncordon = async () => {
  if (!liveNode.value) return
  const nodeName = liveNode.value.name
  try {
    await kubernetesService.uncordonNode({ name: nodeName })
    toast.add({
      severity: 'success',
      summary: 'Node Uncordoned',
      detail: `Node ${nodeName} marked as schedulable`,
      life: 3000
    })
  } catch (e) {
    toast.add({
      severity: 'error',
      summary: 'Error',
      detail: e instanceof Error ? e.message : 'Failed to uncordon node',
      life: 5000
    })
  }
}

const handleDrain = () => {
  if (!liveNode.value) return
  const nodeName = liveNode.value.name
  confirm.require({
    message: `Are you sure you want to drain node "${nodeName}"? The node will be cordoned and all eligible pods will be evicted.`,
    header: 'Confirm Drain Node',
    icon: 'pi pi-exclamation-triangle',
    rejectProps: {
      label: 'Cancel',
      severity: 'secondary',
      variant: 'text',
      size: 'small'
    },
    acceptProps: {
      label: 'Drain',
      severity: 'danger',
      size: 'small'
    },
    accept: async () => {
      try {
        await kubernetesService.drainNode({ name: nodeName })
        toast.add({
          severity: 'success',
          summary: 'Drain Initiated',
          detail: `Node ${nodeName} drain initiated`,
          life: 3000
        })
      } catch (e) {
        toast.add({
          severity: 'error',
          summary: 'Error',
          detail: e instanceof Error ? e.message : 'Failed to drain node',
          life: 5000
        })
      }
    }
  })
}

const handleEdit = () => {
  if (!liveNode.value) return
  router.push({
    name: 'edit-workload',
    params: {
      kind: KUBERNETES_RESOURCE_KIND.Node,
      namespace: 'default',
      name: liveNode.value.name
    }
  })
}
</script>

<template>
  <BaseResourceDrawer
    data-component="node-details-drawer"
    v-model:active-tab="activeTab"
    :visible="visible"
    :has-resource="!!liveNode"
    :title="liveNode?.name ?? ''"
    :kind="KUBERNETES_RESOURCE_KIND.Node"
    kind-severity="info"
    :status-badge-class="getNodeStatusBadgeClass(nodeStatus)"
    @update:visible="(val) => emit('update:visible', val)"
  >
    <template #tags>
      <Tag severity="info" :value="KUBERNETES_RESOURCE_KIND.Node" />
      <Tag
        v-if="liveNode?.isCordoned"
        severity="warn"
        value="Cordoned"
        class="font-mono text-[10px]"
      />
    </template>

    <template #metadata>
      <div
        v-if="liveNode"
        class="flex items-center gap-2 text-xs text-muted-color font-mono mt-0.5"
      >
        <span>role: {{ liveNode.role }}</span>
        <span>•</span>
        <span>version: {{ liveNode.version }}</span>
      </div>
    </template>

    <template #actions>
      <Button
        v-if="liveNode?.isCordoned"
        severity="secondary"
        size="small"
        variant="outlined"
        class="text-xs flex items-center gap-1.5"
        v-tooltip.top="'Mark node as schedulable'"
        @click="handleUncordon"
      >
        <CheckCircle class="w-3.5 h-3.5" />
        <span>Uncordon</span>
      </Button>

      <Button
        v-else
        severity="secondary"
        size="small"
        variant="outlined"
        class="text-xs flex items-center gap-1.5"
        v-tooltip.top="'Mark node as unschedulable'"
        @click="handleCordon"
      >
        <Ban class="w-3.5 h-3.5" />
        <span>Cordon</span>
      </Button>

      <Button
        severity="danger"
        size="small"
        variant="outlined"
        class="text-xs flex items-center gap-1.5"
        v-tooltip.top="'Cordon node and evict pods'"
        @click="handleDrain"
      >
        <LogOut class="w-3.5 h-3.5" />
        <span>Drain</span>
      </Button>

      <Button
        severity="secondary"
        size="small"
        variant="outlined"
        class="text-xs flex items-center gap-1.5"
        v-tooltip.top="'Edit YAML'"
        @click="handleEdit"
      >
        <FileCode class="w-3.5 h-3.5" />
        <span>Edit</span>
      </Button>
    </template>

    <!-- Extra Tabs -->
    <template #extra-tabs>
      <Tab value="events" class="text-xs flex items-center gap-1.5 py-2.5 px-3">
        <Activity class="w-3.5 h-3.5" />
        <span>Events ({{ nodeEvents.length }})</span>
      </Tab>
    </template>

    <!-- Overview Panel -->
    <template #overview>
      <NodeOverviewTab v-if="liveNode" :node="liveNode" />
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
