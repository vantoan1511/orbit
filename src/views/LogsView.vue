<script setup lang="ts">
import LogHighlightRulesDialog from '@/components/logs/LogHighlightRulesDialog.vue'
import HighlightedText from '@/components/shared/HighlightedText.vue'
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
  tailLinesOptions,
  podOptions,
  containerOptions
} = useLogSelection()

watch(
  [
    selectedNamespace,
    selectedWorkloadKind,
    selectedWorkloadName,
    selectedPodName,
    selectedContainerName
  ],
  ([ns, kind, workload, pod, container]) => {
    if (ns && workload) {
      logsStore.addRecentLog({
        namespace: ns,
        workloadKind: kind,
        workloadName: workload,
        pod: pod || 'All',
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
  virtualScrollerRef,
  isAtBottom,
  onScroll,
  scrollToBottom,
  clearLogs,
  downloadLogs,
  copyLogs,
  isCopied
} = useLogStream({
  selectedNamespace,
  selectedWorkloadName,
  selectedWorkloadKind,
  selectedPodName,
  selectedContainerName,
  tailLines,
  onMountedCallback: loadRules
})

// Fixed row height for VirtualScroller; must stay in sync with item padding/font.
const LOG_ITEM_HEIGHT = 28
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
    <div class="flex flex-col gap-2.5">
      <!-- Row 1: Context Selection & Actions -->
      <div class="flex items-center justify-between gap-4 flex-wrap">
        <div class="flex items-center gap-4 flex-wrap">
          <div class="flex items-center gap-2 shrink-0">
            <span
              class="text-[10px] font-semibold tracking-wider text-muted-color uppercase select-none"
              >Pod</span
            >
            <Select
              v-model="selectedPodName"
              :options="podOptions"
              variant="filled"
              size="small"
              class="text-xs min-w-32 max-w-56"
            />
          </div>
          <div class="flex items-center gap-2 shrink-0">
            <span
              class="text-[10px] font-semibold tracking-wider text-muted-color uppercase select-none"
              >Container</span
            >
            <Select
              v-model="selectedContainerName"
              :options="containerOptions"
              variant="filled"
              size="small"
              class="text-xs min-w-32 max-w-56"
            />
          </div>
          <div class="flex items-center gap-2 shrink-0">
            <span
              class="text-[10px] font-semibold tracking-wider text-muted-color uppercase select-none"
              >Lines</span
            >
            <Select
              v-model="tailLines"
              :options="tailLinesOptions"
              optionLabel="label"
              optionValue="value"
              variant="filled"
              size="small"
              class="text-xs min-w-28"
            />
          </div>
        </div>

        <div class="flex items-center gap-1">
          <Button
            severity="secondary"
            variant="text"
            size="small"
            class="p-1! w-7! h-7!"
            :icon="isPaused ? 'pi pi-play' : 'pi pi-pause'"
            :title="isPaused ? 'Resume Stream' : 'Pause Stream'"
            @click="isPaused = !isPaused"
          />
          <Button
            severity="secondary"
            icon="pi pi-trash"
            size="small"
            variant="text"
            class="p-1! w-7! h-7!"
            title="Clear Logs"
            @click="clearLogs"
          />
          <div class="w-px h-3.5 bg-(--border) mx-1"></div>
          <Button
            severity="secondary"
            icon="pi pi-palette"
            size="small"
            variant="text"
            class="p-1! w-7! h-7!"
            title="Highlight Rules"
            @click="showRulesDialog = true"
          />
          <Button
            severity="secondary"
            :icon="isCopied ? 'pi pi-check' : 'pi pi-copy'"
            size="small"
            variant="text"
            class="p-1! w-7! h-7!"
            :disabled="logLines.length <= 0"
            title="Copy Logs"
            @click="copyLogs"
          />
          <Button
            severity="secondary"
            icon="pi pi-download"
            size="small"
            variant="text"
            class="p-1! w-7! h-7!"
            :disabled="logLines.length <= 0"
            title="Download Logs"
            @click="downloadLogs"
          />
          <Button
            severity="secondary"
            size="small"
            variant="text"
            class="p-1! w-7! h-7!"
            :icon="isFullscreen ? 'pi pi-window-minimize' : 'pi pi-window-maximize'"
            :title="isFullscreen ? 'Exit Fullscreen' : 'Fullscreen'"
            @click="isFullscreen = !isFullscreen"
          />
        </div>
      </div>

      <!-- Row 2: Search & Display Toggles -->
      <div class="flex items-center justify-between gap-4 flex-wrap">
        <div class="flex items-center gap-3 flex-1 min-w-64 max-w-xl">
          <IconField class="flex-1">
            <InputIcon class="pi pi-search" />
            <InputText
              v-model="searchQuery"
              placeholder="Search logs..."
              variant="filled"
              size="small"
              fluid
              class="text-xs"
            />
          </IconField>
          <div class="flex items-center gap-1.5 shrink-0">
            <Checkbox v-model="isRegex" inputId="is-regex" binary class="border-surface" />
            <label for="is-regex" class="text-xs text-muted-color cursor-pointer select-none"
              >Regex</label
            >
          </div>
        </div>

        <div class="flex items-center gap-4 shrink-0">
          <div class="flex items-center gap-1.5 shrink-0">
            <Checkbox v-model="showTimestamps" inputId="show-timestamps" binary />
            <label for="show-timestamps" class="text-xs text-muted-color cursor-pointer select-none"
              >Timestamps</label
            >
          </div>
          <div class="flex items-center gap-1.5 shrink-0">
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
        </div>
      </div>
    </div>

    <!-- Console Viewer -->
    <div
      class="flex-1 dark:bg-zinc-950 rounded p-3 font-mono text-sm text-primary-300 dark:text-surface-600 leading-relaxed min-h-0 selection:bg-surface-200 dark:selection:bg-primary-700 h-full overflow-hidden relative"
    >
      <div
        v-if="filteredLogLines.length === 0"
        class="flex flex-col items-center justify-center h-full text-zinc-500"
      >
        <p>No log lines streamed or matching query.</p>
      </div>
      <VirtualScroller
        v-else
        ref="virtualScrollerRef"
        :items="filteredLogLines"
        :itemSize="LOG_ITEM_HEIGHT"
        class="h-full w-full"
        @scroll="onScroll"
      >
        <template #item="{ item: line, options }">
          <div
            :style="{ height: options.itemSize + 'px' }"
            class="flex gap-2 hover:bg-surface-100 dark:hover:bg-primary-800 py-0.5 rounded px-1 items-center whitespace-nowrap overflow-hidden"
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
              <HighlightedText :text="line.text" :query="searchQuery" :is-regex="isRegex" />
            </span>
          </div>
        </template>
      </VirtualScroller>

      <!-- Scroll to Bottom Button -->
      <Transition
        enter-active-class="transition duration-200 ease-out"
        enter-from-class="opacity-0 translate-y-2"
        enter-to-class="opacity-100 translate-y-0"
        leave-active-class="transition duration-150 ease-in"
        leave-from-class="opacity-100 translate-y-0"
        leave-to-class="opacity-0 translate-y-2"
      >
        <Button
          v-if="!isAtBottom && filteredLogLines.length > 0"
          icon="pi pi-arrow-down"
          rounded
          class="absolute bottom-6 left-1/2 -translate-x-1/2 shadow-lg z-10"
          size="small"
          severity="secondary"
          title="Scroll to bottom"
          @click="scrollToBottom"
        />
      </Transition>
    </div>

    <!-- Highlight Rules Config Dialog -->
    <LogHighlightRulesDialog v-model:visible="showRulesDialog" :highlighting="logHighlighting" />
  </div>
</template>
