<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import TableFilterSelect from '@/components/shared/TableFilterSelect.vue'
import { KUBERNETES_HPA_STATUS, KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes.ts'
import { kubernetesService } from '@/services/kubernetesService.ts'
import { useKubernetesStore } from '@/stores/kubernetesStore.ts'
import type { HpaInfo } from '@/types/kubernetes.ts'
import { getHpaStatusBadgeClass, getHpaStatusSeverity, resolveHpaTargetRoute } from '@/utils/hpa.ts'
import { ExternalLink, Sliders } from '@lucide/vue'
import Button from 'primevue/button'
import Column from 'primevue/column'
import InputNumber from 'primevue/inputnumber'
import Popover from 'primevue/popover'
import Tag from 'primevue/tag'
import { useToast } from 'primevue/usetoast'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import HpaDetailsDrawer from './HpaDetailsDrawer.vue'

const k8sStore = useKubernetesStore()
const router = useRouter()
const toast = useToast()
const loading = ref(false)

const columns = [
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'target', header: 'Target', visible: true },
  { field: 'metrics', header: 'Metrics', visible: true },
  { field: 'minMax', header: 'Min / Max', visible: true },
  { field: 'replicas', header: 'Replicas', visible: true },
  { field: 'conditions', header: 'Conditions', visible: true },
  { field: 'age', header: 'Age', visible: true }
]

const selectedStatus = ref('All Statuses')
const statusOptions = ['All Statuses', ...Object.values(KUBERNETES_HPA_STATUS)]

const filteredHpas = computed(() => {
  return k8sStore.hpas.filter((hpa) => {
    if (selectedStatus.value !== 'All Statuses') {
      if (hpa.status !== selectedStatus.value) return false
    }
    return true
  })
})

const fetchHpas = async () => {
  loading.value = true
  try {
    await kubernetesService.getHpas()
  } catch (e) {
    console.error('Error fetching HPAs', e)
  } finally {
    loading.value = false
  }
}

// Inline Replica Bounds Popover editing
const boundsPopover = ref()
const editingHpa = ref<HpaInfo | null>(null)
const editMinReplicas = ref(1)
const editMaxReplicas = ref(10)
const isUpdatingBounds = ref(false)

const openBoundsEdit = (event: MouseEvent, hpa: HpaInfo) => {
  editingHpa.value = hpa
  editMinReplicas.value = hpa.minReplicas
  editMaxReplicas.value = hpa.maxReplicas
  boundsPopover.value?.toggle(event)
}

const saveBounds = async () => {
  if (!editingHpa.value) return
  if (editMinReplicas.value < 1 || editMaxReplicas.value < editMinReplicas.value) return

  isUpdatingBounds.value = true
  try {
    await kubernetesService.updateHpaReplicas({
      namespace: editingHpa.value.namespace,
      name: editingHpa.value.name,
      minReplicas: editMinReplicas.value,
      maxReplicas: editMaxReplicas.value
    })
    toast.add({
      severity: 'success',
      summary: 'Success',
      detail: `Updated replica bounds for ${editingHpa.value.name}`,
      life: 3000
    })
    boundsPopover.value?.hide()
  } catch (e) {
    toast.add({
      severity: 'error',
      summary: 'Error',
      detail: e instanceof Error ? e.message : 'Failed to update bounds',
      life: 5000
    })
  } finally {
    isUpdatingBounds.value = false
  }
}

const navigateToTarget = (hpa: HpaInfo) => {
  const route = resolveHpaTargetRoute(
    hpa.scaleTargetRef.kind,
    hpa.scaleTargetRef.name,
    hpa.namespace
  )
  if (route) {
    router.push(route)
  }
}
</script>

<template>
  <GenericResourceTable
    data-component="hpas-table"
    :data="filteredHpas"
    :initialColumns="columns"
    :hideStatusFilter="true"
    :hideStatusColumn="true"
    :searchFields="['name', 'scaleTargetRef.name', 'namespace']"
    :kind="KUBERNETES_RESOURCE_KIND.HorizontalPodAutoscaler"
    searchPlaceholder="Search HPAs, targets, or namespaces..."
    emptyMessage="No HorizontalPodAutoscalers found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} HPAs"
    :loading="loading || k8sStore.hpasLoading"
    @refresh="fetchHpas"
  >
    <!-- Custom Filter -->
    <template #filters>
      <TableFilterSelect v-model="selectedStatus" :options="statusOptions" class="min-w-44" />
    </template>

    <template #default="{ visibleCols }">
      <!-- Status Column -->
      <Column v-if="visibleCols['status']" field="status" header="Status" sortable class="p-3">
        <template #body="{ data }">
          <div class="flex items-center gap-2">
            <span
              class="w-2 h-2 rounded-full shrink-0"
              :class="getHpaStatusBadgeClass(data.status)"
            ></span>
            <Tag
              :severity="getHpaStatusSeverity(data.status)"
              :value="data.status"
              class="font-mono text-xs"
            />
          </div>
        </template>
      </Column>

      <!-- Target Reference Column -->
      <Column
        v-if="visibleCols['target']"
        header="Target"
        sortable
        field="scaleTargetRef.name"
        class="p-3"
      >
        <template #body="{ data }">
          <div
            class="group inline-flex items-center gap-1.5 cursor-pointer font-mono text-xs hover:text-(--accent) transition-colors"
            @click.stop="navigateToTarget(data)"
          >
            <Tag severity="secondary" class="font-mono text-[11px] py-0 px-1">
              {{ data.scaleTargetRef.kind }}
            </Tag>
            <span class="font-medium text-primary group-hover:text-(--accent)">
              {{ data.scaleTargetRef.name }}
            </span>
            <ExternalLink
              class="w-3 h-3 opacity-0 group-hover:opacity-100 transition-opacity text-muted-color"
            />
          </div>
        </template>
      </Column>

      <!-- Metrics Summary Column -->
      <Column v-if="visibleCols['metrics']" header="Metrics" class="p-3 max-w-64">
        <template #body="{ data }">
          <div class="flex flex-col gap-1">
            <span
              v-for="(summary, idx) in data.metricsSummary"
              :key="idx"
              class="font-mono text-xs text-primary truncate"
            >
              {{ summary }}
            </span>
            <span v-if="data.metricsSummary.length === 0" class="text-xs text-muted-color">
              -
            </span>
          </div>
        </template>
      </Column>

      <!-- Min / Max Column with Inline Edit Popover -->
      <Column v-if="visibleCols['minMax']" header="Min / Max" class="p-3">
        <template #body="{ data }">
          <div
            aria-label="Click to edit replica bounds"
            class="group inline-flex items-center gap-1.5 cursor-pointer font-mono text-xs text-primary hover:text-(--accent) transition-colors"
            @click.stop="openBoundsEdit($event, data)"
          >
            <span>{{ data.minReplicas }} / {{ data.maxReplicas }}</span>
            <Sliders
              class="w-3 h-3 shrink-0 opacity-0 group-hover:opacity-60 transition-opacity text-muted-color"
            />
          </div>
        </template>
      </Column>

      <!-- Replicas Column (Current / Desired) -->
      <Column v-if="visibleCols['replicas']" header="Replicas" class="p-3 text-right">
        <template #body="{ data }">
          <span class="font-mono font-medium text-xs text-primary">
            {{ data.currentReplicas }} / {{ data.desiredReplicas }}
          </span>
        </template>
      </Column>

      <!-- Conditions Column -->
      <Column v-if="visibleCols['conditions']" header="Conditions" class="p-3 max-w-48">
        <template #body="{ data }">
          <div class="flex flex-wrap gap-1">
            <Tag
              v-for="c in data.conditions"
              :key="c.type"
              :severity="
                c.status === 'True'
                  ? c.type === 'ScalingLimited'
                    ? 'warn'
                    : 'success'
                  : 'secondary'
              "
              :value="c.type"
              class="font-mono text-[10px] py-0 px-1"
              v-tooltip.top="c.message || c.reason || c.type"
            />
            <span v-if="data.conditions.length === 0" class="text-xs text-muted-color"> - </span>
          </div>
        </template>
      </Column>
    </template>

    <!-- Drawer Slot -->
    <template #drawer="{ selectedItem, visible, close }">
      <HpaDetailsDrawer
        :visible="visible"
        :hpa="selectedItem"
        @update:visible="!$event && close()"
      />
    </template>
  </GenericResourceTable>

  <!-- Inline Replica Bounds Edit Popover -->
  <Popover ref="boundsPopover">
    <div class="flex flex-col gap-3 p-1 min-w-56" @click.stop>
      <div class="flex flex-col gap-0.5">
        <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
          Replica Bounds
        </span>
        <span class="text-xs text-muted-color font-mono truncate">
          {{ editingHpa?.name }}
        </span>
      </div>

      <div class="grid grid-cols-2 gap-2">
        <div class="flex flex-col gap-1">
          <label class="text-[11px] text-muted-color font-semibold">Min</label>
          <InputNumber
            v-model="editMinReplicas"
            :min="1"
            :max="editMaxReplicas"
            showButtons
            buttonLayout="horizontal"
            size="small"
            class="w-full"
            inputClass="text-center font-mono rounded-[var(--ui-radius-control)]"
          />
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-[11px] text-muted-color font-semibold">Max</label>
          <InputNumber
            v-model="editMaxReplicas"
            :min="editMinReplicas"
            :max="1000"
            showButtons
            buttonLayout="horizontal"
            size="small"
            class="w-full"
            inputClass="text-center font-mono rounded-[var(--ui-radius-control)]"
          />
        </div>
      </div>

      <div class="flex justify-end gap-1.5 pt-1">
        <Button
          label="Cancel"
          severity="secondary"
          variant="text"
          size="small"
          class="rounded-[var(--ui-radius-control)] text-xs"
          @click="boundsPopover?.hide()"
        />
        <Button
          label="Save"
          severity="primary"
          size="small"
          class="rounded-[var(--ui-radius-control)] text-xs"
          :loading="isUpdatingBounds"
          :disabled="editMinReplicas < 1 || editMaxReplicas < editMinReplicas"
          @click="saveBounds"
        />
      </div>
    </div>
  </Popover>
</template>
