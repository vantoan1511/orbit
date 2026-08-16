<script setup lang="ts">
import { useLogHighlighting } from '@/composables/useLogHighlighting'
import { Plus, Save, Trash2 } from '@lucide/vue'
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
        class="bg-(--bg-hover)/40 rounded-lg p-3 flex flex-wrap items-center justify-between gap-4"
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
            size="small"
            class="text-xs min-w-48"
            @change="highlighting.saveRules"
          />
          <Button
            v-if="highlighting.isCustomPresetActive.value"
            severity="danger"
            variant="text"
            size="small"
            class="w-8! h-8! p-0! shrink-0"
            v-tooltip="'Delete this custom preset'"
            @click="highlighting.deleteCustomPreset"
          >
            <template #icon>
              <Trash2 class="w-4 h-4" />
            </template>
          </Button>
        </div>

        <div class="flex items-center gap-2">
          <InputText
            v-model="newPresetName"
            placeholder="Preset name..."
            size="small"
            class="text-xs w-40"
          />
          <Button
            label="Save"
            size="small"
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
            size="small"
            class="text-xs flex-1"
            :disabled="rule.isPreset"
            @change="highlighting.saveRules"
          />
          <Select
            v-model="rule.color"
            :options="highlighting.colorOptions"
            optionLabel="label"
            optionValue="value"
            size="small"
            class="text-xs min-w-32"
            :disabled="rule.isPreset"
            @change="highlighting.saveRules"
          />
          <div class="flex items-center gap-1.5 ml-1">
            <Checkbox
              v-model="rule.bold"
              :inputId="'bold-' + rule.id"
              binary
              :disabled="rule.isPreset"
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
              :disabled="rule.isPreset"
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
              :disabled="rule.isPreset"
              @change="highlighting.saveRules"
            />
            <label
              :for="'rx-' + rule.id"
              class="text-xs uppercase font-bold text-muted-color cursor-pointer select-none"
              >Regex</label
            >
          </div>
          <Button
            v-if="!rule.isPreset"
            severity="danger"
            variant="text"
            size="small"
            class="p-1! text-muted-color hover:text-rose-500 cursor-pointer"
            @click="highlighting.deleteCustomRule(rule.id)"
          >
            <template #icon>
              <Trash2 class="w-4 h-4" />
            </template>
          </Button>
        </div>
      </div>
    </div>

    <!-- Dialog Footer -->
    <template #footer>
      <div class="flex justify-between items-center w-full">
        <Button
          label="Add Rule"
          size="small"
          severity="secondary"
          variant="text"
          @click="highlighting.addRule"
        >
          <template #icon>
            <Plus class="w-3.5 h-3.5 mr-1" />
          </template>
        </Button>
        <Button label="Close" size="small" severity="secondary" @click="visible = false" />
      </div>
    </template>
  </Dialog>
</template>
