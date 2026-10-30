<script setup lang="ts">
import BaseResourceDrawer from '@/components/shared/BaseResourceDrawer.vue'
import KeyValueBadgeList from '@/components/shared/KeyValueBadgeList.vue'
import ReactiveAge from '@/components/shared/ReactiveAge.vue'
import ResourceYamlTab from '@/components/shared/ResourceYamlTab.vue'
import WorkloadEventsTab from '@/components/workloads/WorkloadEventsTab.vue'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import type { PersistentVolumeInfo } from '@/types/kubernetes'
import { sortEventsDesc } from '@/utils/events'
import { getVolumeStatusBadgeClass } from '@/utils/severity'
import { Activity, Pencil } from '@lucide/vue'
import { storeToRefs } from 'pinia'
import Button from 'primevue/button'
import Tab from 'primevue/tab'
import TabPanel from 'primevue/tabpanel'
import Tag from 'primevue/tag'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import * as yaml from 'yaml'

const props = defineProps<{
  visible: boolean
  pv: PersistentVolumeInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const router = useRouter()
const k8sStore = useKubernetesStore()
const { events: clusterEvents } = storeToRefs(k8sStore)

const activeTab = ref('overview')
const rawYamlData = ref<string | null>(null)
const rawManifest = ref<Record<string, unknown> | null>(null)
const isYamlLoading = ref<boolean>(false)
const yamlError = ref<string | null>(null)
const copied = ref<boolean>(false)
const currentRequestId = ref<number>(0)

const pvEvents = computed(() => {
  if (!props.pv) return []
  const name = props.pv.name
  const filtered = clusterEvents.value.filter((ev) => {
    return ev.objectName === name || ev.message?.includes(name)
  })
  return sortEventsDesc(filtered)
})

const handleEdit = () => {
  if (!props.pv) return
  router.push({
    name: 'edit-workload',
    params: {
      kind: KUBERNETES_RESOURCE_KIND.PersistentVolume,
      namespace: 'default',
      name: props.pv.name
    }
  })
}

const fetchRawYaml = async () => {
  if (!props.pv || !props.visible) return
  const reqId = ++currentRequestId.value
  isYamlLoading.value = true
  yamlError.value = null
  try {
    await kubernetesService.getResourceRaw({
      namespace: '',
      kind: KUBERNETES_RESOURCE_KIND.PersistentVolume,
      name: props.pv.name,
      requestId: String(reqId)
    })
  } catch (e) {
    if (reqId === currentRequestId.value) {
      isYamlLoading.value = false
      yamlError.value = String(e)
    }
  }
}

const handleRawData = (payload: {
  kind?: string
  name?: string
  namespace?: string
  data?: unknown
  requestId?: string
}) => {
  if (!props.visible || !props.pv) return
  if (payload?.requestId && payload.requestId !== String(currentRequestId.value)) return

  const matchesKind =
    !payload?.kind ||
    payload.kind.toLowerCase() === KUBERNETES_RESOURCE_KIND.PersistentVolume.toLowerCase()
  const matchesName = payload?.name === props.pv.name

  if (matchesKind && matchesName) {
    if (payload.data) {
      if (typeof payload.data === 'string') {
        try {
          rawManifest.value = JSON.parse(payload.data) as Record<string, unknown>
          rawYamlData.value = payload.data
        } catch {
          rawYamlData.value = payload.data
        }
      } else {
        rawManifest.value = payload.data as Record<string, unknown>
        rawYamlData.value = JSON.stringify(payload.data)
      }
    }
    isYamlLoading.value = false
  }
}

const handleBatchUpdate = (payload: { kind?: string }) => {
  if (
    props.visible &&
    props.pv &&
    (!payload?.kind || payload.kind === KUBERNETES_RESOURCE_KIND.PersistentVolume)
  ) {
    void fetchRawYaml()
  }
}

onMounted(() => {
  events.on(OrbitEvents.ResourceRawData, handleRawData)
  events.on(OrbitEvents.ResourceBatchUpdated, handleBatchUpdate)
})

onUnmounted(() => {
  events.off(OrbitEvents.ResourceRawData, handleRawData)
  events.off(OrbitEvents.ResourceBatchUpdated, handleBatchUpdate)
})

watch(
  () => [props.visible, props.pv?.name] as const,
  ([newVisible, newName], prev) => {
    const oldVisible = prev?.[0]
    const oldName = prev?.[1]
    if (newVisible && props.pv) {
      if (newName !== oldName || !oldVisible) {
        rawYamlData.value = null
        rawManifest.value = null
        void fetchRawYaml()
      }
    }
  },
  { immediate: true }
)

const displayedYaml = computed(() => {
  if (!rawYamlData.value) {
    if (!props.pv) return ''
    return yaml.stringify({
      apiVersion: 'v1',
      kind: 'PersistentVolume',
      metadata: {
        name: props.pv.name
      },
      spec: {
        capacity: {
          storage: props.pv.capacity
        },
        accessModes: [props.pv.accessMode],
        persistentVolumeReclaimPolicy: props.pv.reclaimPolicy,
        storageClassName: props.pv.storageClass,
        volumeMode: props.pv.volumeMode
      },
      status: {
        phase: props.pv.status
      }
    })
  }
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
    console.error('Failed to copy YAML:', e)
  }
}

const volumeSource = computed(() => {
  const spec = (rawManifest.value?.spec as Record<string, unknown>) || {}
  if (spec.hostPath) {
    const hp = spec.hostPath as { path?: string; type?: string }
    return { type: 'hostPath', details: `Path: ${hp.path || '-'} (${hp.type || 'Unspecified'})` }
  }
  if (spec.csi) {
    const csi = spec.csi as { driver?: string; volumeHandle?: string; fsType?: string }
    return {
      type: 'CSI',
      details: `Driver: ${csi.driver || '-'}, Handle: ${csi.volumeHandle || '-'}, FSType: ${csi.fsType || '-'}`
    }
  }
  if (spec.nfs) {
    const nfs = spec.nfs as { server?: string; path?: string }
    return { type: 'NFS', details: `Server: ${nfs.server || '-'}, Path: ${nfs.path || '-'}` }
  }
  if (spec.local) {
    const local = spec.local as { path?: string }
    return { type: 'Local', details: `Path: ${local.path || '-'}` }
  }
  if (spec.iscsi) {
    const iscsi = spec.iscsi as { targetPortal?: string; iqn?: string }
    return {
      type: 'iSCSI',
      details: `Portal: ${iscsi.targetPortal || '-'}, IQN: ${iscsi.iqn || '-'}`
    }
  }
  return { type: 'Standard', details: 'Configured via volume source plugin' }
})

const claimRefText = computed(() => {
  const spec = (rawManifest.value?.spec as Record<string, unknown>) || {}
  const claimRef = spec.claimRef as { namespace?: string; name?: string } | undefined
  if (claimRef?.name) {
    return claimRef.namespace ? `${claimRef.namespace}/${claimRef.name}` : claimRef.name
  }
  return '-'
})

const mountOptions = computed<string[]>(() => {
  const spec = (rawManifest.value?.spec as Record<string, unknown>) || {}
  return (spec.mountOptions as string[]) || []
})

const pvLabels = computed(() => {
  const meta = (rawManifest.value?.metadata as Record<string, unknown>) || {}
  return (meta.labels as Record<string, string>) || {}
})

const pvAnnotations = computed(() => {
  const meta = (rawManifest.value?.metadata as Record<string, unknown>) || {}
  return (meta.annotations as Record<string, string>) || {}
})

const pvUid = computed(() => {
  const meta = (rawManifest.value?.metadata as Record<string, unknown>) || {}
  return (meta.uid as string) || '-'
})
</script>

<template>
  <BaseResourceDrawer
    data-component="persistent-volume-details-drawer"
    v-model:active-tab="activeTab"
    :visible="props.visible"
    :has-resource="!!props.pv"
    :title="props.pv?.name ?? ''"
    :kind="KUBERNETES_RESOURCE_KIND.PersistentVolume"
    kind-severity="info"
    :status-badge-class="getVolumeStatusBadgeClass(props.pv?.status)"
    :age="props.pv?.age"
    @update:visible="emit('update:visible', $event)"
  >
    <template #tags>
      <Tag
        rounded
        severity="info"
        class="font-mono"
        :value="KUBERNETES_RESOURCE_KIND.PersistentVolume"
      />
      <Tag
        v-if="props.pv?.storageClass"
        rounded
        severity="secondary"
        class="font-mono"
        :value="props.pv.storageClass"
      />
    </template>

    <template #actions>
      <Button
        v-if="props.pv"
        size="small"
        severity="secondary"
        variant="outlined"
        class="flex items-center gap-1.5"
        @click="handleEdit"
      >
        <Pencil class="w-3.5 h-3.5" />
        <span>Edit</span>
      </Button>
    </template>

    <template #extra-tabs>
      <Tab value="events" class="text-xs flex items-center gap-1.5 py-2.5 px-3">
        <Activity class="w-3.5 h-3.5" />
        <span>Events ({{ pvEvents.length }})</span>
      </Tab>
    </template>

    <template #overview>
      <div v-if="props.pv" class="space-y-6">
        <!-- General Info -->
        <div>
          <h3 class="text-xs font-bold text-muted-color uppercase tracking-wider mb-3">General</h3>
          <div
            class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-card)] p-4 flex flex-col gap-3 text-xs font-ui"
          >
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Name</span>
              <span class="col-span-2 font-mono text-primary font-semibold">{{
                props.pv.name
              }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Status</span>
              <span class="col-span-2 font-mono text-primary">{{ props.pv.status }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Capacity</span>
              <span class="col-span-2 font-mono text-(--accent) font-semibold">{{
                props.pv.capacity
              }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Volume Mode</span>
              <span class="col-span-2 font-mono text-primary">{{
                props.pv.volumeMode || 'Filesystem'
              }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Access Modes</span>
              <span class="col-span-2 font-mono text-primary">{{ props.pv.accessMode }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Reclaim Policy</span>
              <span class="col-span-2 font-mono text-primary">{{ props.pv.reclaimPolicy }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Storage Class</span>
              <span class="col-span-2 font-mono text-primary">{{
                props.pv.storageClass || '-'
              }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Claim</span>
              <span class="col-span-2 font-mono text-primary">{{ claimRefText }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Age</span>
              <span class="col-span-2 text-primary"><ReactiveAge :age="props.pv.age" /></span>
            </div>
            <div class="grid grid-cols-3 items-start gap-4">
              <span class="text-muted-color font-semibold shrink-0">UID</span>
              <span class="col-span-2 font-mono text-[10px] text-muted-color truncate">{{
                pvUid
              }}</span>
            </div>
          </div>
        </div>

        <!-- Volume Source -->
        <div>
          <h3 class="text-xs font-bold text-muted-color uppercase tracking-wider mb-3">
            Volume Source
          </h3>
          <div
            class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-card)] p-4 flex flex-col gap-2 text-xs font-ui"
          >
            <div class="flex items-center gap-2">
              <span class="text-muted-color font-semibold">Type:</span>
              <Tag
                rounded
                severity="info"
                class="font-mono text-[10px]"
                :value="volumeSource.type"
              />
            </div>
            <div class="text-[11px] font-mono text-muted-color break-all">
              {{ volumeSource.details }}
            </div>
          </div>
        </div>

        <!-- Mount Options -->
        <div v-if="mountOptions.length > 0">
          <h3 class="text-xs font-bold text-muted-color uppercase tracking-wider mb-3">
            Mount Options
          </h3>
          <div class="flex flex-wrap gap-2">
            <Tag
              v-for="opt in mountOptions"
              :key="opt"
              severity="secondary"
              class="font-mono"
              :value="opt"
            />
          </div>
        </div>

        <!-- Labels & Annotations -->
        <KeyValueBadgeList :items="pvLabels" title="Labels" variant="tag" />
        <KeyValueBadgeList :items="pvAnnotations" title="Annotations" variant="list" />
      </div>
    </template>

    <template #extra-panels>
      <TabPanel value="events">
        <WorkloadEventsTab :events="pvEvents" />
      </TabPanel>
    </template>

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
