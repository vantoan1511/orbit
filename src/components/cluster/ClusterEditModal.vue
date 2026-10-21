<script setup lang="ts">
import { kubernetesService } from '@/services/kubernetesService'
import type { ClusterInfo } from '@/types/kubernetes'
import { Check, RotateCcw, Server, Tag as TagIcon, X } from '@lucide/vue'
import { Button, Dialog, InputText, Textarea } from 'primevue'
import { computed, ref, watch } from 'vue'

const props = defineProps<{
  visible: boolean
  cluster: ClusterInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void
  (e: 'saved', cluster: ClusterInfo): void
}>()

const COLOR_PRESETS = [
  { label: 'Red (Production)', hex: '#ef4444' },
  { label: 'Orange', hex: '#f97316' },
  { label: 'Amber (Staging)', hex: '#f59e0b' },
  { label: 'Emerald (Dev)', hex: '#10b981' },
  { label: 'Cyan (Test)', hex: '#06b6d4' },
  { label: 'Blue', hex: '#3b82f6' },
  { label: 'Purple', hex: '#8b5cf6' },
  { label: 'Slate (Neutral)', hex: '#64748b' }
] as const

const alias = ref('')
const selectedColor = ref<string | null>(null)
const tags = ref<string[]>([])
const newTagInput = ref('')
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
      newTagInput.value = ''
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

const addTag = () => {
  const raw = newTagInput.value.trim().toLowerCase()
  if (!raw) return

  // Split on commas if multiple entered
  const tokens = raw.split(',').map((t) => t.trim().replace(/^#+/, '')).filter(Boolean)
  for (const token of tokens) {
    if (!tags.value.includes(token)) {
      tags.value.push(token)
    }
  }
  newTagInput.value = ''
}

const handleTagKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter' || e.key === ',') {
    e.preventDefault()
    addTag()
  }
}

const removeTag = (tagToRemove: string) => {
  tags.value = tags.value.filter((t) => t !== tagToRemove)
}

const selectColor = (hex: string) => {
  if (selectedColor.value === hex) {
    selectedColor.value = null
  } else {
    selectedColor.value = hex
  }
}

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
  if (newTagInput.value.trim()) {
    addTag()
  }

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
      root: { class: 'rounded-xl overflow-hidden' },
      header: { class: 'border-b border-(--border) px-5 py-3.5 bg-(--bg-sidebar)/60' },
      content: { class: 'p-5 flex flex-col gap-5' },
      footer: { class: 'border-t border-(--border) px-5 py-3 bg-(--bg-sidebar)/40 flex items-center justify-between' }
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
      <div class="rounded-lg p-3 bg-(--bg-hover)/40 flex flex-col gap-1 select-text">
        <div class="flex items-center justify-between text-xs">
          <span class="text-muted-color font-medium">Context Name:</span>
          <span class="font-mono font-semibold text-primary truncate max-w-[280px]" :title="cluster.name">
            {{ cluster.name }}
          </span>
        </div>
        <div v-if="cluster.serverUrl" class="flex items-center justify-between text-xs">
          <span class="text-muted-color font-medium">Server API:</span>
          <span class="font-mono text-[11px] text-muted-color truncate max-w-[280px]" :title="cluster.serverUrl">
            {{ cluster.serverUrl }}
          </span>
        </div>
      </div>

      <!-- Display Alias Input -->
      <div class="flex flex-col gap-1.5">
        <label for="cluster-alias-input" class="text-xs font-semibold text-primary">
          Display Name (Alias)
        </label>
        <InputText
          id="cluster-alias-input"
          v-model="alias"
          placeholder="e.g. Production US-West"
          class="w-full text-xs h-8"
          size="small"
        />
        <span class="text-[11px] text-muted-color">
          Custom alias displayed across the sidebar, titlebar, and workspace tabs.
        </span>
      </div>

      <!-- Environment Color Indicator -->
      <div class="flex flex-col gap-2">
        <label class="text-xs font-semibold text-primary">
          Environment Color Indicator
        </label>
        <div class="flex items-center gap-2 flex-wrap">
          <Button
            v-for="preset in COLOR_PRESETS"
            :key="preset.hex"
            type="button"
            v-tooltip.top="preset.label"
            :style="{ backgroundColor: preset.hex }"
            :class="[
              'w-6 h-6 rounded-full p-0! flex items-center justify-center transition-all cursor-pointer border border-transparent shadow-none',
              selectedColor === preset.hex ? 'ring-2 ring-primary ring-offset-2 scale-110' : 'hover:scale-105 opacity-80 hover:opacity-100'
            ]"
            @click="selectColor(preset.hex)"
          >
            <template #icon>
              <Check v-if="selectedColor === preset.hex" :size="12" class="text-white drop-shadow" />
            </template>
          </Button>
          <Button
            v-if="selectedColor"
            variant="text"
            severity="secondary"
            size="small"
            class="text-[11px] h-6 px-2 text-muted-color hover:text-primary"
            @click="selectedColor = null"
          >
            Clear color
          </Button>
        </div>
      </div>

      <!-- Labels & Tags Input -->
      <div class="flex flex-col gap-2">
        <label for="cluster-tag-input" class="text-xs font-semibold text-primary">
          Labels & Tags
        </label>
        <div class="flex items-center gap-2">
          <div class="relative flex-1">
            <TagIcon :size="13" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-color pointer-events-none" />
            <InputText
              id="cluster-tag-input"
              v-model="newTagInput"
              placeholder="Add tag and press Enter (e.g. prod, aws)"
              class="w-full text-xs h-8 pl-8!"
              size="small"
              @keydown="handleTagKeydown"
            />
          </div>
          <Button
            size="small"
            variant="outlined"
            severity="secondary"
            class="h-8 px-3 text-xs"
            :disabled="!newTagInput.trim()"
            @click="addTag"
          >
            Add
          </Button>
        </div>

        <!-- Render Tag Chips -->
        <div v-if="tags.length > 0" class="flex flex-wrap gap-1.5 pt-1">
          <span
            v-for="tag in tags"
            :key="tag"
            class="inline-flex items-center gap-1 pl-2 pr-1 py-0.5 rounded text-[11px] font-medium bg-(--bg-hover) text-primary border border-(--border)"
          >
            <span>#{{ tag }}</span>
            <Button
              variant="text"
              severity="secondary"
              size="small"
              class="w-4! h-4! p-0! text-muted-color hover:text-(--danger) cursor-pointer"
              @click="removeTag(tag)"
            >
              <template #icon>
                <X :size="10" />
              </template>
            </Button>
          </span>
        </div>
      </div>

      <!-- Operational Description / Notes -->
      <div class="flex flex-col gap-1.5">
        <label for="cluster-desc-input" class="text-xs font-semibold text-primary">
          Description & Notes
        </label>
        <Textarea
          id="cluster-desc-input"
          v-model="description"
          placeholder="Optional notes regarding this cluster's purpose or environment..."
          rows="2"
          auto-resize
          class="w-full text-xs"
        />
      </div>
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
