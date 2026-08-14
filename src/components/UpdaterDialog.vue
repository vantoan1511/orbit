<script setup lang="ts">
import { useUpdaterStore } from '@/stores/updater'
import { Download, Gift, Loader2, X } from '@lucide/vue'
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
        class: 'font-bold text-primary text-base mt-3 first:mt-0 pb-1 border-b border-(--border)'
      }
    } else if (trimmed.startsWith('- ') || trimmed.startsWith('* ')) {
      return {
        text: '• ' + trimmed.substring(2),
        class: 'pl-2 text-secondary leading-snug'
      }
    } else if (trimmed.length === 0) {
      return {
        text: '',
        class: 'h-1.5'
      }
    } else {
      return {
        text: trimmed,
        class: 'text-secondary leading-normal'
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
      <div class="flex items-start gap-4 pb-4 border-b border-(--border)">
        <div
          class="w-12 h-12 rounded-xl bg-blue-500/10 border border-blue-500/20 flex items-center justify-center shrink-0"
        >
          <Gift class="w-6 h-6 text-blue-500" />
        </div>
        <div class="flex flex-col">
          <div class="flex items-center gap-2">
            <h3 class="text-lg font-bold text-primary">
              Orbit v{{ updaterStore.manifest?.version || '' }}
            </h3>
            <span
              class="px-2 py-0.5 text-xs font-semibold rounded-full bg-blue-500/15 text-blue-500 border border-blue-500/20"
            >
              New Release
            </span>
          </div>
          <p class="text-xs text-muted-color mt-0.5">
            A new version of Orbit is available. Review the release notes below to install or
            dismiss.
          </p>
        </div>
      </div>

      <!-- Release Notes Section -->
      <div>
        <h4 class="text-xs font-semibold uppercase tracking-wider text-muted-color mb-2">
          Release Notes
        </h4>
        <div
          class="bg-(--bg-hover)/40 border border-(--border) rounded-xl p-4 max-h-64 overflow-y-auto text-sm flex flex-col gap-1.5"
        >
          <template v-if="parsedNotes.length > 0">
            <div v-for="(block, index) in parsedNotes" :key="index" :class="block.class">
              {{ block.text }}
            </div>
          </template>
          <div v-else class="text-muted-color italic text-xs py-2">
            No release notes provided for this version.
          </div>
        </div>
      </div>

      <!-- Download Progress Bar -->
      <div v-if="updaterStore.isDownloading" class="pt-2 border-t border-(--border)">
        <div class="flex justify-between items-center text-xs mb-1.5">
          <span class="font-medium text-muted-color">Downloading update...</span>
          <span class="font-bold text-blue-500">{{ updaterStore.downloadProgress }}%</span>
        </div>
        <div
          class="w-full bg-(--bg-hover) rounded-full h-2 border border-(--border-strong) overflow-hidden"
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
          severity="secondary"
          variant="text"
          size="small"
          :disabled="updaterStore.isDownloading"
          @click="updaterStore.showUpdateDialog = false"
        >
          <X class="w-3.5 h-3.5 mr-1" />
          <span>Dismiss</span>
        </Button>
        <Button
          severity="primary"
          size="small"
          :disabled="updaterStore.isDownloading"
          @click="updaterStore.applyUpdate()"
        >
          <Loader2 v-if="updaterStore.isDownloading" class="w-3.5 h-3.5 mr-1 animate-spin" />
          <Download v-else class="w-3.5 h-3.5 mr-1" />
          <span>
            {{
              updaterStore.isDownloading
                ? `Downloading... ${updaterStore.downloadProgress}%`
                : 'Install'
            }}
          </span>
        </Button>
      </div>
    </template>
  </Dialog>
</template>
