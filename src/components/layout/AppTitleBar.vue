<script setup lang="ts">
import { useWindowControls } from '@/composables/useWindowControls'
import WindowControls from './WindowControls.vue'

const { toggleMaximize } = useWindowControls()
</script>

<template>
  <header
    class="flex items-center justify-between h-8 w-full shrink-0 select-none bg-(--bg-sidebar) border-b border-(--border) text-xs z-30"
    data-tauri-drag-region
    @dblclick="toggleMaximize"
  >
    <!-- Left: Orbit Logo & Brand -->
    <div class="flex items-center gap-2 px-3 shrink-0 pointer-events-none" data-tauri-drag-region>
      <img src="/logo.png" alt="Orbit" class="w-3.5 h-3.5 object-contain" />
      <span class="text-xs font-semibold tracking-tight text-primary">Orbit</span>
    </div>

    <!-- Center: Window Drag Strip -->
    <div class="flex-1 h-full" data-tauri-drag-region />

    <!-- Right: Window Controls -->
    <WindowControls />
  </header>
</template>
