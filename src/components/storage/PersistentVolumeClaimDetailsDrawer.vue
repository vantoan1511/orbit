<script setup lang="ts">
import BaseResourceDrawer from '@/components/shared/BaseResourceDrawer.vue'
import KeyValueBadgeList from '@/components/shared/KeyValueBadgeList.vue'
import ReactiveAge from '@/components/shared/ReactiveAge.vue'
import ResourceYamlTab from '@/components/shared/ResourceYamlTab.vue'
import WorkloadEventsTab from '@/components/workloads/WorkloadEventsTab.vue'
import { useResourceRawManifest } from '@/composables/useResourceRawManifest'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { PersistentVolumeClaimInfo } from '@/types/kubernetes'
import { sortEventsDesc } from '@/utils/events'
import { getVolumeStatusBadgeClass } from '@/utils/severity'
import { resolveEditTargetParams } from '@/utils/workloadActions'
import { Activity, Pencil } from '@lucide/vue'
import { storeToRefs } from 'pinia'
import Button from 'primevue/button'
import Tab from 'primevue/tab'
import TabPanel from 'primevue/tabpanel'
import Tag from 'primevue/tag'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

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

const { rawManifest, displayedYaml, isYamlLoading, copied, copyYaml } = useResourceRawManifest({
  kind: KUBERNETES_RESOURCE_KIND.PersistentVolumeClaim,
  name: computed(() => props.pvc?.name),
  namespace: computed(() => props.pvc?.namespace),
  visible: computed(() => props.visible),
  fallbackResource: computed(() => {
    if (!props.pvc) return null
    return {
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
    }
  })
})

const pvcEvents = computed(() => {
  if (!props.pvc) return []
  const name = props.pvc.name
  const ns = props.pvc.namespace
  const filtered = clusterEvents.value.filter((ev) => {
    return (
      ev.objectKind === KUBERNETES_RESOURCE_KIND.PersistentVolumeClaim &&
      ev.namespace === ns &&
      ev.objectName === name
    )
  })
  return sortEventsDesc(filtered)
})

const handleEdit = () => {
  if (!props.pvc) return
  const target = resolveEditTargetParams(props.pvc, KUBERNETES_RESOURCE_KIND.PersistentVolumeClaim)
  router.push({
    name: 'edit-workload',
    params: {
      kind: target.kind,
      namespace: target.namespace,
      name: target.name
    }
  })
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
    :status-text="props.pvc?.status ?? 'Unknown'"
    @update:visible="emit('update:visible', $event)"
  >
    <template #header-actions>
      <Button
        v-if="props.pvc"
        severity="secondary"
        size="small"
        class="flex items-center gap-1.5"
        @click="handleEdit"
      >
        <Pencil class="w-3.5 h-3.5 text-muted-color" />
        <span>Edit</span>
      </Button>
    </template>

    <template #extra-tabs>
      <Tab value="events" class="flex items-center gap-1.5">
        <Activity class="w-3.5 h-3.5" />
        <span>Events ({{ pvcEvents.length }})</span>
      </Tab>
    </template>

    <template #overview>
      <div v-if="props.pvc" class="flex flex-col gap-6">
        <!-- Claim Information -->
        <div class="flex flex-col gap-3">
          <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
            Claim Information
          </span>
          <div class="grid grid-cols-2 gap-x-4 gap-y-3 text-xs">
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Name</span>
              <span class="font-mono text-primary truncate">{{ props.pvc.name }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Namespace</span>
              <span class="font-mono text-primary truncate">{{ props.pvc.namespace }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Status</span>
              <div>
                <Tag
                  :value="props.pvc.status"
                  :class="getVolumeStatusBadgeClass(props.pvc.status)"
                  class="text-[11px]"
                />
              </div>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Bound Volume</span>
              <span class="font-mono text-primary truncate">{{ props.pvc.volume || '-' }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Requested Storage</span>
              <span class="font-semibold text-primary">{{ props.pvc.capacity }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Actual Capacity</span>
              <span class="font-semibold text-primary">{{ actualCapacity }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Access Mode</span>
              <span class="font-mono text-primary">{{ props.pvc.accessMode }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Storage Class</span>
              <span class="font-mono text-primary">{{ props.pvc.storageClass || '-' }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Volume Mode</span>
              <span class="text-primary">{{ volumeMode }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Age</span>
              <ReactiveAge :age="props.pvc.age" class="text-primary font-medium" />
            </div>
            <div class="col-span-2 flex flex-col gap-1">
              <span class="text-muted-color">UID</span>
              <span class="font-mono text-xs text-muted-color break-all">{{ pvcUid }}</span>
            </div>
          </div>
        </div>

        <!-- Conditions (e.g. Resizing) -->
        <div v-if="conditions.length > 0" class="flex flex-col gap-2">
          <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
            Volume Conditions
          </span>
          <div class="flex flex-col gap-2">
            <div
              v-for="cond in conditions"
              :key="cond.type"
              class="p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30 flex flex-col gap-1 text-xs"
            >
              <div class="flex items-center justify-between">
                <span class="font-semibold text-primary">{{ cond.type }}</span>
                <Tag
                  :value="cond.status"
                  :severity="cond.status === 'True' ? 'success' : 'secondary'"
                  class="text-[11px]"
                />
              </div>
              <p v-if="cond.message" class="text-muted-color">{{ cond.message }}</p>
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
