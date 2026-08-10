<script setup lang="ts">
import LogHighlightRulesDialog from '@/components/logs/LogHighlightRulesDialog.vue'
import { useLogHighlighting } from '@/composables/useLogHighlighting'
import { useLogSelection } from '@/composables/useLogSelection'
import { useLogStream } from '@/composables/useLogStream'
import { ArrowLeft } from '@lucide/vue'
import Button from 'primevue/button'
import Checkbox from 'primevue/checkbox'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { useRouter } from 'vue-router'

import { useLogsStore } from '@/stores/logsStore'
import { watch } from 'vue'

const router = useRouter()
const logsStore = useLogsStore()

const {
  selectedNamespace,
  selectedWorkloadName,
  selectedWorkloadKind,
  selectedPodName,
  selectedContainerName,
  tailLines,
  namespaces,
  workloads,
  workloadKinds,
  podOptions,
  containerOptions,
  tailLinesOptions
} = useLogSelection()

watch(
  [selectedNamespace, selectedPodName, selectedContainerName],
  ([ns, pod, container]) => {
    if (ns && pod && pod !== 'All') {
      logsStore.addRecentLog({
        namespace: ns,
        workloadKind: selectedWorkloadKind.value,
        workloadName: selectedWorkloadName.value,
        pod,
        container: container || 'All'
      })
    }
  },
  { immediate: true }
)

const logHighlighting = useLogHighlighting()
const { showRulesDialog, loadRules, getLogLevelColor } = logHighlighting

const {
  logLines,
  maxLogLines,
  searchQuery,
  isRegex,
  showTimestamps,
  isPaused,
  isFullscreen,
  filteredLogLines,
  clearLogs,
  downloadLogs
} = useLogStream({
  selectedNamespace,
  selectedWorkloadName,
  selectedWorkloadKind,
  selectedPodName,
  selectedContainerName,
  tailLines,
  onMountedCallback: loadRules
})
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
          <h2 class="text-xl font-bold tracking-tight text-primary">Logs</h2>
          <p class="text-sm text-muted-color">Stream logs from your container in real-time</p>
        </div>
      </div>
    </div>

    <!-- Filters Bar -->
    <Card>
      <template #content>
        <div class="flex flex-wrap items-center gap-3 p-3">
          <div class="flex flex-col gap-1">
            <label class="text-sm font-bold text-muted-color">Namespace</label>
            <Select v-model="selectedNamespace" :options="namespaces" class="text-sm min-w-36" />
          </div>

          <div class="flex flex-col gap-1">
            <label class="text-sm font-bold text-muted-color">Kind</label>
            <Select
              v-model="selectedWorkloadKind"
              :options="workloadKinds"
              class="text-sm min-w-32"
            />
          </div>

          <div class="flex flex-col gap-1">
            <label class="text-sm font-bold text-muted-color">Workload</label>
            <Select v-model="selectedWorkloadName" :options="workloads" class="text-sm min-w-44" />
          </div>

          <div class="flex flex-col gap-1">
            <label class="text-sm font-bold text-muted-color">Pod</label>
            <Select v-model="selectedPodName" :options="podOptions" class="text-sm min-w-44" />
          </div>

          <div class="flex flex-col gap-1">
            <label class="text-sm font-bold text-muted-color">Container</label>
            <Select
              v-model="selectedContainerName"
              :options="containerOptions"
              class="text-sm min-w-36"
            />
          </div>

          <div class="flex flex-col gap-1">
            <label class="text-sm font-bold text-muted-color">Lines</label>
            <Select
              v-model="tailLines"
              :options="tailLinesOptions"
              optionLabel="label"
              optionValue="value"
              class="text-sm min-w-28"
            />
          </div>
        </div>
      </template>
    </Card>

    <!-- Controls Bar -->
    <Card>
      <template #content>
        <div class="flex items-center justify-between gap-3 p-3">
          <div class="flex items-center gap-4 flex-1">
            <InputText
              v-model="searchQuery"
              placeholder="Search logs..."
              class="text-sm w-full max-w-md"
            />
            <div class="flex items-center gap-2">
              <Checkbox v-model="isRegex" inputId="is-regex" binary class="border-(--border)" />
              <label for="is-regex" class="text-sm text-muted-color cursor-pointer select-none"
                >Regex</label
              >
            </div>
            <div class="flex items-center gap-2">
              <Checkbox v-model="showTimestamps" inputId="show-timestamps" binary />
              <label
                for="show-timestamps"
                class="text-sm text-muted-color cursor-pointer select-none"
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
              variant="text"
              @click="showRulesDialog = true"
              title="Highlight Rules"
            />
            <Button
              icon="pi pi-download"
              size="small"
              variant="text"
              :disabled="logLines.length <= 0"
              @click="downloadLogs"
              title="Download Logs"
            ></Button>
            <Button
              size="small"
              variant="text"
              :icon="isFullscreen ? 'pi pi-window-minimize' : 'pi pi-window-maximize'"
              @click="isFullscreen = !isFullscreen"
              title="Fullscreen"
            >
            </Button>
          </div>
        </div>
      </template>
    </Card>

    <!-- Console Viewer -->
    <div
      ref="terminalRef"
      class="flex-1 dark:bg-zinc-950 rounded p-3 overflow-y-auto font-mono text-sm text-primary-300 dark:text-surface-600 leading-relaxed min-h-0 selection:bg-surface-200 dark:selection:bg-primary-700"
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
          class="flex gap-2 hover:bg-surface-100 dark:hover:bg-primary-800 py-0.5 rounded px-1"
        >
          <!-- Timestamps -->
          <span v-if="showTimestamps && line.timestamp" class="text-zinc-600 select-none shrink-0">
            {{ line.timestamp }}
          </span>

          <!-- Origin Pod/Container Badge -->
          <span class="text-surface-500 font-bold shrink-0 select-none">
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
    <div class="fixed bottom-0 rounded-xl p-2 text-sm text-muted-color px-2">
      <div class="flex w-full justify-between items-center gap-1.5">
        <span
          class="w-2 h-2 rounded-full"
          :class="isPaused ? 'bg-amber-500 animate-pulse' : 'bg-emerald-500 animate-pulse'"
        ></span>
        <span>{{ isPaused ? 'Paused' : 'Streaming...' }}</span>
        <div>
          <span>Total lines in buffer: {{ logLines.length }} / {{ maxLogLines }}</span>
        </div>
      </div>
    </div>

    <!-- Highlight Rules Config Dialog -->
    <LogHighlightRulesDialog v-model:visible="showRulesDialog" :highlighting="logHighlighting" />
  </div>
</template>
