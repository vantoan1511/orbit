<script setup lang="ts">
import { useTerminalStore } from '@/stores/terminalStore'
import {
  Box,
  ChevronDown,
  Maximize2,
  Minimize2,
  Plus,
  Terminal as TerminalIcon,
  X
} from '@lucide/vue'
import Button from 'primevue/button'
import { computed, ref } from 'vue'
import TerminalTab from './TerminalTab.vue'

const terminalStore = useTerminalStore()

const panelHeight = ref(280)
const isMaximized = ref(false)
const isDragging = ref(false)
let startY = 0
let startHeight = 280

const displayHeight = computed(() => {
  if (isMaximized.value) return 'calc(100vh - 120px)'
  return `${panelHeight.value}px`
})

function handleMouseDown(e: MouseEvent) {
  isDragging.value = true
  startY = e.clientY
  startHeight = panelHeight.value

  const onMouseMove = (moveEvent: MouseEvent) => {
    if (!isDragging.value) return
    const delta = startY - moveEvent.clientY
    const newHeight = Math.max(140, Math.min(window.innerHeight - 100, startHeight + delta))
    panelHeight.value = newHeight
  }

  const onMouseUp = () => {
    isDragging.value = false
    window.removeEventListener('mousemove', onMouseMove)
    window.removeEventListener('mouseup', onMouseUp)
  }

  window.addEventListener('mousemove', onMouseMove)
  window.addEventListener('mouseup', onMouseUp)
}

function handleNewLocalTerminal() {
  const id = `term-local-${Date.now()}`
  terminalStore.addSession({
    id,
    title: `Local Shell ${terminalStore.sessions.filter((s) => s.type === 'local').length + 1}`,
    type: 'local'
  })
}

function handleToggleMaximize() {
  isMaximized.value = !isMaximized.value
}
</script>

<template>
  <div
    v-if="terminalStore.isOpen"
    class="flex flex-col bg-(--bg-app) border-t border-(--border) relative shrink-0 z-30 transition-all select-none"
    :style="{ height: displayHeight }"
  >
    <!-- Resize Handle -->
    <div
      class="h-1 w-full cursor-row-resize hover:bg-(--accent) transition-colors absolute top-0 left-0 z-40"
      @mousedown="handleMouseDown"
    />

    <!-- Terminal Header Bar -->
    <div
      class="h-8 min-h-8 flex items-center justify-between px-2 bg-(--bg-sidebar) border-b border-(--border) text-xs"
    >
      <!-- Left side: Session Tabs -->
      <div class="flex items-center gap-1 overflow-x-auto flex-1 no-scrollbar mr-2">
        <div class="flex items-center gap-1.5 px-2 text-muted-color font-medium shrink-0">
          <TerminalIcon class="w-3.5 h-3.5 text-primary" />
          <span class="text-[11px] uppercase tracking-wider font-semibold text-primary"
            >Terminal</span
          >
        </div>

        <div class="h-3.5 w-px bg-(--border) mx-1 shrink-0" />

        <!-- Tabs list -->
        <div
          v-for="session in terminalStore.sessions"
          :key="session.id"
          class="flex items-center gap-1.5 px-2 py-1 rounded text-xs cursor-pointer border transition-colors shrink-0"
          :class="[
            session.id === terminalStore.activeSessionId
              ? 'bg-(--bg-app) border-(--border) text-primary font-medium'
              : 'border-transparent text-muted-color hover:text-primary hover:bg-(--surface-hover)'
          ]"
          @click="terminalStore.setActiveSession(session.id)"
        >
          <Box v-if="session.type === 'pod'" class="w-3 h-3 text-(--accent)" />
          <TerminalIcon v-else class="w-3 h-3 text-muted-color" />
          <span class="max-w-36 truncate">{{ session.title }}</span>
          <Button
            rounded
            variant="text"
            size="small"
            class="w-4! h-4! p-0! text-muted-color hover:text-primary"
            v-tooltip.top="'Close Session'"
            @click.stop="terminalStore.removeSession(session.id)"
          >
            <template #icon>
              <X :size="10" />
            </template>
          </Button>
        </div>

        <!-- Add Local Terminal Button -->
        <Button
          rounded
          variant="text"
          size="small"
          class="w-5! h-5! p-0! text-muted-color hover:text-primary shrink-0"
          v-tooltip.top="'New Local Terminal'"
          @click="handleNewLocalTerminal"
        >
          <template #icon>
            <Plus :size="12" />
          </template>
        </Button>
      </div>

      <!-- Right controls: Maximize, Minimize, Close -->
      <div class="flex items-center gap-0.5 shrink-0">
        <Button
          rounded
          variant="text"
          size="small"
          class="w-6! h-6! p-0! text-muted-color hover:text-primary"
          v-tooltip.top="isMaximized ? 'Restore Terminal' : 'Maximize Terminal'"
          @click="handleToggleMaximize"
        >
          <template #icon>
            <Minimize2 v-if="isMaximized" :size="12" />
            <Maximize2 v-else :size="12" />
          </template>
        </Button>
        <Button
          rounded
          variant="text"
          size="small"
          class="w-6! h-6! p-0! text-muted-color hover:text-primary"
          v-tooltip.top="'Minimize Panel'"
          @click="terminalStore.closePanel()"
        >
          <template #icon>
            <ChevronDown :size="12" />
          </template>
        </Button>
        <Button
          rounded
          variant="text"
          size="small"
          class="w-6! h-6! p-0! text-muted-color hover:text-primary"
          v-tooltip.top="'Close Terminal'"
          @click="terminalStore.closePanel()"
        >
          <template #icon>
            <X :size="12" />
          </template>
        </Button>
      </div>
    </div>

    <!-- Terminal Content Area -->
    <div class="flex-1 w-full h-full relative overflow-hidden bg-(--bg-app)">
      <template v-if="terminalStore.sessions.length > 0">
        <TerminalTab
          v-for="session in terminalStore.sessions"
          :key="session.id"
          :session="session"
          :is-active="session.id === terminalStore.activeSessionId"
          v-show="session.id === terminalStore.activeSessionId"
        />
      </template>

      <!-- Empty Sessions State -->
      <div
        v-else
        class="flex flex-col items-center justify-center h-full text-muted-color text-xs gap-3 p-4"
      >
        <TerminalIcon class="w-8 h-8 opacity-40 text-muted-color" />
        <span>No active terminal sessions</span>
        <Button
          size="small"
          variant="outlined"
          class="text-xs flex items-center gap-1.5"
          @click="handleNewLocalTerminal"
        >
          <Plus class="w-3.5 h-3.5" />
          <span>New Local Shell</span>
        </Button>
      </div>
    </div>
  </div>
</template>
