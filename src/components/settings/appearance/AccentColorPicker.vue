<script setup lang="ts">
import { useAppearance } from '@/composables/useAppearance'
import type { AccentColor } from '@/types/appearance'
import { ACCENT_COLOR_DEFINITIONS } from '@/utils/appearanceConstants'
import { Check } from '@lucide/vue'

const { accentColor, setAccentColor, isDark } = useAppearance()

const getSwatchHex = (colorId: AccentColor): string => {
  const def = ACCENT_COLOR_DEFINITIONS.find((c) => c.id === colorId)
  if (!def) return '#4f8cff'
  return isDark.value ? def.dark.accent : def.light.accent
}
</script>

<template>
  <div data-component="accent-color-picker" class="flex flex-wrap items-center gap-3">
    <div
      v-for="color in ACCENT_COLOR_DEFINITIONS"
      :key="color.id"
      v-tooltip.top="color.name"
      role="button"
      tabindex="0"
      class="group relative w-9 h-9 rounded-full flex items-center justify-center transition-all cursor-pointer select-none outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-(--bg-panel) focus-visible:ring-(--accent)"
      :class="[
        accentColor === color.id
          ? 'ring-2 ring-offset-2 ring-offset-(--bg-panel) ring-(--accent) scale-110'
          : 'hover:scale-105 opacity-85 hover:opacity-100'
      ]"
      @click="setAccentColor(color.id)"
      @keydown.enter.space.prevent="setAccentColor(color.id)"
    >
      <!-- Color circle -->
      <span
        class="w-full h-full rounded-full flex items-center justify-center shadow-xs"
        :style="{ backgroundColor: getSwatchHex(color.id) }"
      >
        <Check
          v-if="accentColor === color.id"
          class="w-4 h-4 text-white stroke-[3] drop-shadow-xs"
        />
      </span>
    </div>
  </div>
</template>
