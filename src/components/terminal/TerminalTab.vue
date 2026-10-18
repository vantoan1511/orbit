<script setup lang="ts">
import { useAppearance } from '@/composables/useAppearance'
import { useTheme } from '@/composables/useTheme'
import { events } from '@/services/nativeService'
import { terminalService } from '@/services/terminalService'
import { useTerminalStore } from '@/stores/terminalStore'
import { OrbitEvents } from '@/types/events'
import type { TerminalSession } from '@/types/terminal'
import { shouldDispatchTerminalResize, type TerminalDimensions } from '@/utils/terminalHelpers'
import { FitAddon } from '@xterm/addon-fit'
import { Terminal } from '@xterm/xterm'
import '@xterm/xterm/css/xterm.css'
import { nextTick, onMounted, onUnmounted, ref, watch } from 'vue'

const props = defineProps<{
  session: TerminalSession
  isActive: boolean
}>()

const { isDark } = useTheme()
const { accentColor } = useAppearance()
const terminalStore = useTerminalStore()

const terminalContainer = ref<HTMLDivElement | null>(null)
let term: Terminal | null = null
let fitAddon: FitAddon | null = null
let resizeObserver: ResizeObserver | null = null
let resizeTimeout: ReturnType<typeof setTimeout> | null = null
let lastDispatchedDims: TerminalDimensions | null = null

function getTerminalTheme() {
  const computedStyle = getComputedStyle(document.documentElement)
  const bg = computedStyle.getPropertyValue('--bg-app').trim() || '#0b0d0f'
  const fg = computedStyle.getPropertyValue('--text-primary').trim() || '#ececed'
  const cursor = computedStyle.getPropertyValue('--accent').trim() || '#4f8cff'
  const selection =
    computedStyle.getPropertyValue('--selection').trim() || 'rgba(79, 140, 255, 0.3)'

  return {
    background: bg,
    foreground: fg,
    cursor: cursor,
    selectionBackground: selection
  }
}

const handleTerminalStarted = (payload: { sessionId: string; shell: string }) => {
  if (payload.sessionId === props.session.id) {
    terminalStore.updateSessionShell(props.session.id, payload.shell)
  }
}

const handleTerminalData = (payload: { sessionId: string; data: string }) => {
  if (payload.sessionId === props.session.id && term) {
    term.write(payload.data)
  }
}

const handleTerminalClosed = (payload: { sessionId: string; exitCode?: number }) => {
  if (payload.sessionId === props.session.id) {
    terminalStore.removeSession(props.session.id)
  }
}

function handleResize(immediate = false) {
  if (!fitAddon || !term || !props.isActive) return
  try {
    fitAddon.fit()
    const newDims: TerminalDimensions = { cols: term.cols, rows: term.rows }
    if (shouldDispatchTerminalResize(newDims, lastDispatchedDims)) {
      if (resizeTimeout) {
        clearTimeout(resizeTimeout)
        resizeTimeout = null
      }

      const applyResize = () => {
        if (!term) return
        const currentDims: TerminalDimensions = { cols: term.cols, rows: term.rows }
        if (shouldDispatchTerminalResize(currentDims, lastDispatchedDims)) {
          lastDispatchedDims = currentDims
          terminalService.resize(props.session.id, currentDims.cols, currentDims.rows)
        }
      }

      if (immediate) {
        applyResize()
      } else {
        resizeTimeout = setTimeout(applyResize, 80)
      }
    }
  } catch {
    // Ignore fit errors if container is hidden or 0x0
  }
}

onMounted(async () => {
  if (!terminalContainer.value) return

  term = new Terminal({
    cursorBlink: true,
    fontFamily: 'var(--font-mono), monospace',
    fontSize: 12,
    lineHeight: 1.25,
    theme: getTerminalTheme(),
    convertEol: true
  })

  fitAddon = new FitAddon()
  term.loadAddon(fitAddon)

  term.open(terminalContainer.value)

  term.onData((data) => {
    terminalService.sendData(props.session.id, data)
  })

  events.on(OrbitEvents.TerminalStarted, handleTerminalStarted)
  events.on(OrbitEvents.TerminalData, handleTerminalData)
  events.on(OrbitEvents.TerminalClosed, handleTerminalClosed)

  const initialDims = fitAddon.proposeDimensions()
  if (initialDims && initialDims.cols > 0 && initialDims.rows > 0) {
    lastDispatchedDims = { cols: initialDims.cols, rows: initialDims.rows }
  }

  // Start the session on backend with measured container dimensions
  if (props.session.type === 'pod') {
    await terminalService.openPodTerminal(
      props.session.id,
      props.session.namespace || 'default',
      props.session.pod || '',
      props.session.container,
      initialDims?.cols,
      initialDims?.rows
    )
  } else {
    await terminalService.openLocalTerminal(
      props.session.id,
      undefined,
      initialDims?.cols,
      initialDims?.rows
    )
  }

  await nextTick()
  handleResize(true)
  if (props.isActive) {
    term.focus()
  }

  resizeObserver = new ResizeObserver(() => {
    handleResize()
  })
  resizeObserver.observe(terminalContainer.value)
})

watch(
  () => props.isActive,
  (active) => {
    if (active) {
      nextTick(() => {
        handleResize(true)
        term?.focus()
      })
    }
  }
)

watch([isDark, accentColor], () => {
  nextTick(() => {
    if (term) {
      term.options.theme = getTerminalTheme()
    }
  })
})

onUnmounted(() => {
  if (resizeTimeout) {
    clearTimeout(resizeTimeout)
    resizeTimeout = null
  }
  lastDispatchedDims = null

  if (resizeObserver && terminalContainer.value) {
    resizeObserver.disconnect()
    resizeObserver = null
  }

  events.off(OrbitEvents.TerminalStarted, handleTerminalStarted)
  events.off(OrbitEvents.TerminalData, handleTerminalData)
  events.off(OrbitEvents.TerminalClosed, handleTerminalClosed)

  terminalService.close(props.session.id)

  if (term) {
    term.dispose()
    term = null
  }
  fitAddon = null
})
</script>

<template>
  <div
    data-component="terminal-tab"
    class="w-full h-full bg-(--bg-app) overflow-hidden p-2 font-mono text-xs"
  >
    <div ref="terminalContainer" class="w-full h-full overflow-hidden" />
  </div>
</template>
