<script setup lang="ts">
import { useWindowControls } from '@/composables/useWindowControls'
import { useSearchStore } from '@/stores/searchStore'
import { Search } from '@lucide/vue'
import Button from 'primevue/button'
import WindowControls from './WindowControls.vue'

const { toggleMaximize } = useWindowControls()
const searchStore = useSearchStore()

const isMac =
  typeof navigator !== 'undefined' &&
  navigator.platform &&
  navigator.platform.toUpperCase().indexOf('MAC') >= 0
</script>

<template>
  <header
    class="relative flex items-center justify-between h-8 w-full shrink-0 select-none bg-(--bg-sidebar) border-b border-(--border) text-xs z-30"
    data-tauri-drag-region
    @dblclick="toggleMaximize"
  >
    <!-- Left: Orbit Logo & Brand -->
    <div class="flex items-center gap-2 px-3 shrink-0 pointer-events-none" data-tauri-drag-region>
      <img src="/logo.png" alt="Orbit" class="w-3.5 h-3.5 object-contain" />
      <span class="text-xs font-semibold tracking-tight text-primary">Orbit</span>
    </div>

    <!-- Center: Search Everywhere Trigger (Centered to Screen/Window) -->
    <div
      class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-40 pointer-events-auto"
    >
      <Button
        type="button"
        severity="secondary"
        variant="outlined"
        size="small"
        class="group flex items-center justify-between gap-2 h-6 w-56 sm:w-64 md:w-80 px-2 text-xs text-muted-color bg-(--bg-hover)/40! dark:bg-(--bg-hover)/30! border-(--border)! hover:bg-(--bg-hover)/80! hover:border-(--border-strong)! hover:text-primary! active:bg-(--bg-active)! rounded-md font-normal cursor-pointer transition-all duration-150"
        v-tooltip.bottom="isMac ? 'Search everywhere (⌘⇧P)' : 'Search everywhere (Ctrl+Shift+P)'"
        @click.stop="searchStore.open()"
        @dblclick.stop
      >
        <div class="flex items-center gap-1.5 min-w-0">
          <Search
            class="w-3 h-3 shrink-0 text-muted-color group-hover:text-primary transition-colors"
          />
          <span
            class="hidden sm:inline truncate text-muted-color group-hover:text-primary transition-colors font-normal text-[11px]"
          >
            Search everywhere...
          </span>
        </div>
        <kbd
          class="hidden md:inline-flex items-center px-1.5 py-0.5 text-[10px] font-mono font-medium leading-none text-muted-color bg-(--bg-card) dark:bg-(--bg-card) border border-(--border) rounded group-hover:border-(--border-strong) group-hover:text-primary transition-colors shrink-0"
        >
          {{ isMac ? '⌘⇧P' : 'Ctrl+Shift+P' }}
        </kbd>
      </Button>
    </div>

    <!-- Center: Window Drag Strip -->
    <div class="flex-1 h-full" data-tauri-drag-region />

    <!-- Right: Window Controls -->
    <WindowControls />
  </header>
</template>
