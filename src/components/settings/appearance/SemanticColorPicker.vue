<script setup lang="ts">
import { useAppearance } from '@/composables/useAppearance'
import type { SemanticColorRole, SemanticPresetId } from '@/types/appearance'
import { SEMANTIC_ROLE_OPTIONS } from '@/utils/appearanceConstants'
import { Check } from '@lucide/vue'
import Button from 'primevue/button'
import SemanticPreviewCard from './SemanticPreviewCard.vue'

const { semanticColors, setSemanticPreset, setSemanticColorRole, isDark } = useAppearance()

interface PresetButton {
  id: SemanticPresetId
  label: string
  description: string
}

const presetButtons: readonly PresetButton[] = [
  {
    id: 'default',
    label: 'Default Orbit',
    description: 'Balanced technical noir status palette.'
  },
  {
    id: 'colorblind',
    label: 'Colorblind Friendly',
    description: 'Calibrated for deuteranopia & protanopia discrimination.'
  },
  {
    id: 'high-contrast',
    label: 'High Contrast',
    description: 'Deep, saturated tones for maximum visibility.'
  }
]

const roleLabels: Record<SemanticColorRole, { title: string; subtitle: string }> = {
  success: {
    title: 'Success',
    subtitle: 'Healthy workloads, ready nodes, active namespaces'
  },
  warning: {
    title: 'Warning',
    subtitle: 'Pending pods, progressing states, warning events'
  },
  danger: {
    title: 'Danger',
    subtitle: 'Failed resources, crash loops, critical errors'
  },
  info: {
    title: 'Info',
    subtitle: 'Active port-forwards, informational notices'
  }
}

const getSwatchHex = (role: SemanticColorRole, optionId: string): string => {
  const options = SEMANTIC_ROLE_OPTIONS[role]
  const opt = options.find((o) => o.id === optionId) ?? options[0]!
  return isDark.value ? opt.dark.solid : opt.light.solid
}
</script>

<template>
  <div class="flex flex-col gap-6">
    <!-- Presets bar -->
    <div class="flex flex-col gap-2">
      <div class="flex items-center justify-between">
        <span class="text-xs font-semibold text-primary">Palette Presets</span>
        <span
          v-if="semanticColors.preset === 'custom'"
          class="text-[11px] font-medium text-muted-color px-2 py-0.5 rounded-full bg-(--bg-hover)"
        >
          Customized Palette
        </span>
      </div>

      <div class="flex flex-wrap gap-2">
        <Button
          v-for="preset in presetButtons"
          :key="preset.id"
          v-tooltip.top="preset.description"
          size="small"
          :severity="semanticColors.preset === preset.id ? 'primary' : 'secondary'"
          :variant="semanticColors.preset === preset.id ? undefined : 'outlined'"
          class="text-xs font-medium px-3 py-1.5"
          @click="setSemanticPreset(preset.id)"
        >
          {{ preset.label }}
        </Button>
      </div>
    </div>

    <!-- Live Preview Card -->
    <SemanticPreviewCard />

    <!-- Role Customizer Swatches -->
    <div class="flex flex-col gap-4">
      <div
        v-for="(meta, role) in roleLabels"
        :key="role"
        class="flex flex-col sm:flex-row sm:items-center justify-between gap-2 p-3 rounded-lg bg-(--bg-hover)/20 border border-(--border)"
      >
        <div class="flex flex-col gap-0.5 min-w-[140px]">
          <span class="text-xs font-semibold text-primary">{{ meta.title }}</span>
          <span class="text-[11px] text-muted-color leading-tight">{{ meta.subtitle }}</span>
        </div>

        <!-- Swatches list -->
        <div class="flex items-center gap-2.5 shrink-0">
          <div
            v-for="opt in SEMANTIC_ROLE_OPTIONS[role]"
            :key="opt.id"
            v-tooltip.top="opt.name"
            role="button"
            tabindex="0"
            class="group relative w-8 h-8 rounded-full flex items-center justify-center transition-all cursor-pointer select-none outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-(--bg-panel) focus-visible:ring-(--accent)"
            :class="[
              semanticColors[role] === opt.id
                ? 'ring-2 ring-offset-2 ring-offset-(--bg-panel) ring-(--accent) scale-105'
                : 'hover:scale-105 opacity-80 hover:opacity-100'
            ]"
            @click="setSemanticColorRole(role, opt.id)"
            @keydown.enter.space.prevent="setSemanticColorRole(role, opt.id)"
          >
            <!-- Color circle -->
            <span
              class="w-full h-full rounded-full flex items-center justify-center shadow-xs"
              :style="{ backgroundColor: getSwatchHex(role, opt.id) }"
            >
              <Check
                v-if="semanticColors[role] === opt.id"
                class="w-3.5 h-3.5 text-white stroke-[3] drop-shadow-xs"
              />
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
