<script setup lang="ts">
import BaseResourceDrawer from '@/components/shared/BaseResourceDrawer.vue'
import KeyValueBadgeList from '@/components/shared/KeyValueBadgeList.vue'
import ResourceYamlTab from '@/components/shared/ResourceYamlTab.vue'
import ScaleHpaDialog from '@/components/shared/ScaleHpaDialog.vue'
import WorkloadEventsTab from '@/components/workloads/WorkloadEventsTab.vue'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes.ts'
import { useResourceRawManifest } from '@/composables/useResourceRawManifest.ts'
import { kubernetesService } from '@/services/kubernetesService.ts'
import { useKubernetesStore } from '@/stores/kubernetesStore.ts'
import type { EventInfo, HpaInfo } from '@/types/kubernetes.ts'
import { sortEventsDesc } from '@/utils/events.ts'
import { formatHpaConditions, getHpaStatusBadgeClass, resolveHpaTargetRoute } from '@/utils/hpa.ts'
import { Activity, ExternalLink, Sliders } from '@lucide/vue'
import Button from 'primevue/button'
import Tab from 'primevue/tab'
import TabPanel from 'primevue/tabpanel'
import Tag from 'primevue/tag'
import { useDialog } from 'primevue/usedialog'
import { useToast } from 'primevue/usetoast'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

const props = defineProps<{
  visible: boolean
  hpa: HpaInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const router = useRouter()
const toast = useToast()
const dialog = useDialog()
const k8sStore = useKubernetesStore()

const activeTab = ref('overview')

const { displayedYaml, isYamlLoading, copied, copyYaml } = useResourceRawManifest({
  kind: computed(() => (props.hpa ? KUBERNETES_RESOURCE_KIND.HorizontalPodAutoscaler : '')),
  name: computed(() => props.hpa?.name),
  namespace: computed(() => props.hpa?.namespace),
  visible: computed(() => props.visible)
})

const hpaEvents = computed<EventInfo[]>(() => {
  if (!props.hpa) return []
  const ns = props.hpa.namespace
  const name = props.hpa.name
  const filtered = k8sStore.events.filter((e) => e.namespace === ns && e.objectName === name)
  return sortEventsDesc(filtered)
})

const targetRoute = computed(() => {
  if (!props.hpa) return null
  return resolveHpaTargetRoute(
    props.hpa.scaleTargetRef.kind,
    props.hpa.scaleTargetRef.name,
    props.hpa.namespace
  )
})

const navigateToTarget = () => {
  if (!targetRoute.value) return
  emit('update:visible', false)
  router.push(targetRoute.value)
}

const handleEditBounds = () => {
  if (!props.hpa) return
  const hpa = props.hpa
  dialog.open(ScaleHpaDialog, {
    props: {
      header: 'Edit Replica Bounds',
      style: {
        width: '360px'
      },
      modal: true
    },
    data: {
      name: hpa.name,
      namespace: hpa.namespace,
      minReplicas: hpa.minReplicas,
      maxReplicas: hpa.maxReplicas
    },
    onClose: async (options) => {
      const bounds = options?.data
      if (
        bounds &&
        typeof bounds.minReplicas === 'number' &&
        typeof bounds.maxReplicas === 'number'
      ) {
        try {
          await kubernetesService.updateHpaReplicas({
            namespace: hpa.namespace,
            name: hpa.name,
            minReplicas: bounds.minReplicas,
            maxReplicas: bounds.maxReplicas
          })
          toast.add({
            severity: 'success',
            summary: 'Success',
            detail: `Updated replica bounds for ${hpa.name}`,
            life: 3000
          })
        } catch (e) {
          toast.add({
            severity: 'error',
            summary: 'Error',
            detail: e instanceof Error ? e.message : 'Failed to update replica bounds',
            life: 5000
          })
        }
      }
    }
  })
}

const handleEditYaml = () => {
  if (!props.hpa) return
  emit('update:visible', false)
  router.push({
    name: 'edit-workload',
    params: {
      kind: KUBERNETES_RESOURCE_KIND.HorizontalPodAutoscaler,
      namespace: props.hpa.namespace,
      name: props.hpa.name
    }
  })
}

const formattedConditions = computed(() => {
  if (!props.hpa?.conditions) return []
  return formatHpaConditions(props.hpa.conditions)
})
</script>

<template>
  <BaseResourceDrawer
    data-component="hpa-details-drawer"
    :visible="props.visible"
    :hasResource="!!props.hpa"
    :title="props.hpa?.name"
    kind="HPA"
    kindSeverity="info"
    :statusBadgeClass="getHpaStatusBadgeClass(props.hpa?.status)"
    :namespace="props.hpa?.namespace"
    :age="props.hpa?.age"
    v-model:activeTab="activeTab"
    @update:visible="emit('update:visible', $event)"
  >
    <!-- Action buttons in drawer header -->
    <template #actions>
      <Button
        v-tooltip.top="'Edit replica bounds'"
        severity="secondary"
        variant="text"
        size="small"
        class="flex items-center gap-1.5 rounded-[var(--ui-radius-control)]"
        @click="handleEditBounds"
      >
        <Sliders class="w-3.5 h-3.5" />
        <span class="text-xs">Edit Bounds</span>
      </Button>

      <Button
        v-tooltip.top="'Edit YAML manifest'"
        severity="secondary"
        variant="text"
        size="small"
        class="rounded-[var(--ui-radius-control)]"
        @click="handleEditYaml"
      >
        <span class="text-xs font-mono">YAML</span>
      </Button>
    </template>

    <template #extra-tabs>
      <Tab value="events" class="text-xs flex items-center gap-1.5 py-2.5 px-3">
        <Activity class="w-3.5 h-3.5" />
        <span>Events</span>
      </Tab>
    </template>

    <!-- Overview Content -->
    <template #overview>
      <div v-if="props.hpa" class="flex flex-col gap-6">
        <!-- Scale Target Reference Card -->
        <div class="flex flex-col gap-2 p-4 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/40">
          <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
            Target Workload
          </span>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <Tag severity="secondary" class="font-mono text-xs">
                {{ props.hpa.scaleTargetRef.kind }}
              </Tag>
              <span class="font-mono font-medium text-primary text-sm">
                {{ props.hpa.scaleTargetRef.name }}
              </span>
            </div>

            <Button
              v-if="targetRoute"
              v-tooltip.top="'Navigate to workload'"
              severity="secondary"
              variant="text"
              size="small"
              class="flex items-center gap-1 rounded-[var(--ui-radius-control)] text-xs"
              @click="navigateToTarget"
            >
              <span>View Target</span>
              <ExternalLink class="w-3.5 h-3.5" />
            </Button>
          </div>
        </div>

        <!-- Replica Bounds & Status -->
        <div class="flex flex-col gap-2">
          <div class="flex items-center justify-between">
            <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
              Replicas & Bounds
            </span>
            <Button
              variant="text"
              size="small"
              severity="secondary"
              class="flex items-center gap-1 rounded-[var(--ui-radius-control)] text-xs"
              @click="handleEditBounds"
            >
              <Sliders class="w-3 h-3" />
              <span>Modify</span>
            </Button>
          </div>

          <div class="grid grid-cols-4 gap-2">
            <div
              class="flex flex-col p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/40 text-center"
            >
              <span class="text-xs text-muted-color">Min</span>
              <span class="text-base font-bold font-mono text-primary">{{
                props.hpa.minReplicas
              }}</span>
            </div>
            <div
              class="flex flex-col p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/40 text-center"
            >
              <span class="text-xs text-muted-color">Max</span>
              <span class="text-base font-bold font-mono text-primary">{{
                props.hpa.maxReplicas
              }}</span>
            </div>
            <div
              class="flex flex-col p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/40 text-center"
            >
              <span class="text-xs text-muted-color">Current</span>
              <span class="text-base font-bold font-mono text-primary">{{
                props.hpa.currentReplicas
              }}</span>
            </div>
            <div
              class="flex flex-col p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/40 text-center"
            >
              <span class="text-xs text-muted-color">Desired</span>
              <span class="text-base font-bold font-mono text-primary">{{
                props.hpa.desiredReplicas
              }}</span>
            </div>
          </div>

          <div
            v-if="props.hpa.lastScaleTime"
            class="text-xs text-muted-color flex items-center gap-1 mt-1"
          >
            <span>Last scaled:</span>
            <span class="font-mono text-primary">{{ props.hpa.lastScaleTime }}</span>
          </div>
        </div>

        <!-- Metrics Targets -->
        <div class="flex flex-col gap-2">
          <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
            Metrics & Utilization
          </span>
          <div v-if="props.hpa.metrics.length === 0" class="text-xs text-muted-color">
            No metrics defined.
          </div>
          <div v-else class="flex flex-col gap-2">
            <div
              v-for="m in props.hpa.metrics"
              :key="m.name"
              class="flex items-center justify-between p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/40"
            >
              <div class="flex items-center gap-2">
                <Tag severity="info" class="font-mono text-xs">{{ m.metricType }}</Tag>
                <span class="font-mono font-medium text-primary text-xs">{{ m.name }}</span>
              </div>
              <div class="flex items-center gap-2 font-mono text-xs">
                <span class="text-muted-color">Current:</span>
                <span class="font-bold text-primary">{{
                  m.currentValue || '&lt;unknown&gt;'
                }}</span>
                <span class="text-muted-color/60">/</span>
                <span class="text-muted-color">Target:</span>
                <span class="font-bold text-primary">{{ m.targetValue }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Conditions -->
        <div class="flex flex-col gap-2">
          <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
            Autoscaler Conditions
          </span>
          <div v-if="formattedConditions.length === 0" class="text-xs text-muted-color">
            No conditions reported.
          </div>
          <div v-else class="flex flex-col gap-2">
            <div
              v-for="c in formattedConditions"
              :key="c.type"
              class="flex flex-col gap-1 p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/40"
            >
              <div class="flex items-center justify-between">
                <span class="font-mono font-semibold text-xs text-primary">{{ c.type }}</span>
                <Tag :severity="c.severity" :value="c.status" class="text-xs" />
              </div>
              <div v-if="c.reason" class="text-xs text-muted-color font-mono">
                Reason: {{ c.reason }}
              </div>
              <div v-if="c.message" class="text-xs text-muted-color">
                {{ c.message }}
              </div>
            </div>
          </div>
        </div>

        <!-- Labels & Annotations -->
        <KeyValueBadgeList :items="props.hpa.labels" title="Labels" variant="tag" />
        <KeyValueBadgeList :items="props.hpa.annotations" title="Annotations" variant="list" />
      </div>
    </template>

    <!-- Events Panel -->
    <template #extra-panels>
      <TabPanel value="events" class="p-0">
        <WorkloadEventsTab :events="hpaEvents" />
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
