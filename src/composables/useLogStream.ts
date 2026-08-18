import { ref, computed, watch, onMounted, onUnmounted, nextTick, type Ref } from 'vue'
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { OrbitEvents, TAIL_ALL_LINES } from '@/types/events'
import type { VirtualScrollerMethods } from 'primevue/virtualscroller'

export interface LogLine {
  pod: string
  container: string
  text: string
  timestamp?: string
}

export function useLogStream(options: {
  selectedNamespace: Ref<string>
  selectedWorkloadName: Ref<string>
  selectedWorkloadKind: Ref<string>
  selectedPodName: Ref<string>
  selectedContainerName: Ref<string>
  tailLines: Ref<number>
  onMountedCallback?: () => Promise<void>
}) {
  const logLines = ref<LogLine[]>([])
  const maxLogLines = 2000
  // Maximum lines retained in All mode to prevent unbounded memory growth.
  const maxLogLinesAll = 100_000
  const virtualScrollerRef = ref<VirtualScrollerMethods | null>(null)

  const searchQuery = ref<string>('')
  const isRegex = ref<boolean>(false)
  const showTimestamps = ref<boolean>(true)
  const isPaused = ref<boolean>(false)
  const isFullscreen = ref<boolean>(false)
  const isFollowing = ref<boolean>(true)

  const startStreaming = async () => {
    logLines.value = []
    await kubernetesService.stopLogs()

    if (!options.selectedNamespace.value || !options.selectedWorkloadName.value) return

    await kubernetesService.streamLogs({
      namespace: options.selectedNamespace.value,
      workload: options.selectedWorkloadName.value,
      kind: options.selectedWorkloadKind.value,
      pod: options.selectedPodName.value === 'All' ? undefined : options.selectedPodName.value,
      container:
        options.selectedContainerName.value === 'All'
          ? undefined
          : options.selectedContainerName.value,
      tailLines: options.tailLines.value
    })
  }

  const parseLogLine = (rawLine: string) => {
    let text = rawLine
    let timestamp: string | undefined

    const tsRegex =
      /^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z|\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z)\s?/
    const match = rawLine.match(tsRegex)
    if (match) {
      const rawTimestamp = match[1]
      if (rawTimestamp) {
        try {
          const date = new Date(rawTimestamp)
          timestamp = isNaN(date.getTime()) ? rawTimestamp : date.toLocaleString()
        } catch {
          timestamp = rawTimestamp
        }
      }
      text = rawLine.replace(tsRegex, '')
    }

    return { text, timestamp }
  }

  const handleLogLine = (data: { pod: string; container: string; line: string }) => {
    if (isPaused.value) return

    const { text, timestamp } = parseLogLine(data.line)
    logLines.value.push({
      pod: data.pod,
      container: data.container,
      text,
      timestamp
    })

    if (options.tailLines.value === TAIL_ALL_LINES) {
      if (logLines.value.length > maxLogLinesAll) {
        logLines.value = logLines.value.slice(-maxLogLinesAll)
      }
    } else if (logLines.value.length > maxLogLines + 100) {
      logLines.value = logLines.value.slice(-maxLogLines)
    }

    if (isFollowing.value) {
      scrollToBottom()
    }
  }

  const handleLogLinesChunk = (data: { pod: string; container: string; lines: string[] }) => {
    if (isPaused.value) return

    const parsedLines: LogLine[] = data.lines.map((rawLine) => {
      const { text, timestamp } = parseLogLine(rawLine)
      return {
        pod: data.pod,
        container: data.container,
        text,
        timestamp
      }
    })

    logLines.value.push(...parsedLines)

    if (options.tailLines.value === TAIL_ALL_LINES) {
      if (logLines.value.length > maxLogLinesAll) {
        logLines.value = logLines.value.slice(-maxLogLinesAll)
      }
    } else if (logLines.value.length > maxLogLines + 100) {
      logLines.value = logLines.value.slice(-maxLogLines)
    }

    if (isFollowing.value) {
      scrollToBottom()
    }
  }

  const isAtBottom = ref<boolean>(true)

  const scrollToBottom = () => {
    isAtBottom.value = true
    isFollowing.value = true
    nextTick(() => {
      if (virtualScrollerRef.value && filteredLogLines.value.length > 0) {
        virtualScrollerRef.value.scrollToIndex(filteredLogLines.value.length - 1)
      }
    })
  }

  const onScroll = (event: Event) => {
    const target = event.target as HTMLElement
    if (!target) return
    const tolerance = 20
    const atBottom = target.scrollHeight - target.scrollTop - target.clientHeight <= tolerance
    isAtBottom.value = atBottom

    if (!atBottom && isFollowing.value) {
      isFollowing.value = false
    }
  }

  const filteredLogLines = computed(() => {
    if (!searchQuery.value) return logLines.value

    return logLines.value.filter((line) => {
      if (isRegex.value) {
        try {
          const regex = new RegExp(searchQuery.value, 'i')
          return regex.test(line.text)
        } catch {
          return false
        }
      }
      return line.text.toLowerCase().includes(searchQuery.value.toLowerCase())
    })
  })

  const clearLogs = () => {
    logLines.value = []
  }

  const isCopied = ref<boolean>(false)

  const copyLogs = async () => {
    const content = filteredLogLines.value
      .map((l) => `${l.timestamp ? l.timestamp + ' ' : ''}[${l.pod}/${l.container}] ${l.text}`)
      .join('\n')

    try {
      await navigator.clipboard.writeText(content)
      isCopied.value = true
      setTimeout(() => {
        isCopied.value = false
      }, 1500)
    } catch (err) {
      console.error('Failed to copy logs to clipboard:', err)
    }
  }

  const downloadLogs = () => {
    const content = filteredLogLines.value
      .map((l) => `${l.timestamp ? l.timestamp + ' ' : ''}[${l.pod}/${l.container}] ${l.text}`)
      .join('\n')

    const blob = new Blob([content], { type: 'text/plain;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.setAttribute('download', `${options.selectedWorkloadName.value}-logs.txt`)
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
  }

  onMounted(async () => {
    if (options.onMountedCallback) {
      await options.onMountedCallback()
    }
    // LogLineReceived is kept for backward compatibility; stream_pod_logs now
    // always emits LogLinesChunkReceived (even for single lines).
    events.on(OrbitEvents.LogLineReceived, handleLogLine)
    events.on(OrbitEvents.LogLinesChunkReceived, handleLogLinesChunk)
    startStreaming()
  })

  onUnmounted(async () => {
    events.off(OrbitEvents.LogLineReceived, handleLogLine)
    events.off(OrbitEvents.LogLinesChunkReceived, handleLogLinesChunk)
    await kubernetesService.stopLogs()
  })

  watch(
    [
      options.selectedNamespace,
      options.selectedWorkloadName,
      options.selectedWorkloadKind,
      options.selectedPodName,
      options.selectedContainerName,
      options.tailLines
    ],
    () => {
      startStreaming()
    }
  )

  return {
    logLines,
    maxLogLines,
    virtualScrollerRef,
    searchQuery,
    isRegex,
    showTimestamps,
    isPaused,
    isFullscreen,
    isFollowing,
    filteredLogLines,
    isAtBottom,
    onScroll,
    scrollToBottom,
    clearLogs,
    downloadLogs,
    copyLogs,
    isCopied
  }
}
