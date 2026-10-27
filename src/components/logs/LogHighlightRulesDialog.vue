<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import type { HighlightRule, useLogHighlighting } from '@/composables/useLogHighlighting'
import { Pencil, Plus, Save, Trash2 } from '@lucide/vue'
import Button from 'primevue/button'
import Checkbox from 'primevue/checkbox'
import Dialog from 'primevue/dialog'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'

const props = defineProps<{
  highlighting: ReturnType<typeof useLogHighlighting>
}>()

const visible = defineModel<boolean>('visible', { default: false })

// Avoid unexpected mutation of prop errors by accessing refs directly
const selectedPreset = props.highlighting.selectedPreset
const newPresetName = props.highlighting.newPresetName

// State for child Edit Highlight Rule dialog
const isEditModalOpen = ref(false)
const editingRuleId = ref<string | null>(null)
const editForm = reactive({
  pattern: '',
  color: 'rose',
  bold: false,
  caseSensitive: false,
  isRegex: false
})

const editError = computed(() => {
  if (!editForm.pattern.trim()) {
    return 'Pattern cannot be empty'
  }
  const validation = props.highlighting.validateRule(
    editForm.pattern,
    editForm.isRegex,
    editForm.caseSensitive
  )
  return validation.valid ? null : (validation.error ?? 'Invalid regular expression')
})

const openEditModal = (rule: HighlightRule) => {
  editingRuleId.value = rule.id
  editForm.pattern = rule.pattern
  editForm.color = rule.color
  editForm.bold = rule.bold
  editForm.caseSensitive = rule.caseSensitive
  editForm.isRegex = rule.isRegex
  isEditModalOpen.value = true
}

const handleSaveEdit = () => {
  if (!editingRuleId.value || editError.value) return
  const success = props.highlighting.updateCustomRule(editingRuleId.value, {
    pattern: editForm.pattern.trim(),
    color: editForm.color,
    bold: editForm.bold,
    caseSensitive: editForm.caseSensitive,
    isRegex: editForm.isRegex
  })
  if (success) {
    isEditModalOpen.value = false
    editingRuleId.value = null
  }
}
</script>

<template>
  <Dialog
    v-model:visible="visible"
    modal
    header="Highlight Rules"
    :style="{ width: '820px', maxWidth: '92vw' }"
  >
    <div class="flex flex-col gap-4">
      <p class="text-xs text-muted-color">
        Define search patterns to style log lines dynamically. Presets are read-only; custom rules
        can be edited and deleted.
      </p>

      <!-- Preset Bar (borderless well) -->
      <div
        class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-card)] p-3 flex flex-wrap items-center justify-between gap-4"
      >
        <div class="flex items-center gap-3">
          <label class="text-xs font-semibold text-muted-color">Rule Preset:</label>
          <Select
            v-model="selectedPreset"
            :options="highlighting.presetOptions.value"
            optionLabel="label"
            optionValue="value"
            optionGroupLabel="label"
            optionGroupChildren="items"
            class="text-xs min-w-48"
            @change="highlighting.saveRules"
          />
          <Button
            v-if="highlighting.isCustomPresetActive.value"
            severity="danger"
            variant="text"
            class="shrink-0"
            v-tooltip="'Delete this custom preset'"
            @click="highlighting.deleteCustomPreset"
          >
            <template #icon>
              <Trash2 class="w-4 h-4" />
            </template>
          </Button>
        </div>

        <div class="flex items-center gap-2">
          <InputText v-model="newPresetName" placeholder="Preset name..." class="text-xs w-40" />
          <Button
            label="Save"
            severity="secondary"
            :disabled="
              !highlighting.customRules.value.length || !highlighting.newPresetName.value.trim()
            "
            @click="highlighting.saveCustomPreset"
          >
            <template #icon>
              <Save class="w-3.5 h-3.5 mr-1" />
            </template>
          </Button>
        </div>
      </div>

      <!-- Rules List -->
      <div class="flex flex-col gap-2 max-h-80 overflow-y-auto pr-1">
        <div
          v-for="rule in highlighting.activeRules.value"
          :key="rule.id"
          class="flex items-center gap-2 p-1.5 rounded bg-(--bg-hover)/20"
        >
          <InputText
            v-model="rule.pattern"
            placeholder="Pattern..."
            class="text-xs flex-1"
            :disabled="highlighting.isBuiltInRule(rule)"
            @change="highlighting.saveRules"
          />
          <Select
            v-model="rule.color"
            :options="highlighting.colorOptions"
            optionLabel="label"
            optionValue="value"
            class="text-xs min-w-32"
            :disabled="highlighting.isBuiltInRule(rule)"
            @change="highlighting.saveRules"
          />
          <div class="flex items-center gap-1.5 ml-1">
            <Checkbox
              v-model="rule.bold"
              :inputId="'bold-' + rule.id"
              binary
              :disabled="highlighting.isBuiltInRule(rule)"
              @change="highlighting.saveRules"
            />
            <label
              :for="'bold-' + rule.id"
              class="text-xs uppercase font-bold text-muted-color cursor-pointer select-none"
              >Bold</label
            >
          </div>
          <div class="flex items-center gap-1.5 ml-1">
            <Checkbox
              v-model="rule.caseSensitive"
              :inputId="'cs-' + rule.id"
              binary
              :disabled="highlighting.isBuiltInRule(rule)"
              @change="highlighting.saveRules"
            />
            <label
              :for="'cs-' + rule.id"
              class="text-xs uppercase font-bold text-muted-color cursor-pointer select-none"
              >CS</label
            >
          </div>
          <div class="flex items-center gap-1.5 ml-1">
            <Checkbox
              v-model="rule.isRegex"
              :inputId="'rx-' + rule.id"
              binary
              :disabled="highlighting.isBuiltInRule(rule)"
              @change="highlighting.saveRules"
            />
            <label
              :for="'rx-' + rule.id"
              class="text-xs uppercase font-bold text-muted-color cursor-pointer select-none"
              >Regex</label
            >
          </div>
          <template v-if="!highlighting.isBuiltInRule(rule)">
            <Button
              severity="secondary"
              variant="text"
              class="p-1 text-muted-color hover:text-primary cursor-pointer"
              v-tooltip.top="'Edit rule'"
              @click="openEditModal(rule)"
            >
              <template #icon>
                <Pencil class="w-4 h-4" />
              </template>
            </Button>
            <Button
              severity="danger"
              variant="text"
              class="p-1 text-muted-color hover:text-rose-500 cursor-pointer"
              v-tooltip.top="'Delete rule'"
              @click="highlighting.deleteCustomRule(rule.id)"
            >
              <template #icon>
                <Trash2 class="w-4 h-4" />
              </template>
            </Button>
          </template>
        </div>
      </div>
    </div>

    <!-- Edit Rule Modal -->
    <Dialog
      v-model:visible="isEditModalOpen"
      modal
      header="Edit Highlight Rule"
      :style="{ width: '480px', maxWidth: '92vw' }"
    >
      <div class="flex flex-col gap-4 py-2">
        <!-- Pattern Input -->
        <div class="flex flex-col gap-1.5">
          <label for="edit-pattern" class="text-xs font-semibold text-muted-color">Pattern</label>
          <InputText
            id="edit-pattern"
            v-model="editForm.pattern"
            placeholder="e.g. error, timeout, \b(404|500)\b"
            class="text-xs w-full"
            :invalid="!!editError"
          />
          <span v-if="editError" class="text-xs text-rose-500">{{ editError }}</span>
        </div>

        <!-- Color Select -->
        <div class="flex flex-col gap-1.5">
          <label for="edit-color" class="text-xs font-semibold text-muted-color">Color</label>
          <Select
            id="edit-color"
            v-model="editForm.color"
            :options="highlighting.colorOptions"
            optionLabel="label"
            optionValue="value"
            class="text-xs w-full"
          />
        </div>

        <!-- Flags / Options -->
        <div class="flex items-center gap-4 pt-1">
          <div class="flex items-center gap-1.5">
            <Checkbox v-model="editForm.bold" inputId="edit-bold" binary />
            <label
              for="edit-bold"
              class="text-xs font-medium text-muted-color cursor-pointer select-none"
            >
              Bold
            </label>
          </div>
          <div class="flex items-center gap-1.5">
            <Checkbox v-model="editForm.caseSensitive" inputId="edit-cs" binary />
            <label
              for="edit-cs"
              class="text-xs font-medium text-muted-color cursor-pointer select-none"
            >
              Case Sensitive
            </label>
          </div>
          <div class="flex items-center gap-1.5">
            <Checkbox v-model="editForm.isRegex" inputId="edit-rx" binary />
            <label
              for="edit-rx"
              class="text-xs font-medium text-muted-color cursor-pointer select-none"
            >
              Regex
            </label>
          </div>
        </div>
      </div>

      <template #footer>
        <div class="flex justify-end items-center gap-2 w-full">
          <Button
            label="Cancel"
            severity="secondary"
            variant="text"
            @click="isEditModalOpen = false"
          />
          <Button label="Save Changes" :disabled="!!editError" @click="handleSaveEdit" />
        </div>
      </template>
    </Dialog>

    <!-- Dialog Footer -->
    <template #footer>
      <div class="flex justify-between items-center w-full">
        <Button label="Add Rule" severity="secondary" variant="text" @click="highlighting.addRule">
          <template #icon>
            <Plus class="w-3.5 h-3.5 mr-1" />
          </template>
        </Button>
        <Button label="Close" severity="secondary" @click="visible = false" />
      </div>
    </template>
  </Dialog>
</template>
