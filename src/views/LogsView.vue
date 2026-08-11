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
import ToggleSwitch from 'primevue/toggleswitch'
import VirtualScroller from 'primevue/virtualscroller'
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
  searchQuery,
  isRegex,
  showTimestamps,
  isPaused,
  isFullscreen,
  isFollowing,
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
    class="flex flex-col gap-2.5 h-[calc(100vh-6rem)]"
    :class="{ 'fixed inset-0 z-50 bg-surface-200 dark:bg-surface-700 p-4 h-screen': isFullscreen }"
  >
    <!-- Header -->
    <div class="flex items-center justify-between" v-if="!isFullscreen">
      <div class="flex items-center gap-2">
        <Button severity="secondary" variant="text" size="small" @click="router.back()">
          <ArrowLeft class="w-4 h-4" />
        </Button>
        <div class="flex items-baseline gap-2">
          <h2 class="text-lg font-bold tracking-tight text-primary leading-none">Logs</h2>
          <span class="text-xs text-muted-color">Stream logs from container in real-time</span>
        </div>
      </div>
    </div>

    <!-- Controls Bar -->
    <Card :dt="{ body: { padding: '0' } }">
      <template #content>
        <div class="flex items-center justify-between gap-3 p-1.5 px-3">
          <div class="flex items-center gap-3.5 flex-1">
            <InputText
              v-model="searchQuery"
              placeholder="Search logs..."
              size="small"
              class="text-xs w-full max-w-xs"
            />
            <div class="flex items-center gap-1.5">
              <Checkbox v-model="isRegex" inputId="is-regex" binary class="border-surface" />
              <label for="is-regex" class="text-xs text-muted-color cursor-pointer select-none"
                >Regex</label
              >
            </div>
            <div class="flex items-center gap-1.5">
              <Checkbox v-model="showTimestamps" inputId="show-timestamps" binary />
              <label
                for="show-timestamps"
                class="text-xs text-muted-color cursor-pointer select-none"
                >Timestamps</label
              >
            </div>
            <div class="flex items-center gap-1.5">
              <ToggleSwitch
                v-model="isFollowing"
                inputId="is-following"
                class="scale-75 origin-left"
              />
              <label
                for="is-following"
                class="text-xs text-muted-color cursor-pointer select-none -ml-1"
                >Follow</label
              >
            </div>
            <div class="flex items-center gap-1.5">
              <span class="text-xs text-muted-color select-none">Lines</span>
              <Select
                v-model="tailLines"
                :options="tailLinesOptions"
                optionLabel="label"
                optionValue="value"
                size="small"
                class="text-xs min-w-28"
              />
            </div>
          </div>

          <div class="flex items-center gap-1.5">
            <Button
              variant="text"
              size="small"
              :icon="isPaused ? 'pi pi-play' : 'pi pi-pause'"
              @click="isPaused = !isPaused"
            />
            <Button icon="pi pi-trash" size="small" variant="text" @click="clearLogs" />
          </div>

          <div class="flex justify-center items-center gap-1.5">
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
      class="flex-1 dark:bg-zinc-950 rounded p-3 font-mono text-sm text-primary-300 dark:text-surface-600 leading-relaxed min-h-0 selection:bg-surface-200 dark:selection:bg-primary-700 h-full overflow-hidden"
    >
      <div
        v-if="filteredLogLines.length === 0"
        class="flex flex-col items-center justify-center h-full text-zinc-500"
      >
        <p>No log lines streamed or matching query.</p>
      </div>
      <VirtualScroller
        v-else
        :items="filteredLogLines"
        :itemSize="28"
        class="h-full w-full overflow-y-auto"
      >
        <template #item="{ item: line, options }">
          <div
            :style="{ height: options.itemSize + 'px' }"
            class="flex gap-2 hover:bg-surface-100 dark:hover:bg-primary-800 py-0.5 rounded px-1 items-center whitespace-nowrap overflow-x-auto"
          >
            <!-- Timestamps -->
            <span
              v-if="showTimestamps && line.timestamp"
              class="text-zinc-600 select-none shrink-0"
            >
              {{ line.timestamp }}
            </span>

            <!-- Origin Pod/Container Badge -->
            <span class="text-surface-500 font-bold shrink-0 select-none">
              [{{ line.pod.split('-').pop() }}/{{ line.container }}]
            </span>

            <!-- Log Content -->
            <span class="flex-1" :class="getLogLevelColor(line.text)">
              {{ line.text }}
            </span>
          </div>
        </template>
      </VirtualScroller>
    </div>

    <!-- Highlight Rules Config Dialog -->
    <LogHighlightRulesDialog v-model:visible="showRulesDialog" :highlighting="logHighlighting" />
  </div>
</template>
