<script setup lang="ts">
import ClusterCustomizationFields from '@/components/cluster/ClusterCustomizationFields.vue'
import { kubernetesService } from '@/services/kubernetesService'
import type { ClusterInfo } from '@/types/kubernetes'
import { RotateCcw, Server } from '@lucide/vue'
import { Button, Dialog } from 'primevue'
import { computed, ref, watch } from 'vue'

const props = defineProps<{
  visible: boolean
  cluster: ClusterInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void
  (e: 'saved', cluster: ClusterInfo): void
}>()

const alias = ref('')
const selectedColor = ref<string | null>(null)
const tags = ref<string[]>([])
const description = ref('')
const isSaving = ref(false)

// Sync form state when modal opens or target cluster changes
watch(
  () => [props.visible, props.cluster] as const,
  ([isVisible, currentCluster]) => {
    if (isVisible && currentCluster) {
      alias.value = currentCluster.alias || ''
      selectedColor.value = currentCluster.color || null
      tags.value = Array.isArray(currentCluster.tags) ? [...currentCluster.tags] : []
      description.value = currentCluster.description || ''
    }
  },
  { immediate: true }
)

const hasCustomizations = computed(() => {
  return (
    Boolean(alias.value.trim()) ||
    Boolean(selectedColor.value) ||
    tags.value.length > 0 ||
    Boolean(description.value.trim())
  )
})

const handleReset = () => {
  alias.value = ''
  selectedColor.value = null
  tags.value = []
  description.value = ''
}

const handleClose = () => {
  emit('update:visible', false)
}

const handleSave = async () => {
  if (!props.cluster) return

  isSaving.value = true
  try {
    const payload = {
      id: props.cluster.id,
      alias: alias.value.trim() || null,
      tags: tags.value,
      color: selectedColor.value || null,
      description: description.value.trim() || null
    }

    await kubernetesService.updateClusterMetadata(payload)

    emit('saved', {
      ...props.cluster,
      alias: payload.alias || undefined,
      tags: payload.tags,
      color: payload.color || undefined,
      description: payload.description || undefined
    })

    emit('update:visible', false)
  } catch (error) {
    console.error('Failed to update cluster metadata:', error)
  } finally {
    isSaving.value = false
  }
}
</script>

<template>
  <Dialog
    data-component="cluster-edit-modal"
    :visible="visible"
    modal
    :draggable="false"
    class="w-full max-w-lg border border-(--border) bg-(--bg-card) text-primary shadow-[var(--shadow)]"
    :pt="{
      root: { class: 'rounded-[var(--ui-radius-card)] overflow-hidden' },
      header: { class: 'border-b border-(--border) px-5 py-3.5 bg-(--bg-sidebar)/60' },
      content: { class: 'p-5 flex flex-col gap-5' },
      footer: {
        class:
          'border-t border-(--border) px-5 py-3 bg-(--bg-sidebar)/40 flex items-center justify-between'
      }
    }"
    @update:visible="emit('update:visible', $event)"
  >
    <template #header>
      <div class="flex items-center gap-2">
        <Server :size="16" class="text-(--accent)" />
        <span class="text-sm font-bold tracking-tight">Edit Cluster Metadata</span>
      </div>
    </template>

    <div v-if="cluster" class="flex flex-col gap-5">
      <!-- Cluster Context Information Header (Zonal Background) -->
      <div
        class="rounded-[var(--ui-radius-card)] p-3 bg-(--bg-hover)/40 flex flex-col gap-1 select-text"
      >
        <div class="flex items-center justify-between text-xs">
          <span class="text-muted-color font-medium">Context Name:</span>
          <span
            class="font-mono font-semibold text-primary truncate max-w-[280px]"
            v-tooltip.top="cluster.name"
          >
            {{ cluster.name }}
          </span>
        </div>
        <div v-if="cluster.serverUrl" class="flex items-center justify-between text-xs">
          <span class="text-muted-color font-medium">Server API:</span>
          <span
            class="font-mono text-[11px] text-muted-color truncate max-w-[280px]"
            v-tooltip.top="cluster.serverUrl"
          >
            {{ cluster.serverUrl }}
          </span>
        </div>
      </div>

      <!-- Reusable Customization Form Fields -->
      <ClusterCustomizationFields
        v-model:alias="alias"
        v-model:color="selectedColor"
        v-model:tags="tags"
        v-model:description="description"
        :show-description="true"
      />
    </div>

    <template #footer>
      <!-- Left side: Reset to Default action -->
      <Button
        variant="text"
        severity="secondary"
        size="small"
        :disabled="!hasCustomizations || isSaving"
        class="flex items-center gap-1.5 text-xs text-muted-color hover:text-primary"
        @click="handleReset"
      >
        <RotateCcw :size="12" />
        <span>Reset to Default</span>
      </Button>

      <!-- Right side: Cancel & Save -->
      <div class="flex items-center gap-2">
        <Button
          variant="text"
          severity="secondary"
          size="small"
          class="h-8 px-3 text-xs"
          :disabled="isSaving"
          @click="handleClose"
        >
          Cancel
        </Button>
        <Button
          severity="primary"
          size="small"
          class="h-8 px-3 text-xs"
          :loading="isSaving"
          @click="handleSave"
        >
          Save Changes
        </Button>
      </div>
    </template>
  </Dialog>
</template>
