<script setup lang="ts">
import { useAppearance } from '@/composables/useAppearance'
import {
  DEFAULT_APPEARANCE_PREFERENCES,
  DEFAULT_SEMANTIC_COLORS_PREFERENCES
} from '@/types/appearance'
import { RotateCcw } from '@lucide/vue'
import Button from 'primevue/button'
import { useToast } from 'primevue/usetoast'
import AccentColorPicker from './appearance/AccentColorPicker.vue'
import SemanticColorPicker from './appearance/SemanticColorPicker.vue'
import ThemeModeSelector from './appearance/ThemeModeSelector.vue'
import UiStyleSelector from './appearance/UiStyleSelector.vue'

const { preferences, setThemeMode, setAccentColor, setUiStyle, setSemanticColors } = useAppearance()
const toast = useToast()

const isModified = () => {
  const currentSemantic = preferences.value.semanticColors ?? DEFAULT_SEMANTIC_COLORS_PREFERENCES
  const defaultSemantic = DEFAULT_SEMANTIC_COLORS_PREFERENCES
  const semanticModified =
    currentSemantic.preset !== defaultSemantic.preset ||
    currentSemantic.success !== defaultSemantic.success ||
    currentSemantic.warning !== defaultSemantic.warning ||
    currentSemantic.danger !== defaultSemantic.danger ||
    currentSemantic.info !== defaultSemantic.info

  return (
    preferences.value.themeMode !== DEFAULT_APPEARANCE_PREFERENCES.themeMode ||
    preferences.value.accentColor !== DEFAULT_APPEARANCE_PREFERENCES.accentColor ||
    preferences.value.uiStyle !== DEFAULT_APPEARANCE_PREFERENCES.uiStyle ||
    semanticModified
  )
}

const handleReset = async () => {
  await setThemeMode(DEFAULT_APPEARANCE_PREFERENCES.themeMode)
  await setAccentColor(DEFAULT_APPEARANCE_PREFERENCES.accentColor)
  await setUiStyle(DEFAULT_APPEARANCE_PREFERENCES.uiStyle)
  await setSemanticColors({ ...DEFAULT_SEMANTIC_COLORS_PREFERENCES })
  toast.add({
    severity: 'success',
    summary: 'Appearance Reset',
    detail: 'Appearance settings have been reset to defaults.',
    life: 2000
  })
}
</script>

<template>
  <div data-component="settings-appearance-tab" class="flex flex-col gap-10">
    <!-- Header row with Title and Reset Action -->
    <div class="flex items-start justify-between gap-4">
      <div>
        <h3 class="text-base font-semibold text-primary">Appearance</h3>
        <p class="text-xs text-muted-color mt-1">
          Customize color themes, accent colors, and desktop visual aesthetics.
        </p>
      </div>

      <Button
        v-tooltip.top="'Reset to defaults'"
        severity="secondary"
        variant="text"
        size="small"
        :disabled="!isModified()"
        class="flex items-center gap-1.5 text-xs text-muted-color hover:text-primary shrink-0"
        @click="handleReset"
      >
        <RotateCcw class="w-3.5 h-3.5" />
        <span>Reset</span>
      </Button>
    </div>

    <!-- Section 1: Theme Mode -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <h4 class="text-xs font-semibold text-primary uppercase tracking-wider">Theme Mode</h4>
        <p class="text-[11px] text-muted-color">
          Choose between dark, light, or automatic system theme synchronization.
        </p>
      </div>
      <div class="w-full md:w-2/3 xl:w-3/4">
        <ThemeModeSelector />
      </div>
    </div>

    <!-- Section 2: UI Style -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <h4 class="text-xs font-semibold text-primary uppercase tracking-wider">
          UI Style & Aesthetic
        </h4>
        <p class="text-[11px] text-muted-color">
          Choose desktop surface materials ranging from classic Nora flat to modern glass and
          Cupertino acrylic.
        </p>
      </div>
      <div class="w-full md:w-2/3 xl:w-3/4">
        <UiStyleSelector />
      </div>
    </div>

    <!-- Section 3: Accent Color -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <h4 class="text-xs font-semibold text-primary uppercase tracking-wider">Accent Color</h4>
        <p class="text-[11px] text-muted-color">
          Select your preferred accent hue for active controls, badges, and focus rings.
        </p>
      </div>
      <div class="w-full md:w-2/3 xl:w-3/4">
        <AccentColorPicker />
      </div>
    </div>

    <!-- Section 4: Semantic Colors -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <h4 class="text-xs font-semibold text-primary uppercase tracking-wider">Semantic Colors</h4>
        <p class="text-[11px] text-muted-color">
          Customize status indicators, health states, and alert levels across all clusters and
          resources.
        </p>
      </div>
      <div class="w-full md:w-2/3 xl:w-3/4">
        <SemanticColorPicker />
      </div>
    </div>
  </div>
</template>
