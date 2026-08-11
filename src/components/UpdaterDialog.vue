<script setup lang="ts">
import { useUpdaterStore } from '@/stores/updater'
import Button from 'primevue/button'
import Dialog from 'primevue/dialog'
import { computed } from 'vue'

const updaterStore = useUpdaterStore()

interface ParsedNoteBlock {
  text: string
  class: string
}

const parsedNotes = computed<ParsedNoteBlock[]>(() => {
  const notes = updaterStore.manifest?.release_notes
  if (!notes) return []

  return notes.split('\n').map((line) => {
    let trimmed = line.trim()
    // Strip standard markdown links [label](url) -> label
    trimmed = trimmed.replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')

    if (trimmed.startsWith('# ') || trimmed.startsWith('## ') || trimmed.startsWith('### ')) {
      return {
        text: trimmed.replace(/^#+\s*/, ''),
        class:
          'font-bold text-slate-900 dark:text-slate-100 text-base mt-3 first:mt-0 pb-1 border-b border-slate-200 dark:border-zinc-800'
      }
    } else if (trimmed.startsWith('- ') || trimmed.startsWith('* ')) {
      return {
        text: '• ' + trimmed.substring(2),
        class: 'pl-2 text-slate-700 dark:text-slate-300 leading-snug'
      }
    } else if (trimmed.length === 0) {
      return {
        text: '',
        class: 'h-1.5'
      }
    } else {
      return {
        text: trimmed,
        class: 'text-slate-700 dark:text-slate-300 leading-normal'
      }
    }
  })
})
</script>

<template>
  <Dialog
    v-model:visible="updaterStore.showUpdateDialog"
    modal
    header="Software Update"
    :style="{ width: '560px', maxWidth: '90vw' }"
  >
    <div class="flex flex-col gap-4">
      <!-- Header / Version Info -->
      <div class="flex items-start gap-4 pb-4 border-b border-slate-200 dark:border-zinc-800">
        <div
          class="w-12 h-12 rounded-xl bg-blue-500/10 border border-blue-500/20 flex items-center justify-center shrink-0"
        >
          <i class="pi pi-gift text-2xl text-blue-500"></i>
        </div>
        <div class="flex flex-col">
          <div class="flex items-center gap-2">
            <h3 class="text-lg font-bold text-slate-900 dark:text-slate-100">
              Orbit v{{ updaterStore.manifest?.version || '' }}
            </h3>
            <span
              class="px-2 py-0.5 text-xs font-semibold rounded-full bg-blue-500/15 text-blue-500 border border-blue-500/20"
            >
              New Release
            </span>
          </div>
          <p class="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
            A new version of Orbit is available. Review the release notes below to install or
            dismiss.
          </p>
        </div>
      </div>

      <!-- Release Notes Section -->
      <div>
        <h4
          class="text-xs font-semibold uppercase tracking-wider text-slate-500 dark:text-slate-400 mb-2"
        >
          Release Notes
        </h4>
        <div
          class="bg-slate-100/70 dark:bg-zinc-900/60 border border-slate-200 dark:border-zinc-800 rounded-xl p-4 max-h-64 overflow-y-auto text-sm flex flex-col gap-1.5"
        >
          <template v-if="parsedNotes.length > 0">
            <div v-for="(block, index) in parsedNotes" :key="index" :class="block.class">
              {{ block.text }}
            </div>
          </template>
          <div v-else class="text-slate-500 dark:text-slate-400 italic text-xs py-2">
            No release notes provided for this version.
          </div>
        </div>
      </div>

      <!-- Download Progress Bar -->
      <div
        v-if="updaterStore.isDownloading"
        class="pt-2 border-t border-slate-200 dark:border-zinc-800"
      >
        <div class="flex justify-between items-center text-xs mb-1.5">
          <span class="font-medium text-slate-500 dark:text-slate-400">Downloading update...</span>
          <span class="font-bold text-blue-500">{{ updaterStore.downloadProgress }}%</span>
        </div>
        <div
          class="w-full bg-slate-200 dark:bg-zinc-800 rounded-full h-2 border border-slate-300 dark:border-zinc-700 overflow-hidden"
        >
          <div
            class="bg-blue-500 h-full rounded-full transition-all duration-300"
            :style="{ width: `${updaterStore.downloadProgress}%` }"
          ></div>
        </div>
      </div>
    </div>

    <!-- Footer Action Buttons -->
    <template #footer>
      <div class="flex justify-between items-center w-full pt-2">
        <Button
          label="Dismiss"
          icon="pi pi-times"
          severity="secondary"
          variant="text"
          size="small"
          :disabled="updaterStore.isDownloading"
          @click="updaterStore.showUpdateDialog = false"
        />
        <Button
          :label="
            updaterStore.isDownloading
              ? `Downloading... ${updaterStore.downloadProgress}%`
              : 'Install'
          "
          :icon="updaterStore.isDownloading ? 'pi pi-spinner pi-spin' : 'pi pi-download'"
          severity="primary"
          size="small"
          :disabled="updaterStore.isDownloading"
          @click="updaterStore.applyUpdate()"
        />
      </div>
    </template>
  </Dialog>
</template>
