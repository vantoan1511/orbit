<script setup lang="ts">
import { kubernetesService } from '@/services/kubernetesService'
import { events, storage } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import { ArrowLeft } from '@lucide/vue'
import Button from 'primevue/button'
import Checkbox from 'primevue/checkbox'
import Dialog from 'primevue/dialog'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

const route = useRoute()
const router = useRouter()
const k8sStore = useKubernetesStore()

// State
const selectedNamespace = ref<string>((route.query.namespace as string) || 'default')
const selectedWorkloadName = ref<string>((route.query.workload as string) || '')
const selectedWorkloadKind = ref<string>((route.query.kind as string) || 'Deployment')
const selectedPodName = ref<string>('All')
const selectedContainerName = ref<string>('All')
const tailLines = ref<number>(100)
const searchQuery = ref<string>('')
const isRegex = ref<boolean>(false)
const showTimestamps = ref<boolean>(true)
const isPaused = ref<boolean>(false)
const isFullscreen = ref<boolean>(false)

const logLines = ref<Array<{ pod: string; container: string; text: string; timestamp?: string }>>(
  []
)
const maxLogLines = 2000
const terminalRef = ref<HTMLDivElement | null>(null)

// Dropdown options
const namespaces = computed(() => k8sStore.namespaces.filter((n) => n !== 'All Namespaces'))

const workloads = computed(() => {
  const ns = selectedNamespace.value
  if (selectedWorkloadKind.value === 'Deployment') {
    return k8sStore.deployments.filter((d) => d.namespace === ns).map((d) => d.name)
  } else if (selectedWorkloadKind.value === 'StatefulSet') {
    return k8sStore.statefulSets.filter((s) => s.namespace === ns).map((s) => s.name)
  } else if (selectedWorkloadKind.value === 'DaemonSet') {
    return k8sStore.daemonSets.filter((d) => d.namespace === ns).map((d) => d.name)
  } else if (selectedWorkloadKind.value === 'ReplicaSet') {
    return k8sStore.replicaSets.filter((r) => r.namespace === ns).map((r) => r.name)
  } else if (selectedWorkloadKind.value === 'Job') {
    return k8sStore.jobs.filter((j) => j.namespace === ns).map((j) => j.name)
  } else if (selectedWorkloadKind.value === 'CronJob') {
    return k8sStore.cronJobs.filter((c) => c.namespace === ns).map((c) => c.name)
  }
  return []
})

const workloadKinds = [
  'Deployment',
  'StatefulSet',
  'DaemonSet',
  'ReplicaSet',
  'Job',
  'CronJob',
  'Pod'
]

const workloadPods = computed(() => {
  if (!selectedWorkloadName.value) return []
  if (selectedWorkloadKind.value === 'Pod') {
    const pod = k8sStore.pods.find(
      (p) => p.name === selectedWorkloadName.value && p.namespace === selectedNamespace.value
    )
    return pod ? [pod] : []
  }
  return k8sStore.pods.filter(
    (p) =>
      p.namespace === selectedNamespace.value && p.name.startsWith(selectedWorkloadName.value + '-')
  )
})

const podOptions = computed(() => {
  return ['All', ...workloadPods.value.map((p) => p.name)]
})

const containerOptions = computed(() => {
  const options = new Set<string>()
  if (selectedPodName.value && selectedPodName.value !== 'All') {
    const pod = k8sStore.pods.find((p) => p.name === selectedPodName.value)
    pod?.containers?.forEach((c) => options.add(c.name))
  } else {
    workloadPods.value.forEach((p) => {
      p.containers?.forEach((c) => options.add(c.name))
    })
  }
  return ['All', ...Array.from(options)]
})

const tailLinesOptions = [
  { label: '50 lines', value: 50 },
  { label: '100 lines', value: 100 },
  { label: '250 lines', value: 250 },
  { label: '500 lines', value: 500 },
  { label: '1000 lines', value: 1000 }
]

// Log Streaming Control
const startStreaming = async () => {
  logLines.value = []
  await kubernetesService.stopLogs()

  if (!selectedNamespace.value || !selectedWorkloadName.value) return

  await kubernetesService.streamLogs({
    namespace: selectedNamespace.value,
    workload: selectedWorkloadName.value,
    kind: selectedWorkloadKind.value,
    pod: selectedPodName.value === 'All' ? undefined : selectedPodName.value,
    container: selectedContainerName.value === 'All' ? undefined : selectedContainerName.value,
    tailLines: tailLines.value
  })
}

// Log formatting helper (extract timestamps and levels)
const parseLogLine = (rawLine: string) => {
  let text = rawLine
  let timestamp: string | undefined

  // RFC3339 timestamp regex (e.g. 2025-07-18T17:42:31.123Z)
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

  // Limit memory usage (prune in batches of 100 to minimize reallocations)
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

// Filtered log lines based on search query
const filteredLogLines = computed(() => {
  if (!searchQuery.value) return logLines.value

  return logLines.value.filter((line) => {
    if (isRegex.value) {
      try {
        const regex = new RegExp(searchQuery.value, 'i')
        return regex.test(line.text)
      } catch {
        return false // invalid regex
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
  link.setAttribute('download', `${selectedWorkloadName.value}-logs.txt`)
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
}

// Event Listeners
onMounted(async () => {
  await loadRules()
  events.on(OrbitEvents.LogLineReceived, handleLogLine)
  startStreaming()
})

onUnmounted(async () => {
  events.off(OrbitEvents.LogLineReceived, handleLogLine)
  await kubernetesService.stopLogs()
})

// Watchers to trigger restart of logs stream
watch(
  [
    selectedNamespace,
    selectedWorkloadName,
    selectedWorkloadKind,
    selectedPodName,
    selectedContainerName,
    tailLines
  ],
  () => {
    startStreaming()
  }
)

watch(selectedWorkloadKind, () => {
  // Reset selected workload when changing kind
  selectedWorkloadName.value = workloads.value[0] || ''
})

watch(selectedNamespace, () => {
  // Fetch fresh workloads on namespace change
  selectedWorkloadName.value = workloads.value[0] || ''
})

interface HighlightRule {
  id: string
  pattern: string
  color: string
  bold: boolean
  caseSensitive: boolean
}

const DEFAULT_RULES: HighlightRule[] = [
  { id: '1', pattern: 'error', color: 'rose', bold: true, caseSensitive: false },
  { id: '2', pattern: 'fail', color: 'rose', bold: true, caseSensitive: false },
  { id: '3', pattern: 'err:', color: 'rose', bold: true, caseSensitive: false },
  { id: '4', pattern: 'warn', color: 'amber', bold: true, caseSensitive: false },
  { id: '5', pattern: 'warning', color: 'amber', bold: true, caseSensitive: false },
  { id: '6', pattern: 'info', color: 'emerald', bold: true, caseSensitive: false },
  { id: '7', pattern: 'debug', color: 'emerald', bold: true, caseSensitive: false }
]

const showRulesDialog = ref(false)
const rules = ref<HighlightRule[]>([])

const colorOptions = [
  { label: 'Red', value: 'rose' },
  { label: 'Orange', value: 'amber' },
  { label: 'Green', value: 'emerald' },
  { label: 'Blue', value: 'sky' },
  { label: 'Purple', value: 'purple' },
  { label: 'Pink', value: 'fuchsia' },
  { label: 'Gray', value: 'gray' }
]

const loadRules = async () => {
  try {
    const saved = await storage.getData('orbit_log_highlight_rules')
    if (saved) {
      rules.value = JSON.parse(saved)
    } else {
      rules.value = [...DEFAULT_RULES]
    }
  } catch (err) {
    // Typically means key doesn't exist in storage
    rules.value = [...DEFAULT_RULES]
  }
}

const saveRules = async () => {
  try {
    await storage.setData('orbit_log_highlight_rules', JSON.stringify(rules.value))
  } catch (err) {
    console.error('Failed to save log highlight rules', err)
  }
}

const addRule = () => {
  rules.value.push({
    id: Date.now().toString(),
    pattern: '',
    color: 'rose',
    bold: false,
    caseSensitive: false
  })
  saveRules()
}

const deleteRule = (index: number) => {
  rules.value.splice(index, 1)
  saveRules()
}

const getLogLevelColor = (text: string) => {
  const matched = rules.value.find((rule) => {
    if (!rule.pattern) return false
    const matchText = rule.caseSensitive ? text : text.toLowerCase()
    const pattern = rule.caseSensitive ? rule.pattern : rule.pattern.toLowerCase()
    return matchText.includes(pattern)
  })

  if (matched) {
    let classes = ''
    switch (matched.color) {
      case 'rose':
        classes = 'text-rose-500'
        break
      case 'amber':
        classes = 'text-amber-500'
        break
      case 'emerald':
        classes = 'text-emerald-500'
        break
      case 'sky':
        classes = 'text-sky-500'
        break
      case 'purple':
        classes = 'text-purple-500'
        break
      case 'fuchsia':
        classes = 'text-fuchsia-500'
        break
      case 'gray':
        classes = 'text-zinc-400'
        break
      default:
        classes = 'text-(--text-secondary)'
        break
    }
    if (matched.bold) {
      classes += ' font-bold'
    }
    return classes
  }

  return 'text-(--text-secondary)'
}
</script>

<template>
  <div
    class="flex flex-col gap-4 h-[calc(100vh-6rem)]"
    :class="{ 'fixed inset-0 z-50 bg-(--bg-card) p-6 h-screen': isFullscreen }"
  >
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <Button
          severity="secondary"
          variant="text"
          size="small"
          @click="router.back()"
          v-if="!isFullscreen"
        >
          <ArrowLeft class="w-4 h-4" />
        </Button>
        <div>
          <h2 class="text-xl font-bold tracking-tight text-(--text-primary)">Logs</h2>
          <p class="text-xs text-(--text-muted)">Stream logs from your container in real-time</p>
        </div>
      </div>
    </div>

    <!-- Filters Bar -->
    <div
      class="flex flex-wrap items-center gap-3 p-3 bg-(--bg-hover)/20 border border-(--border) rounded-xl"
    >
      <div class="flex flex-col gap-1">
        <label class="text-[10px] font-bold text-(--text-muted) uppercase">Namespace</label>
        <Select
          v-model="selectedNamespace"
          :options="namespaces"
          class="text-xs min-w-36 bg-(--bg-card) border-(--border)"
        />
      </div>

      <div class="flex flex-col gap-1">
        <label class="text-[10px] font-bold text-(--text-muted) uppercase">Kind</label>
        <Select
          v-model="selectedWorkloadKind"
          :options="workloadKinds"
          class="text-xs min-w-32 bg-(--bg-card) border-(--border)"
        />
      </div>

      <div class="flex flex-col gap-1">
        <label class="text-[10px] font-bold text-(--text-muted) uppercase">Workload</label>
        <Select
          v-model="selectedWorkloadName"
          :options="workloads"
          class="text-xs min-w-44 bg-(--bg-card) border-(--border)"
        />
      </div>

      <div class="flex flex-col gap-1">
        <label class="text-[10px] font-bold text-(--text-muted) uppercase">Pod</label>
        <Select
          v-model="selectedPodName"
          :options="podOptions"
          class="text-xs min-w-44 bg-(--bg-card) border-(--border)"
        />
      </div>

      <div class="flex flex-col gap-1">
        <label class="text-[10px] font-bold text-(--text-muted) uppercase">Container</label>
        <Select
          v-model="selectedContainerName"
          :options="containerOptions"
          class="text-xs min-w-36 bg-(--bg-card) border-(--border)"
        />
      </div>

      <div class="flex flex-col gap-1">
        <label class="text-[10px] font-bold text-(--text-muted) uppercase">Lines</label>
        <Select
          v-model="tailLines"
          :options="tailLinesOptions"
          optionLabel="label"
          optionValue="value"
          class="text-xs min-w-28 bg-(--bg-card) border-(--border)"
        />
      </div>
    </div>

    <!-- Controls Bar -->
    <div
      class="flex items-center justify-between gap-4 p-3 bg-(--bg-hover)/10 border border-(--border) rounded-xl"
    >
      <div class="flex items-center gap-4 flex-1">
        <InputText
          v-model="searchQuery"
          placeholder="Search logs..."
          class="text-xs bg-(--bg-card) border-(--border) w-full max-w-md"
        />
        <div class="flex items-center gap-2">
          <Checkbox v-model="isRegex" inputId="is-regex" binary class="border-(--border)" />
          <label for="is-regex" class="text-xs text-(--text-secondary) cursor-pointer select-none"
            >Regex</label
          >
        </div>
        <div class="flex items-center gap-2">
          <Checkbox
            v-model="showTimestamps"
            inputId="show-timestamps"
            binary
            class="border-(--border)"
          />
          <label
            for="show-timestamps"
            class="text-xs text-(--text-secondary) cursor-pointer select-none"
            >Timestamps</label
          >
        </div>
      </div>

      <div class="flex items-center gap-2">
        <Button
          variant="text"
          size="small"
          :icon="isPaused ? 'pi pi-play' : 'pi pi-pause'"
          @click="isPaused = !isPaused"
        />
        <Button icon="pi pi-trash" size="small" variant="text" @click="clearLogs" />
      </div>

      <div class="flex justify-center items-center gap-2">
        <Button
          icon="pi pi-palette"
          size="small"
          severity="secondary"
          variant="text"
          @click="showRulesDialog = true"
          title="Highlight Rules"
        />
        <Button
          icon="pi pi-download"
          size="small"
          severity="secondary"
          variant="text"
          :disabled="logLines.length <= 0"
          @click="downloadLogs"
          title="Download Logs"
        ></Button>
        <Button
          size="small"
          severity="secondary"
          variant="text"
          :icon="isFullscreen ? 'pi pi-window-minimize' : 'pi pi-window-maximize'"
          @click="isFullscreen = !isFullscreen"
          title="Fullscreen"
        >
        </Button>
      </div>
    </div>

    <!-- Console Viewer -->
    <div
      ref="terminalRef"
      class="flex-1 bg-zinc-950 border border-zinc-800 rounded-xl p-4 overflow-y-auto font-mono text-xs text-zinc-300 leading-relaxed min-h-0 selection:bg-zinc-800"
    >
      <div
        v-if="filteredLogLines.length === 0"
        class="flex flex-col items-center justify-center h-full text-zinc-500"
      >
        <p>No log lines streamed or matching query.</p>
      </div>
      <div v-else class="space-y-1">
        <div
          v-for="(line, idx) in filteredLogLines"
          :key="idx"
          class="flex gap-2 hover:bg-zinc-900/50 py-0.5 rounded px-1"
        >
          <!-- Timestamps -->
          <span v-if="showTimestamps && line.timestamp" class="text-zinc-600 select-none shrink-0">
            {{ line.timestamp }}
          </span>

          <!-- Origin Pod/Container Badge -->
          <span class="text-zinc-500 font-bold shrink-0 select-none">
            [{{ line.pod.split('-').pop() }}/{{ line.container }}]
          </span>

          <!-- Log Content -->
          <span class="break-all whitespace-pre-wrap flex-1" :class="getLogLevelColor(line.text)">
            {{ line.text }}
          </span>
        </div>
      </div>
    </div>

    <!-- Status Bar -->
    <div
      class="fixed bottom-0 bg-(--bg-hover)/10 border border-(--border) rounded-xl p-2 text-[10px] text-(--text-muted) px-2"
    >
      <div class="flex w-full justify-between items-center gap-1.5">
        <span
          class="w-2 h-2 rounded-full"
          :class="isPaused ? 'bg-amber-500 animate-pulse' : 'bg-emerald-500 animate-pulse'"
        ></span>
        <span>{{ isPaused ? 'Streaming Paused' : 'Streaming logs...' }}</span>
        <div>
          <span>Total lines in buffer: {{ logLines.length }} / {{ maxLogLines }}</span>
        </div>
      </div>
    </div>

    <!-- Highlight Rules Config Dialog -->
    <Dialog
      v-model:visible="showRulesDialog"
      modal
      header="Highlight Rules"
      :style="{ width: '550px' }"
      class="bg-(--bg-card) border border-(--border)"
    >
      <div class="flex flex-col gap-4">
        <p class="text-xs text-(--text-muted)">
          Define search patterns to style log lines dynamically.
        </p>
        <div class="flex flex-col gap-2 max-h-80 overflow-y-auto pr-1">
          <div
            v-for="(rule, index) in rules"
            :key="rule.id"
            class="flex items-center gap-2 p-2 bg-(--bg-hover)/20 border border-(--border) rounded-lg"
          >
            <InputText
              v-model="rule.pattern"
              placeholder="Pattern..."
              class="text-xs bg-(--bg-card) border-(--border) flex-1"
              @change="saveRules"
            />
            <Select
              v-model="rule.color"
              :options="colorOptions"
              optionLabel="label"
              optionValue="value"
              class="text-xs min-w-28 bg-(--bg-card) border-(--border)"
              @change="saveRules"
            />
            <div class="flex items-center gap-1.5 ml-1">
              <Checkbox
                v-model="rule.bold"
                :inputId="'bold-' + rule.id"
                binary
                class="border-(--border)"
                @change="saveRules"
              />
              <label
                :for="'bold-' + rule.id"
                class="text-[10px] uppercase font-bold text-(--text-muted) cursor-pointer select-none"
                >Bold</label
              >
            </div>
            <div class="flex items-center gap-1.5 ml-1">
              <Checkbox
                v-model="rule.caseSensitive"
                :inputId="'cs-' + rule.id"
                binary
                class="border-(--border)"
                @change="saveRules"
              />
              <label
                :for="'cs-' + rule.id"
                class="text-[10px] uppercase font-bold text-(--text-muted) cursor-pointer select-none"
                >CS</label
              >
            </div>
            <Button
              icon="pi pi-trash"
              severity="danger"
              variant="text"
              size="small"
              @click="deleteRule(index)"
            />
          </div>
        </div>
        <div class="flex justify-between items-center mt-2">
          <Button
            label="Add Rule"
            icon="pi pi-plus"
            size="small"
            severity="secondary"
            variant="text"
            @click="addRule"
          />
          <Button label="Close" size="small" variant="text" @click="showRulesDialog = false" />
        </div>
      </div>
    </Dialog>
  </div>
</template>
