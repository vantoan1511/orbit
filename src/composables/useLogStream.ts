import { ref, computed, watch, onMounted, onUnmounted, nextTick, type Ref } from 'vue'
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { OrbitEvents } from '@/types/events'

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
  const terminalRef = ref<HTMLDivElement | null>(null)

  const searchQuery = ref<string>('')
  const isRegex = ref<boolean>(false)
  const showTimestamps = ref<boolean>(true)
  const isPaused = ref<boolean>(false)
  const isFullscreen = ref<boolean>(false)

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
      timestamp = match[1]
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

    if (logLines.value.length > maxLogLines + 100) {
      logLines.value = logLines.value.slice(-maxLogLines)
    }

    scrollToBottom()
  }

  const scrollToBottom = () => {
    nextTick(() => {
      if (terminalRef.value) {
        terminalRef.value.scrollTop = terminalRef.value.scrollHeight
      }
    })
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
    events.on(OrbitEvents.LogLineReceived, handleLogLine)
    startStreaming()
  })

  onUnmounted(async () => {
    events.off(OrbitEvents.LogLineReceived, handleLogLine)
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
    terminalRef,
    searchQuery,
    isRegex,
    showTimestamps,
    isPaused,
    isFullscreen,
    filteredLogLines,
    clearLogs,
    downloadLogs
  }
}
