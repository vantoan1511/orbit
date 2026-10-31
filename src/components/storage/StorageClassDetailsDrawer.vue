<script setup lang="ts">
import BaseResourceDrawer from '@/components/shared/BaseResourceDrawer.vue'
import KeyValueBadgeList from '@/components/shared/KeyValueBadgeList.vue'
import ReactiveAge from '@/components/shared/ReactiveAge.vue'
import ResourceYamlTab from '@/components/shared/ResourceYamlTab.vue'
import WorkloadEventsTab from '@/components/workloads/WorkloadEventsTab.vue'
import { useResourceRawManifest } from '@/composables/useResourceRawManifest'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { StorageClassInfo } from '@/types/kubernetes'
import { sortEventsDesc } from '@/utils/events'
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
  storageClass: StorageClassInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const router = useRouter()
const k8sStore = useKubernetesStore()
const { events: clusterEvents } = storeToRefs(k8sStore)

const activeTab = ref('overview')

const { rawManifest, displayedYaml, isYamlLoading, copied, copyYaml } = useResourceRawManifest({
  kind: KUBERNETES_RESOURCE_KIND.StorageClass,
  name: computed(() => props.storageClass?.name),
  visible: computed(() => props.visible),
  fallbackResource: computed(() => {
    if (!props.storageClass) return null
    return {
      apiVersion: 'storage.k8s.io/v1',
      kind: 'StorageClass',
      metadata: {
        name: props.storageClass.name
      },
      provisioner: props.storageClass.provisioner,
      reclaimPolicy: props.storageClass.reclaimPolicy,
      volumeBindingMode: props.storageClass.volumeBindingMode,
      allowVolumeExpansion: props.storageClass.allowVolumeExpansion
    }
  })
})

const scEvents = computed(() => {
  if (!props.storageClass) return []
  const name = props.storageClass.name
  const filtered = clusterEvents.value.filter((ev) => {
    return ev.objectKind === KUBERNETES_RESOURCE_KIND.StorageClass && ev.objectName === name
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
  const target = resolveEditTargetParams(props.storageClass, KUBERNETES_RESOURCE_KIND.StorageClass)
  router.push({
    name: 'edit-workload',
    params: {
      kind: target.kind,
      namespace: target.namespace,
      name: target.name
    }
  })
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
  const meta = rawManifest.value as Record<string, unknown> | undefined
  return (meta?.mountOptions as string[]) || []
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
        v-if="props.storageClass?.allowVolumeExpansion"
        rounded
        severity="secondary"
        class="font-mono"
        value="Expansion"
      />
    </template>

    <template #header-actions>
      <Button
        v-if="props.storageClass"
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
        <span>Events ({{ scEvents.length }})</span>
      </Tab>
    </template>

    <template #overview>
      <div v-if="props.storageClass" class="flex flex-col gap-6">
        <!-- Provisioner Details -->
        <div class="flex flex-col gap-3">
          <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
            Provisioner Details
          </span>
          <div class="grid grid-cols-2 gap-x-4 gap-y-3 text-xs">
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Name</span>
              <span class="font-mono text-primary truncate">{{ props.storageClass.name }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Provisioner</span>
              <span class="font-mono text-primary truncate">{{
                props.storageClass.provisioner
              }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Reclaim Policy</span>
              <span class="font-medium text-primary">{{ props.storageClass.reclaimPolicy }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Volume Binding Mode</span>
              <span class="font-mono text-primary">{{ props.storageClass.volumeBindingMode }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Volume Expansion</span>
              <div>
                <Tag
                  :value="props.storageClass.allowVolumeExpansion ? 'Allowed' : 'Disabled'"
                  :severity="props.storageClass.allowVolumeExpansion ? 'success' : 'secondary'"
                  class="text-[11px]"
                />
              </div>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Default Class</span>
              <div>
                <Tag
                  :value="isDefaultClass ? 'Yes' : 'No'"
                  :severity="isDefaultClass ? 'success' : 'secondary'"
                  class="text-[11px]"
                />
              </div>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-muted-color">Age</span>
              <ReactiveAge :age="props.storageClass.age" class="text-primary font-medium" />
            </div>
            <div class="col-span-2 flex flex-col gap-1">
              <span class="text-muted-color">UID</span>
              <span class="font-mono text-xs text-muted-color break-all">{{ scUid }}</span>
            </div>
          </div>
        </div>

        <!-- Provisioner Parameters (if present) -->
        <div v-if="Object.keys(scParameters).length > 0" class="flex flex-col gap-2">
          <KeyValueBadgeList :items="scParameters" title="Parameters" variant="list" />
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
