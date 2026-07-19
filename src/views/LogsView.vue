<script setup lang="ts">
import { useLogHighlighting } from '@/composables/useLogHighlighting'
import { useLogSelection } from '@/composables/useLogSelection'
import { useLogStream } from '@/composables/useLogStream'
import { ArrowLeft } from '@lucide/vue'
import Button from 'primevue/button'
import Checkbox from 'primevue/checkbox'
import Dialog from 'primevue/dialog'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { useRouter } from 'vue-router'

const router = useRouter()

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

const {
  showRulesDialog,
  selectedPreset,
  customRules,
  newPresetName,
  presetOptions,
  activeRules,
  isCustomPresetActive,
  colorOptions,
  loadRules,
  saveRules,
  addRule,
  deleteCustomRule,
  saveCustomPreset,
  deleteCustomPreset,
  getLogLevelColor
} = useLogHighlighting()

const {
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
      class="bg-(--bg-card) border border-(--border)"
      :style="{ width: '900px', maxWidth: '90vw' }"
    >
      <div class="flex flex-col gap-4">
        <p class="text-xs text-(--text-muted)">
          Define search patterns to style log lines dynamically. Presets are read-only; custom rules
          can be edited/deleted.
        </p>
        <div
          class="flex flex-wrap items-center justify-between gap-4 bg-(--bg-hover)/10 p-3 border border-(--border) rounded-lg"
        >
          <div class="flex items-center gap-3">
            <label class="text-xs font-semibold text-(--text-secondary)">Rule Preset:</label>
            <Select
              v-model="selectedPreset"
              :options="presetOptions"
              optionLabel="label"
              optionValue="value"
              optionGroupLabel="label"
              optionGroupChildren="items"
              class="text-xs min-w-48 bg-(--bg-card) border-(--border)"
              @change="saveRules"
            />
            <Button
              v-if="isCustomPresetActive"
              icon="pi pi-trash"
              severity="danger"
              variant="text"
              size="small"
              v-tooltip="'Delete this custom preset'"
              @click="deleteCustomPreset"
            />
          </div>
          <div class="flex items-center gap-2">
            <InputText
              v-model="newPresetName"
              placeholder="Preset name..."
              class="text-xs bg-(--bg-card) border-(--border) w-40"
            />
            <Button
              label="Save Custom Rules as Preset"
              icon="pi pi-save"
              size="small"
              severity="secondary"
              :disabled="!customRules.length || !newPresetName.trim()"
              @click="saveCustomPreset"
            />
          </div>
        </div>
        <div class="flex flex-col gap-2 max-h-80 overflow-y-auto pr-1">
          <div
            v-for="rule in activeRules"
            :key="rule.id"
            class="flex items-center gap-2 p-2 border rounded-lg"
            :class="
              rule.isPreset
                ? 'bg-(--bg-hover)/5 border-dashed border-(--border-muted) opacity-80'
                : 'bg-(--bg-hover)/20 border-(--border)'
            "
          >
            <InputText
              v-model="rule.pattern"
              placeholder="Pattern..."
              class="text-xs bg-(--bg-card) border-(--border) flex-1"
              :disabled="rule.isPreset"
              @change="saveRules"
            />
            <Select
              v-model="rule.color"
              :options="colorOptions"
              optionLabel="label"
              optionValue="value"
              class="text-xs min-w-28 bg-(--bg-card) border-(--border)"
              :disabled="rule.isPreset"
              @change="saveRules"
            />
            <div class="flex items-center gap-1.5 ml-1">
              <Checkbox
                v-model="rule.bold"
                :inputId="'bold-' + rule.id"
                binary
                class="border-(--border)"
                :disabled="rule.isPreset"
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
                :disabled="rule.isPreset"
                @change="saveRules"
              />
              <label
                :for="'cs-' + rule.id"
                class="text-[10px] uppercase font-bold text-(--text-muted) cursor-pointer select-none"
                >CS</label
              >
            </div>
            <div class="flex items-center gap-1.5 ml-1">
              <Checkbox
                v-model="rule.isRegex"
                :inputId="'rx-' + rule.id"
                binary
                class="border-(--border)"
                :disabled="rule.isPreset"
                @change="saveRules"
              />
              <label
                :for="'rx-' + rule.id"
                class="text-[10px] uppercase font-bold text-(--text-muted) cursor-pointer select-none"
                >Regex</label
              >
            </div>
            <Button
              v-if="!rule.isPreset"
              icon="pi pi-trash"
              severity="danger"
              variant="text"
              size="small"
              @click="deleteCustomRule(rule.id)"
            />
            <span v-else class="text-[10px] uppercase font-bold text-(--text-muted) px-2"
              >Preset</span
            >
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
