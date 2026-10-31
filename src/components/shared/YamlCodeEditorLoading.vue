<script setup lang="ts">
import { Loader2 } from '@lucide/vue'
</script>

<template>
  <div
    data-component="yaml-code-editor-loading"
    class="w-full h-full flex items-center justify-center gap-2 rounded-[var(--ui-radius-card)] border border-(--border) bg-(--bg-card)"
  >
    <Loader2 class="w-6 h-6 text-muted-color animate-spin" />
    <span class="text-xs text-muted-color">Loading YAML editor...</span>
  </div>
</template>
