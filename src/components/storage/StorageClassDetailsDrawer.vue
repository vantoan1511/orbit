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
import type { StorageClassInfo } from '@/types/kubernetes'
import { sortEventsDesc } from '@/utils/events'
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
  storageClass: StorageClassInfo | null
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

const scEvents = computed(() => {
  if (!props.storageClass) return []
  const name = props.storageClass.name
  const filtered = clusterEvents.value.filter((ev) => {
    return ev.objectName === name || ev.message?.includes(name)
  })
  return sortEventsDesc(filtered)
})

const isDefaultClass = computed(() => {
  const meta = (rawManifest.value?.metadata as Record<string, unknown>) || {}
  const annotations = (meta.annotations as Record<string, string>) || {}
  return (
    annotations['storageclass.kubernetes.io/is-default-class'] === 'true' ||
    annotations['storageclass.beta.kubernetes.io/is-default-class'] === 'true'
  )
})

const handleEdit = () => {
  if (!props.storageClass) return
  router.push({
    name: 'edit-workload',
    params: {
      kind: KUBERNETES_RESOURCE_KIND.StorageClass,
      namespace: 'default',
      name: props.storageClass.name
    }
  })
}

const fetchRawYaml = async () => {
  if (!props.storageClass || !props.visible) return
  const reqId = ++currentRequestId.value
  isYamlLoading.value = true
  yamlError.value = null
  try {
    await kubernetesService.getResourceRaw({
      namespace: '',
      kind: KUBERNETES_RESOURCE_KIND.StorageClass,
      name: props.storageClass.name,
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
  if (!props.visible || !props.storageClass) return
  if (payload?.requestId && payload.requestId !== String(currentRequestId.value)) return

  const matchesKind =
    !payload?.kind ||
    payload.kind.toLowerCase() === KUBERNETES_RESOURCE_KIND.StorageClass.toLowerCase()
  const matchesName = payload?.name === props.storageClass.name

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
    props.storageClass &&
    (!payload?.kind || payload.kind === KUBERNETES_RESOURCE_KIND.StorageClass)
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
  () => [props.visible, props.storageClass?.name] as const,
  ([newVisible, newName], prev) => {
    const oldVisible = prev?.[0]
    const oldName = prev?.[1]
    if (newVisible && props.storageClass) {
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
    if (!props.storageClass) return ''
    return yaml.stringify({
      apiVersion: 'storage.k8s.io/v1',
      kind: 'StorageClass',
      metadata: {
        name: props.storageClass.name
      },
      provisioner: props.storageClass.provisioner,
      reclaimPolicy: props.storageClass.reclaimPolicy,
      volumeBindingMode: props.storageClass.volumeBindingMode,
      allowVolumeExpansion: props.storageClass.allowVolumeExpansion
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

const scLabels = computed(() => {
  const meta = (rawManifest.value?.metadata as Record<string, unknown>) || {}
  return (meta.labels as Record<string, string>) || {}
})

const scAnnotations = computed(() => {
  const meta = (rawManifest.value?.metadata as Record<string, unknown>) || {}
  return (meta.annotations as Record<string, string>) || {}
})

const scUid = computed(() => {
  const meta = (rawManifest.value?.metadata as Record<string, unknown>) || {}
  return (meta.uid as string) || '-'
})

const scParameters = computed(() => {
  return (rawManifest.value?.parameters as Record<string, string>) || {}
})

const mountOptions = computed<string[]>(() => {
  return (rawManifest.value?.mountOptions as string[]) || []
})
</script>

<template>
  <BaseResourceDrawer
    data-component="storage-class-details-drawer"
    v-model:active-tab="activeTab"
    :visible="props.visible"
    :has-resource="!!props.storageClass"
    :title="props.storageClass?.name ?? ''"
    :kind="KUBERNETES_RESOURCE_KIND.StorageClass"
    kind-severity="info"
    status-badge-class="bg-(--success)"
    :age="props.storageClass?.age"
    @update:visible="emit('update:visible', $event)"
  >
    <template #tags>
      <Tag
        rounded
        severity="info"
        class="font-mono"
        :value="KUBERNETES_RESOURCE_KIND.StorageClass"
      />
      <Tag v-if="isDefaultClass" rounded severity="success" class="font-mono" value="Default" />
      <Tag
        rounded
        :severity="props.storageClass?.allowVolumeExpansion ? 'success' : 'secondary'"
        class="font-mono"
        :value="props.storageClass?.allowVolumeExpansion ? 'Resizable' : 'Static'"
      />
    </template>

    <template #actions>
      <Button
        v-if="props.storageClass"
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
        <span>Events ({{ scEvents.length }})</span>
      </Tab>
    </template>

    <template #overview>
      <div v-if="props.storageClass" class="space-y-6">
        <!-- General Info -->
        <div>
          <h3 class="text-xs font-bold text-muted-color uppercase tracking-wider mb-3">General</h3>
          <div
            class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-card)] p-4 flex flex-col gap-3 text-xs font-ui"
          >
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Name</span>
              <span class="col-span-2 font-mono text-primary font-semibold">{{
                props.storageClass.name
              }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Provisioner</span>
              <span class="col-span-2 font-mono text-primary break-all">{{
                props.storageClass.provisioner
              }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Reclaim Policy</span>
              <span class="col-span-2 font-mono text-primary">{{
                props.storageClass.reclaimPolicy
              }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Volume Binding Mode</span>
              <span class="col-span-2 font-mono text-primary">{{
                props.storageClass.volumeBindingMode
              }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Allow Volume Expansion</span>
              <span class="col-span-2 font-mono text-primary">{{
                props.storageClass.allowVolumeExpansion ? 'True' : 'False'
              }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Default Class</span>
              <span class="col-span-2 font-mono text-primary">{{
                isDefaultClass ? 'Yes' : 'No'
              }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Age</span>
              <span class="col-span-2 text-primary"
                ><ReactiveAge :age="props.storageClass.age"
              /></span>
            </div>
            <div class="grid grid-cols-3 items-start gap-4">
              <span class="text-muted-color font-semibold shrink-0">UID</span>
              <span class="col-span-2 font-mono text-[10px] text-muted-color truncate">{{
                scUid
              }}</span>
            </div>
          </div>
        </div>

        <!-- Parameters -->
        <KeyValueBadgeList :items="scParameters" title="Parameters" variant="list" />

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
        <KeyValueBadgeList :items="scLabels" title="Labels" variant="tag" />
        <KeyValueBadgeList :items="scAnnotations" title="Annotations" variant="list" />
      </div>
    </template>

    <template #extra-panels>
      <TabPanel value="events">
        <WorkloadEventsTab :events="scEvents" />
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
