<script setup lang="ts">
import { events } from '@/services/nativeService'
import { terminalService } from '@/services/terminalService'
import { OrbitEvents } from '@/types/events'
import type { TerminalSession } from '@/types/terminal'
import { FitAddon } from '@xterm/addon-fit'
import { Terminal } from '@xterm/xterm'
import '@xterm/xterm/css/xterm.css'
import { nextTick, onMounted, onUnmounted, ref, watch } from 'vue'

const props = defineProps<{
  session: TerminalSession
  isActive: boolean
}>()

const terminalContainer = ref<HTMLDivElement | null>(null)
let term: Terminal | null = null
let fitAddon: FitAddon | null = null
let resizeObserver: ResizeObserver | null = null

const handleTerminalData = (payload: { sessionId: string; data: string }) => {
  if (payload.sessionId === props.session.id && term) {
    term.write(payload.data)
  }
}

const handleTerminalClosed = (payload: { sessionId: string; exitCode?: number }) => {
  if (payload.sessionId === props.session.id && term) {
    const code = payload.exitCode ?? 0
    term.writeln(`\r\n\x1b[90m[Process finished with exit code ${code}]\x1b[0m\r\n`)
  }
}

function handleResize() {
  if (!fitAddon || !term || !props.isActive) return
  try {
    fitAddon.fit()
    if (term.cols && term.rows) {
      terminalService.resize(props.session.id, term.cols, term.rows)
    }
  } catch {
    // Ignore fit errors if container is hidden or 0x0
  }
}

onMounted(async () => {
  if (!terminalContainer.value) return

  // Read current theme colors dynamically from CSS variables
  const computedStyle = getComputedStyle(document.documentElement)
  const bg = computedStyle.getPropertyValue('--bg-app').trim() || '#121214'
  const fg = computedStyle.getPropertyValue('--text-primary').trim() || '#ececed'
  const cursor = computedStyle.getPropertyValue('--accent').trim() || '#4f8cff'

  term = new Terminal({
    cursorBlink: true,
    fontFamily: 'var(--font-mono), monospace',
    fontSize: 12,
    lineHeight: 1.25,
    theme: {
      background: bg,
      foreground: fg,
      cursor: cursor,
      selectionBackground: 'rgba(79, 140, 255, 0.3)'
    },
    convertEol: true
  })

  fitAddon = new FitAddon()
  term.loadAddon(fitAddon)

  term.open(terminalContainer.value)

  term.onData((data) => {
    terminalService.sendData(props.session.id, data)
  })

  events.on(OrbitEvents.TerminalData, handleTerminalData)
  events.on(OrbitEvents.TerminalClosed, handleTerminalClosed)

  // Start the session on backend
  if (props.session.type === 'pod') {
    await terminalService.openPodTerminal(
      props.session.id,
      props.session.namespace || 'default',
      props.session.pod || '',
      props.session.container
    )
  } else {
    await terminalService.openLocalTerminal(props.session.id)
  }

  await nextTick()
  handleResize()
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
        handleResize()
        term?.focus()
      })
    }
  }
)

onUnmounted(() => {
  if (resizeObserver && terminalContainer.value) {
    resizeObserver.disconnect()
    resizeObserver = null
  }

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
    ref="terminalContainer"
    class="w-full h-full bg-(--bg-app) overflow-hidden p-2 font-mono text-xs"
  />
</template>
