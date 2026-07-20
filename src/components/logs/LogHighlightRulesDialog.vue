<script setup lang="ts">
import { useLogHighlighting } from '@/composables/useLogHighlighting'
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
    class="bg-(--bg-card) border border-(--border)"
    :style="{ width: '900px', maxWidth: '90vw' }"
  >
    <div class="flex flex-col gap-4">
      <p class="text-xs text-(--text-muted)">
        Define search patterns to style log lines dynamically. Presets are read-only; custom rules
        can be edited/deleted.
      </p>
      <div
        class="flex flex-wrap items-center justify-between gap-4 bg-(--bg-hover)/10 p-3 border border-(--border) rounded-lg"
      >
        <div class="flex items-center gap-3">
          <label class="text-xs font-semibold text-(--text-secondary)">Rule Preset:</label>
          <Select
            v-model="selectedPreset"
            :options="highlighting.presetOptions.value"
            optionLabel="label"
            optionValue="value"
            optionGroupLabel="label"
            optionGroupChildren="items"
            class="text-xs min-w-48 bg-(--bg-card) border-(--border)"
            @change="highlighting.saveRules"
          />
          <Button
            v-if="highlighting.isCustomPresetActive.value"
            icon="pi pi-trash"
            severity="danger"
            variant="text"
            size="small"
            v-tooltip="'Delete this custom preset'"
            @click="highlighting.deleteCustomPreset"
          />
        </div>
        <div class="flex items-center gap-2">
          <InputText
            v-model="newPresetName"
            placeholder="Preset name..."
            class="text-xs bg-(--bg-card) border-(--border) w-40"
          />
          <Button
            label="Save Custom Rules as Preset"
            icon="pi pi-save"
            size="small"
            severity="secondary"
            :disabled="
              !highlighting.customRules.value.length || !highlighting.newPresetName.value.trim()
            "
            @click="highlighting.saveCustomPreset"
          />
        </div>
      </div>
      <div class="flex flex-col gap-2 max-h-80 overflow-y-auto pr-1">
        <div
          v-for="rule in highlighting.activeRules.value"
          :key="rule.id"
          class="flex items-center gap-2 p-2 border rounded-lg"
          :class="
            rule.isPreset
              ? 'bg-(--bg-hover)/5 border-dashed border-(--border-muted) opacity-80'
              : 'bg-(--bg-hover)/20 border-(--border)'
          "
        >
          <InputText
            v-model="rule.pattern"
            placeholder="Pattern..."
            class="text-xs bg-(--bg-card) border-(--border) flex-1"
            :disabled="rule.isPreset"
            @change="highlighting.saveRules"
          />
          <Select
            v-model="rule.color"
            :options="highlighting.colorOptions"
            optionLabel="label"
            optionValue="value"
            class="text-xs min-w-28 bg-(--bg-card) border-(--border)"
            :disabled="rule.isPreset"
            @change="highlighting.saveRules"
          />
          <div class="flex items-center gap-1.5 ml-1">
            <Checkbox
              v-model="rule.bold"
              :inputId="'bold-' + rule.id"
              binary
              class="border-(--border)"
              :disabled="rule.isPreset"
              @change="highlighting.saveRules"
            />
            <label
              :for="'bold-' + rule.id"
              class="text-[10px] uppercase font-bold text-(--text-muted) cursor-pointer select-none"
              >Bold</label
            >
          </div>
          <div class="flex items-center gap-1.5 ml-1">
            <Checkbox
              v-model="rule.caseSensitive"
              :inputId="'cs-' + rule.id"
              binary
              class="border-(--border)"
              :disabled="rule.isPreset"
              @change="highlighting.saveRules"
            />
            <label
              :for="'cs-' + rule.id"
              class="text-[10px] uppercase font-bold text-(--text-muted) cursor-pointer select-none"
              >CS</label
            >
          </div>
          <div class="flex items-center gap-1.5 ml-1">
            <Checkbox
              v-model="rule.isRegex"
              :inputId="'rx-' + rule.id"
              binary
              class="border-(--border)"
              :disabled="rule.isPreset"
              @change="highlighting.saveRules"
            />
            <label
              :for="'rx-' + rule.id"
              class="text-[10px] uppercase font-bold text-(--text-muted) cursor-pointer select-none"
              >Regex</label
            >
          </div>
          <Button
            v-if="!rule.isPreset"
            icon="pi pi-trash"
            severity="danger"
            variant="text"
            size="small"
            @click="highlighting.deleteCustomRule(rule.id)"
          />
          <span v-else class="text-[10px] uppercase font-bold text-(--text-muted) px-2"
            >Preset</span
          >
        </div>
      </div>
      <div class="flex justify-between items-center mt-2">
        <Button
          label="Add Rule"
          icon="pi pi-plus"
          size="small"
          severity="secondary"
          variant="text"
          @click="highlighting.addRule"
        />
        <Button label="Close" size="small" variant="text" @click="visible = false" />
      </div>
    </div>
  </Dialog>
</template>
