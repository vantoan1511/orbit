<script setup lang="ts">
import { COLOR_PRESETS } from '@/types/cluster'
import { Check, Tag as TagIcon, X } from '@lucide/vue'
import { Button, InputText, Textarea } from 'primevue'
import { ref } from 'vue'

const props = withDefaults(
  defineProps<{
    alias: string
    color: string | null
    tags: string[]
    description?: string
    showDescription?: boolean
  }>(),
  {
    description: '',
    showDescription: false
  }
)

const emit = defineEmits<{
  (e: 'update:alias', val: string): void
  (e: 'update:color', val: string | null): void
  (e: 'update:tags', val: string[]): void
  (e: 'update:description', val: string): void
}>()

const newTagInput = ref('')

function updateAlias(val: string | undefined): void {
  emit('update:alias', val ?? '')
}

function selectColor(hex: string): void {
  if (props.color === hex) {
    emit('update:color', null)
  } else {
    emit('update:color', hex)
  }
}

function clearColor(): void {
  emit('update:color', null)
}

function addTag(): void {
  const raw = newTagInput.value.trim().toLowerCase()
  if (!raw) return

  const tokens = raw
    .split(',')
    .map((t) => t.trim().replace(/^#+/, ''))
    .filter(Boolean)

  const updated = [...props.tags]
  for (const token of tokens) {
    if (!updated.includes(token)) {
      updated.push(token)
    }
  }

  emit('update:tags', updated)
  newTagInput.value = ''
}

function handleTagKeydown(e: KeyboardEvent): void {
  if (e.key === 'Enter' || e.key === ',') {
    e.preventDefault()
    addTag()
  }
}

function removeTag(tagToRemove: string): void {
  emit(
    'update:tags',
    props.tags.filter((t) => t !== tagToRemove)
  )
}
</script>

<template>
  <div class="flex flex-col gap-4 text-xs" data-component="cluster-customization-fields">
    <!-- Friendly Alias -->
    <div class="flex flex-col gap-1.5">
      <label class="text-xs font-medium text-muted-color">Display Name / Alias</label>
      <InputText
        :model-value="alias"
        placeholder="e.g. Staging US-West"
        size="small"
        class="w-full text-xs rounded-[var(--ui-radius-control)]"
        @update:model-value="updateAlias"
      />
      <span class="text-[11px] text-muted-color">
        Custom alias displayed across the sidebar, titlebar, and workspace tabs.
      </span>
    </div>

    <!-- Color Preset Swatches -->
    <div class="flex flex-col gap-1.5">
      <label class="text-xs font-medium text-muted-color">Environment Color</label>
      <div class="flex items-center gap-2 flex-wrap">
        <Button
          v-for="preset in COLOR_PRESETS"
          :key="preset.hex"
          v-tooltip.top="preset.label"
          type="button"
          :style="{ backgroundColor: preset.hex }"
          :class="[
            'w-6 h-6 rounded-full p-0 flex items-center justify-center transition-all cursor-pointer border border-transparent shadow-none',
            color === preset.hex
              ? 'ring-2 ring-primary ring-offset-2 scale-110'
              : 'hover:scale-105 opacity-80 hover:opacity-100'
          ]"
          @click="selectColor(preset.hex)"
        >
          <template #icon>
            <Check v-if="color === preset.hex" :size="12" class="text-white drop-shadow" />
          </template>
        </Button>
        <Button
          v-if="color"
          v-tooltip.top="'Clear Color'"
          size="small"
          variant="text"
          severity="secondary"
          class="p-0 text-[11px] text-muted-color hover:text-(--text-primary) ml-1 h-6 rounded-[var(--ui-radius-control)]"
          @click="clearColor"
        >
          Clear
        </Button>
      </div>
    </div>

    <!-- Tags -->
    <div class="flex flex-col gap-1.5">
      <label class="text-xs font-medium text-muted-color">Tags</label>
      <div
        class="flex flex-wrap items-center gap-1.5 min-h-7 p-1.5 border border-(--border) rounded-[var(--ui-radius-control)] bg-(--bg-app)/60"
      >
        <span
          v-for="t in tags"
          :key="t"
          class="px-2 py-0.5 rounded-[var(--ui-radius-control)] bg-(--bg-hover) text-(--text-primary) text-[11px] flex items-center gap-1 border border-(--border)"
        >
          <TagIcon class="w-2.5 h-2.5 text-muted-color" />
          <span>{{ t }}</span>
          <Button
            type="button"
            variant="text"
            severity="secondary"
            size="small"
            class="w-3.5 h-3.5 p-0 text-muted-color hover:text-(--danger) cursor-pointer flex items-center justify-center border-none shadow-none bg-transparent"
            @click="removeTag(t)"
          >
            <template #icon>
              <X class="w-3 h-3" />
            </template>
          </Button>
        </span>
        <InputText
          v-model="newTagInput"
          placeholder="+ Add tag (Enter)..."
          size="small"
          class="flex-1 min-w-24 text-[11px] p-0 border-none bg-transparent shadow-none focus:outline-none"
          @keydown="handleTagKeydown"
          @blur="addTag"
        />
      </div>
    </div>

    <!-- Optional Description -->
    <div v-if="showDescription" class="flex flex-col gap-1.5">
      <label class="text-xs font-medium text-muted-color">Description</label>
      <Textarea
        :model-value="description"
        placeholder="Optional notes or details regarding this cluster..."
        rows="2"
        auto-resize
        class="w-full text-xs rounded-[var(--ui-radius-control)]"
        @update:model-value="emit('update:description', $event ?? '')"
      />
    </div>
  </div>
</template>
