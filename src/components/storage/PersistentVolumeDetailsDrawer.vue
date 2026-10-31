<script setup lang="ts">
import BaseResourceDrawer from '@/components/shared/BaseResourceDrawer.vue'
import KeyValueBadgeList from '@/components/shared/KeyValueBadgeList.vue'
import ReactiveAge from '@/components/shared/ReactiveAge.vue'
import ResourceYamlTab from '@/components/shared/ResourceYamlTab.vue'
import WorkloadEventsTab from '@/components/workloads/WorkloadEventsTab.vue'
import { useResourceRawManifest } from '@/composables/useResourceRawManifest'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { PersistentVolumeInfo } from '@/types/kubernetes'
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
  pv: PersistentVolumeInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const router = useRouter()
const k8sStore = useKubernetesStore()
const { events: clusterEvents } = storeToRefs(k8sStore)

const activeTab = ref('overview')

const { rawManifest, displayedYaml, isYamlLoading, copied, copyYaml } = useResourceRawManifest({
  kind: KUBERNETES_RESOURCE_KIND.PersistentVolume,
  name: computed(() => props.pv?.name),
  visible: computed(() => props.visible),
  fallbackResource: computed(() => {
    if (!props.pv) return null
    return {
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
    }
  })
})

const pvEvents = computed(() => {
  if (!props.pv) return []
  const name = props.pv.name
  const filtered = clusterEvents.value.filter((ev) => {
    return ev.objectKind === KUBERNETES_RESOURCE_KIND.PersistentVolume && ev.objectName === name
  })
  return sortEventsDesc(filtered)
})

const handleEdit = () => {
  if (!props.pv) return
  const target = resolveEditTargetParams(props.pv, KUBERNETES_RESOURCE_KIND.PersistentVolume)
  router.push({
    name: 'edit-workload',
    params: {
      kind: target.kind,
      namespace: target.namespace,
      name: target.name
    }
  })
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
    :status-text="props.pv?.status ?? 'Unknown'"
    @update:visible="emit('update:visible', $event)"
  >
    <template #header-actions>
      <Button
        v-if="props.pv"
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
        <span>Events ({{ pvEvents.length }})</span>
      </Tab>
    </template>

    <template #overview>
      <div v-if="props.pv" class="flex flex-col gap-6">
        <!-- Identity & Status Panel -->
        <div class="flex flex-col gap-3">
          <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
            Volume Information
          </span>
          <div class="grid grid-cols-2 gap-x-4 gap-y-3 text-xs">
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Name</span>
              <span class="font-mono text-primary truncate">{{ props.pv.name }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Status</span>
              <div>
                <Tag
                  :value="props.pv.status"
                  :class="getVolumeStatusBadgeClass(props.pv.status)"
                  class="text-[11px]"
                />
              </div>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Capacity</span>
              <span class="font-semibold text-primary">{{ props.pv.capacity }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Access Mode</span>
              <span class="font-mono text-primary">{{ props.pv.accessMode }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Reclaim Policy</span>
              <span class="font-medium text-primary">{{ props.pv.reclaimPolicy }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Storage Class</span>
              <span class="font-mono text-primary">{{ props.pv.storageClass || '-' }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Volume Mode</span>
              <span class="text-primary">{{ props.pv.volumeMode || 'Filesystem' }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Age</span>
              <ReactiveAge :age="props.pv.age" class="text-primary font-medium" />
            </div>
            <div class="col-span-2 flex flex-col gap-1">
              <span class="text-muted-color">Claim Reference</span>
              <span class="font-mono text-primary">{{ claimRefText }}</span>
            </div>
            <div class="col-span-2 flex flex-col gap-1">
              <span class="text-muted-color">UID</span>
              <span class="font-mono text-xs text-muted-color break-all">{{ pvUid }}</span>
            </div>
          </div>
        </div>

        <!-- Volume Plugin / Source Details -->
        <div class="flex flex-col gap-2">
          <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
            Storage Plugin Source
          </span>
          <div
            class="p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30 flex flex-col gap-1 text-xs"
          >
            <span class="font-medium text-primary">{{ volumeSource.type }} Volume</span>
            <span class="font-mono text-muted-color break-all">{{ volumeSource.details }}</span>
          </div>
        </div>

        <!-- Mount Options (if present) -->
        <div v-if="mountOptions.length > 0" class="flex flex-col gap-2">
          <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
            Mount Options
          </span>
          <div class="flex flex-wrap gap-1.5">
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
