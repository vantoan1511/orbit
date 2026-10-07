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
      class="h-8 min-h-8 flex items-center justify-between bg-(--bg-sidebar) border-b border-(--border) text-xs"
    >
      <!-- Left side: Session Tabs -->
      <div class="flex items-center h-full overflow-x-auto flex-1 no-scrollbar">
        <div
          class="flex items-center gap-1.5 px-3 h-full text-muted-color font-medium shrink-0 border-r border-(--border)"
        >
          <TerminalIcon class="w-3.5 h-3.5 text-primary" />
          <span class="text-[11px] uppercase tracking-wider font-semibold text-primary"
            >Terminal</span
          >
        </div>

        <!-- Tabs list -->
        <div
          v-for="session in terminalStore.sessions"
          :key="session.id"
          class="group relative flex items-center gap-2 h-8 px-3 border-r border-(--border) text-xs cursor-pointer select-none transition-all duration-150 shrink-0 max-w-48 font-mono"
          :class="[
            session.id === terminalStore.activeSessionId
              ? 'bg-(--bg-app) text-primary font-medium border-t border-t-transparent border-b border-b-(--bg-app)'
              : 'bg-(--bg-sidebar)/70 text-muted-color hover:bg-(--bg-hover)/60 hover:text-primary border-t border-t-transparent border-b border-b-(--border)'
          ]"
          @click="terminalStore.setActiveSession(session.id)"
        >
          <Box v-if="session.type === 'pod'" class="w-3.5 h-3.5 text-(--accent) shrink-0" />
          <TerminalIcon v-else class="w-3.5 h-3.5 text-muted-color shrink-0" />
          <span class="truncate flex-1">{{ session.title }}</span>
          <Button
            variant="text"
            size="small"
            class="w-4! h-4! p-0! shrink-0 rounded-none text-muted-color hover:text-primary hover:bg-(--bg-hover)! transition-opacity"
            :class="
              session.id === terminalStore.activeSessionId
                ? 'opacity-100'
                : 'opacity-0 group-hover:opacity-100'
            "
            v-tooltip.top="'Close Session'"
            @click.stop="terminalStore.removeSession(session.id)"
          >
            <template #icon>
              <X :size="11" />
            </template>
          </Button>
        </div>

        <!-- Add Local Terminal Button -->
        <Button
          variant="text"
          size="small"
          class="w-7! h-7! p-0! rounded-none text-muted-color hover:text-primary hover:bg-(--bg-hover) shrink-0 ml-1"
          v-tooltip.top="'New Local Terminal'"
          @click="handleNewLocalTerminal"
        >
          <template #icon>
            <Plus :size="13" />
          </template>
        </Button>
      </div>

      <!-- Right controls: Maximize, Minimize, Close -->
      <div class="flex items-center gap-0.5 px-1.5 shrink-0">
        <Button
          variant="text"
          size="small"
          class="w-7! h-7! p-0! rounded-none text-muted-color hover:text-primary hover:bg-(--bg-hover)"
          v-tooltip.top="isMaximized ? 'Restore Terminal' : 'Maximize Terminal'"
          @click="handleToggleMaximize"
        >
          <template #icon>
            <Minimize2 v-if="isMaximized" :size="13" />
            <Maximize2 v-else :size="13" />
          </template>
        </Button>
        <Button
          variant="text"
          size="small"
          class="w-7! h-7! p-0! rounded-none text-muted-color hover:text-primary hover:bg-(--bg-hover)"
          v-tooltip.top="'Minimize Panel'"
          @click="terminalStore.closePanel()"
        >
          <template #icon>
            <ChevronDown :size="13" />
          </template>
        </Button>
        <Button
          variant="text"
          size="small"
          class="w-7! h-7! p-0! rounded-none text-muted-color hover:text-primary hover:bg-(--bg-hover)"
          v-tooltip.top="'Close Terminal'"
          @click="terminalStore.closePanel()"
        >
          <template #icon>
            <X :size="13" />
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
        class="flex flex-col items-center justify-center h-full text-muted-color text-xs gap-3 p-6 font-mono select-none"
      >
        <div
          class="p-3 rounded-none border border-(--border) bg-(--bg-sidebar)/50 flex items-center justify-center mb-1"
        >
          <TerminalIcon class="w-6 h-6 text-muted-color opacity-60" />
        </div>
        <div class="flex flex-col items-center gap-1">
          <span class="font-medium text-primary">No active terminal sessions</span>
          <span class="text-[11px] text-muted-color"
            >Open a local shell or connect to a pod container</span
          >
        </div>
        <Button
          size="small"
          variant="outlined"
          class="text-xs font-mono flex items-center gap-1.5 px-3 py-1.5 rounded-none border-(--border) text-primary hover:border-(--accent) mt-2"
          @click="handleNewLocalTerminal"
        >
          <Plus class="w-3.5 h-3.5" />
          <span>New Local Shell</span>
        </Button>
      </div>
    </div>
  </div>
</template>
