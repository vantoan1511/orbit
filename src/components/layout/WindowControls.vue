<script setup lang="ts">
import { useWindowControls } from '@/composables/useWindowControls'
import {
  getMaximizeAriaLabel,
  getMaximizeTooltip,
  getWindowControlButtonClass
} from '@/utils/windowControlStyles'
import { Copy, Minus, Square, X } from '@lucide/vue'
import Button from 'primevue/button'

const { isMaximized, minimize, toggleMaximize, close } = useWindowControls()
</script>

<template>
  <div
    data-component="window-controls"
    class="flex items-center select-none shrink-0 z-30"
    role="toolbar"
    aria-label="Window controls"
  >
    <!-- Minimize Button -->
    <Button
      type="button"
      unstyled
      :class="getWindowControlButtonClass('minimize')"
      aria-label="Minimize window"
      v-tooltip.bottom="'Minimize'"
      @click="minimize"
    >
      <Minus class="w-3.5 h-3.5" />
    </Button>

    <!-- Maximize / Restore Button -->
    <Button
      type="button"
      unstyled
      :class="getWindowControlButtonClass('maximize')"
      :aria-label="getMaximizeAriaLabel(isMaximized)"
      v-tooltip.bottom="getMaximizeTooltip(isMaximized)"
      @click="toggleMaximize"
    >
      <Copy v-if="isMaximized" class="w-3.5 h-3.5" />
      <Square v-else class="w-3 h-3" />
    </Button>

    <!-- Close Button -->
    <Button
      type="button"
      unstyled
      :class="getWindowControlButtonClass('close')"
      aria-label="Close window"
      v-tooltip.bottom="'Close'"
      @click="close"
    >
      <X class="w-3.5 h-3.5" />
    </Button>
  </div>
</template>
