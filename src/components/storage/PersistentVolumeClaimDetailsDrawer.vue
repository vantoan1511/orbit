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
import type { PersistentVolumeClaimInfo } from '@/types/kubernetes'
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
  pvc: PersistentVolumeClaimInfo | null
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

const pvcEvents = computed(() => {
  if (!props.pvc) return []
  const name = props.pvc.name
  const ns = props.pvc.namespace
  const filtered = clusterEvents.value.filter((ev) => {
    const matchesNs = !ev.namespace || ev.namespace === ns
    return matchesNs && (ev.objectName === name || ev.message?.includes(name))
  })
  return sortEventsDesc(filtered)
})

const handleEdit = () => {
  if (!props.pvc) return
  router.push({
    name: 'edit-workload',
    params: {
      kind: KUBERNETES_RESOURCE_KIND.PersistentVolumeClaim,
      namespace: props.pvc.namespace,
      name: props.pvc.name
    }
  })
}

const fetchRawYaml = async () => {
  if (!props.pvc || !props.visible) return
  const reqId = ++currentRequestId.value
  isYamlLoading.value = true
  yamlError.value = null
  try {
    await kubernetesService.getResourceRaw({
      namespace: props.pvc.namespace,
      kind: KUBERNETES_RESOURCE_KIND.PersistentVolumeClaim,
      name: props.pvc.name,
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
  if (!props.visible || !props.pvc) return
  if (payload?.requestId && payload.requestId !== String(currentRequestId.value)) return

  const matchesKind =
    !payload?.kind ||
    payload.kind.toLowerCase() === KUBERNETES_RESOURCE_KIND.PersistentVolumeClaim.toLowerCase()
  const matchesName = payload?.name === props.pvc.name
  const matchesNs = !payload?.namespace || payload.namespace === props.pvc.namespace

  if (matchesKind && matchesName && matchesNs) {
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
    props.pvc &&
    (!payload?.kind || payload.kind === KUBERNETES_RESOURCE_KIND.PersistentVolumeClaim)
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
  () => [props.visible, props.pvc?.name, props.pvc?.namespace] as const,
  ([newVisible, newName], prev) => {
    const oldVisible = prev?.[0]
    const oldName = prev?.[1]
    if (newVisible && props.pvc) {
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
    if (!props.pvc) return ''
    return yaml.stringify({
      apiVersion: 'v1',
      kind: 'PersistentVolumeClaim',
      metadata: {
        name: props.pvc.name,
        namespace: props.pvc.namespace
      },
      spec: {
        accessModes: [props.pvc.accessMode],
        resources: {
          requests: {
            storage: props.pvc.capacity
          }
        },
        storageClassName: props.pvc.storageClass,
        volumeName: props.pvc.volume
      },
      status: {
        phase: props.pvc.status
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

const pvcLabels = computed(() => {
  const meta = (rawManifest.value?.metadata as Record<string, unknown>) || {}
  return (meta.labels as Record<string, string>) || {}
})

const pvcAnnotations = computed(() => {
  const meta = (rawManifest.value?.metadata as Record<string, unknown>) || {}
  return (meta.annotations as Record<string, string>) || {}
})

const pvcUid = computed(() => {
  const meta = (rawManifest.value?.metadata as Record<string, unknown>) || {}
  return (meta.uid as string) || '-'
})

const actualCapacity = computed(() => {
  const status = (rawManifest.value?.status as Record<string, unknown>) || {}
  const cap = (status.capacity as Record<string, string>) || {}
  return cap.storage || props.pvc?.capacity || '-'
})

const volumeMode = computed(() => {
  const spec = (rawManifest.value?.spec as Record<string, unknown>) || {}
  return (spec.volumeMode as string) || 'Filesystem'
})

interface PvcCondition {
  type: string
  status: string
  lastTransitionTime?: string
  reason?: string
  message?: string
}

const conditions = computed<PvcCondition[]>(() => {
  const status = (rawManifest.value?.status as Record<string, unknown>) || {}
  return (status.conditions as PvcCondition[]) || []
})
</script>

<template>
  <BaseResourceDrawer
    data-component="persistent-volume-claim-details-drawer"
    v-model:active-tab="activeTab"
    :visible="props.visible"
    :has-resource="!!props.pvc"
    :title="props.pvc?.name ?? ''"
    :kind="KUBERNETES_RESOURCE_KIND.PersistentVolumeClaim"
    kind-severity="info"
    :status-badge-class="getVolumeStatusBadgeClass(props.pvc?.status)"
    :namespace="props.pvc?.namespace"
    :age="props.pvc?.age"
    @update:visible="emit('update:visible', $event)"
  >
    <template #tags>
      <Tag
        rounded
        severity="info"
        class="font-mono"
        :value="KUBERNETES_RESOURCE_KIND.PersistentVolumeClaim"
      />
      <Tag
        v-if="props.pvc?.storageClass"
        rounded
        severity="secondary"
        class="font-mono"
        :value="props.pvc.storageClass"
      />
    </template>

    <template #actions>
      <Button
        v-if="props.pvc"
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
        <span>Events ({{ pvcEvents.length }})</span>
      </Tab>
    </template>

    <template #overview>
      <div v-if="props.pvc" class="space-y-6">
        <!-- General Info -->
        <div>
          <h3 class="text-xs font-bold text-muted-color uppercase tracking-wider mb-3">General</h3>
          <div
            class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-card)] p-4 flex flex-col gap-3 text-xs font-ui"
          >
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Name</span>
              <span class="col-span-2 font-mono text-primary font-semibold">{{
                props.pvc.name
              }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Namespace</span>
              <span class="col-span-2 font-mono text-primary">{{ props.pvc.namespace }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Status</span>
              <span class="col-span-2 font-mono text-primary">{{ props.pvc.status }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Bound Volume</span>
              <span class="col-span-2 font-mono text-primary">{{ props.pvc.volume || '-' }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Requested Storage</span>
              <span class="col-span-2 font-mono text-(--accent) font-semibold">{{
                props.pvc.capacity
              }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Actual Capacity</span>
              <span class="col-span-2 font-mono text-primary">{{ actualCapacity }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Volume Mode</span>
              <span class="col-span-2 font-mono text-primary">{{ volumeMode }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Access Modes</span>
              <span class="col-span-2 font-mono text-primary">{{ props.pvc.accessMode }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Storage Class</span>
              <span class="col-span-2 font-mono text-primary">{{
                props.pvc.storageClass || '-'
              }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Age</span>
              <span class="col-span-2 text-primary"><ReactiveAge :age="props.pvc.age" /></span>
            </div>
            <div class="grid grid-cols-3 items-start gap-4">
              <span class="text-muted-color font-semibold shrink-0">UID</span>
              <span class="col-span-2 font-mono text-[10px] text-muted-color truncate">{{
                pvcUid
              }}</span>
            </div>
          </div>
        </div>

        <!-- Conditions -->
        <div v-if="conditions.length > 0">
          <h3 class="text-xs font-bold text-muted-color uppercase tracking-wider mb-3">
            Conditions
          </h3>
          <div class="space-y-2">
            <div
              v-for="cond in conditions"
              :key="cond.type"
              class="p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/40 text-xs font-mono flex flex-col gap-1"
            >
              <div class="flex items-center justify-between">
                <span class="font-semibold text-primary">{{ cond.type }}</span>
                <Tag
                  rounded
                  :severity="cond.status === 'True' ? 'success' : 'secondary'"
                  :value="cond.status"
                />
              </div>
              <div v-if="cond.message" class="text-muted-color text-[11px] mt-0.5">
                {{ cond.message }}
              </div>
            </div>
          </div>
        </div>

        <!-- Labels & Annotations -->
        <KeyValueBadgeList :items="pvcLabels" title="Labels" variant="tag" />
        <KeyValueBadgeList :items="pvcAnnotations" title="Annotations" variant="list" />
      </div>
    </template>

    <template #extra-panels>
      <TabPanel value="events">
        <WorkloadEventsTab :events="pvcEvents" />
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
