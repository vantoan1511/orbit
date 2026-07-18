<script setup lang="ts">
import ToggleSwitch from 'primevue/toggleswitch'

const modelValue = defineModel<boolean>({ required: true })
</script>

<template>
  <div class="flex items-center gap-2">
    <ToggleSwitch v-model="modelValue" inputId="system-ns-toggle" />
    <label
      for="system-ns-toggle"
      class="text-xs font-semibold text-(--text-secondary) cursor-pointer select-none"
    >
      Show System
    </label>
  </div>
</template>
