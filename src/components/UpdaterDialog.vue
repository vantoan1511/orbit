<script setup lang="ts">
import { useUpdaterStore } from '@/stores/updater'
import { Download, Gift, Loader2, X } from '@lucide/vue'
import { os } from '@/services/nativeService'
import Button from 'primevue/button'
import Dialog from 'primevue/dialog'
import { computed } from 'vue'

const updaterStore = useUpdaterStore()

type NoteSegmentType = 'text' | 'bold' | 'author' | 'pr' | 'link'

interface NoteSegment {
  type: NoteSegmentType
  text: string
  url?: string
}

interface ParsedNoteBlock {
  type: 'header' | 'list-item' | 'empty' | 'paragraph'
  segments: NoteSegment[]
  class: string
}

function parseSegments(text: string): NoteSegment[] {
  const segments: NoteSegment[] = []
  // Matches:
  // 1,2: Bold (**text**)
  // 3,4: PR URL (https://github.com/.../pull/123)
  // 5,6: Compare URL (https://github.com/.../compare/v0.5.1...v0.6.0)
  // 7,8,9: Markdown Link ([label](url))
  // 10: Author (@username)
  // 11: General URL
  const regex =
    /(\*\*([^*]+)\*\*)|(https:\/\/github\.com\/[\w-]+\/[\w-]+\/pull\/(\d+))|(https:\/\/github\.com\/[\w-]+\/[\w-]+\/compare\/([^\s)]+))|(\[([^\]]+)\]\(([^)]+)\))|(@[\w-]+)|(https?:\/\/[^\s)]+)/g

  let lastIndex = 0
  let match: RegExpExecArray | null

  while ((match = regex.exec(text)) !== null) {
    if (match.index > lastIndex) {
      segments.push({ type: 'text', text: text.substring(lastIndex, match.index) })
    }

    if (match[1] && match[2]) {
      segments.push({ type: 'bold', text: match[2] })
    } else if (match[3] && match[4]) {
      segments.push({ type: 'pr', text: '#' + match[4], url: match[3] })
    } else if (match[5] && match[6]) {
      segments.push({ type: 'link', text: match[6], url: match[5] })
    } else if (match[7] && match[8]) {
      segments.push({ type: 'link', text: match[8], url: match[9] })
    } else if (match[10]) {
      segments.push({ type: 'author', text: match[10] })
    } else if (match[11]) {
      segments.push({ type: 'link', text: match[11], url: match[11] })
    }

    lastIndex = regex.lastIndex
  }

  if (lastIndex < text.length) {
    segments.push({ type: 'text', text: text.substring(lastIndex) })
  }

  return segments
}

const parsedNotes = computed<ParsedNoteBlock[]>(() => {
  const notes = updaterStore.manifest?.release_notes
  if (!notes) return []

  const cleanedNotes = notes.replace(/<!--[\s\S]*?-->/g, '')

  return cleanedNotes
    .split('\n')
    .map((line) => {
      const trimmed = line.trim()

      if (trimmed.startsWith('# ') || trimmed.startsWith('## ') || trimmed.startsWith('### ')) {
        return {
          type: 'header' as const,
          segments: [{ type: 'text' as const, text: trimmed.replace(/^#+\s*/, '') }],
          class: 'font-bold text-primary text-xs mt-3 first:mt-0 pb-1 border-b border-(--border)'
        }
      } else if (trimmed.startsWith('- ') || trimmed.startsWith('* ')) {
        const content = trimmed.substring(2).trim()
        return {
          type: 'list-item' as const,
          segments: parseSegments(content),
          class: 'pl-2 text-secondary leading-snug'
        }
      } else if (trimmed.length === 0) {
        return {
          type: 'empty' as const,
          segments: [],
          class: 'h-1.5'
        }
      } else {
        return {
          type: 'paragraph' as const,
          segments: parseSegments(trimmed),
          class: 'text-secondary leading-normal'
        }
      }
    })
    .filter((block, idx, arr) => {
      if (block.type === 'empty') {
        return idx > 0 && arr[idx - 1]?.type !== 'empty' && idx < arr.length - 1
      }
      return true
    })
})

const openUrl = async (url?: string) => {
  if (url) {
    try {
      await os.open(url)
    } catch (e) {
      console.error('Failed to open URL', e)
    }
  }
}
</script>

<template>
  <Dialog
    v-model:visible="updaterStore.showUpdateDialog"
    modal
    header="Software Update"
    :style="{ width: '520px', maxWidth: '92vw' }"
  >
    <div class="flex flex-col gap-4">
      <!-- Version Info Banner -->
      <div class="flex items-center gap-3.5 bg-(--bg-hover)/40 rounded-lg p-3">
        <div
          class="w-10 h-10 bg-(--bg-card) rounded-lg flex items-center justify-center border border-(--border) shrink-0"
        >
          <Gift class="w-5 h-5 text-primary" />
        </div>
        <div class="flex flex-col">
          <div class="flex items-center gap-2">
            <h3 class="text-sm font-bold text-primary">
              Orbit v{{ updaterStore.manifest?.version || '' }}
            </h3>
            <span
              class="px-2 py-0.5 text-[10px] font-semibold rounded-full bg-(--accent-soft) text-(--accent)"
            >
              New Release
            </span>
          </div>
          <p class="text-xs text-muted-color mt-0.5">
            A new version of Orbit is available. Review release notes below to install.
          </p>
        </div>
      </div>

      <!-- Release Notes Section -->
      <div>
        <h4 class="text-xs font-semibold uppercase tracking-wider text-muted-color mb-1.5">
          Release Notes
        </h4>
        <div
          class="bg-(--bg-hover)/40 rounded-lg p-3 max-h-56 overflow-y-auto text-xs flex flex-col gap-1.5"
        >
          <template v-if="parsedNotes.length > 0">
            <div v-for="(block, index) in parsedNotes" :key="index" :class="block.class">
              <template v-if="block.type === 'header'">
                {{ block.segments[0]?.text }}
              </template>
              <template v-else-if="block.type === 'list-item'">
                <div class="flex items-start">
                  <span class="mr-1.5 shrink-0 select-none">•</span>
                  <div class="leading-relaxed">
                    <template v-for="(segment, sIndex) in block.segments" :key="sIndex">
                      <span v-if="segment.type === 'text'">{{ segment.text }}</span>
                      <span v-else-if="segment.type === 'bold'" class="font-semibold text-primary">
                        {{ segment.text }}
                      </span>
                      <span
                        v-else-if="segment.type === 'author'"
                        class="inline-block px-1 rounded border border-(--warning)/20 bg-(--warning-soft) text-(--warning) text-[10px] font-mono leading-tight whitespace-nowrap mx-0.5"
                      >
                        {{ segment.text }}
                      </span>
                      <a
                        v-else-if="segment.type === 'pr' || segment.type === 'link'"
                        href="#"
                        class="inline-block px-1 rounded border border-(--accent)/20 bg-(--accent-soft) text-(--accent) text-[10px] font-mono leading-tight whitespace-nowrap mx-0.5 cursor-pointer hover:bg-(--accent)/20 transition-colors"
                        @click.prevent="openUrl(segment.url)"
                      >
                        {{ segment.text }}
                      </a>
                    </template>
                  </div>
                </div>
              </template>
              <template v-else-if="block.type === 'paragraph'">
                <template v-for="(segment, sIndex) in block.segments" :key="sIndex">
                  <span v-if="segment.type === 'text'">{{ segment.text }}</span>
                  <span v-else-if="segment.type === 'bold'" class="font-semibold text-primary">
                    {{ segment.text }}
                  </span>
                  <span
                    v-else-if="segment.type === 'author'"
                    class="inline-block px-1 rounded border border-(--warning)/20 bg-(--warning-soft) text-(--warning) text-[10px] font-mono leading-tight whitespace-nowrap mx-0.5"
                  >
                    {{ segment.text }}
                  </span>
                  <a
                    v-else-if="segment.type === 'pr' || segment.type === 'link'"
                    href="#"
                    class="inline-block px-1 rounded border border-(--accent)/20 bg-(--accent-soft) text-(--accent) text-[10px] font-mono leading-tight whitespace-nowrap mx-0.5 cursor-pointer hover:bg-(--accent)/20 transition-colors"
                    @click.prevent="openUrl(segment.url)"
                  >
                    {{ segment.text }}
                  </a>
                </template>
              </template>
            </div>
          </template>
          <div v-else class="text-muted-color italic text-xs py-2">
            No release notes provided for this version.
          </div>
        </div>
      </div>

      <!-- Download Progress Bar -->
      <div v-if="updaterStore.isDownloading" class="flex flex-col gap-1.5 pt-1">
        <div class="flex justify-between items-center text-xs">
          <span class="font-medium text-muted-color">Downloading update...</span>
          <span class="text-primary font-mono font-medium"
            >{{ updaterStore.downloadProgress }}%</span
          >
        </div>
        <div class="w-full bg-(--bg-hover) rounded-full h-1.5 overflow-hidden">
          <div
            class="bg-primary h-1.5 rounded-full transition-all duration-300"
            :style="{ width: `${updaterStore.downloadProgress}%` }"
          ></div>
        </div>
      </div>
    </div>

    <!-- Footer Action Buttons -->
    <template #footer>
      <div class="flex justify-between items-center w-full">
        <Button
          severity="secondary"
          variant="text"
          size="small"
          label="Dismiss"
          :disabled="updaterStore.isDownloading"
          @click="updaterStore.showUpdateDialog = false"
        >
          <template #icon>
            <X class="w-3.5 h-3.5 mr-1" />
          </template>
        </Button>
        <Button
          severity="primary"
          size="small"
          :label="
            updaterStore.isDownloading
              ? `Downloading... ${updaterStore.downloadProgress}%`
              : 'Install'
          "
          :disabled="updaterStore.isDownloading"
          @click="updaterStore.applyUpdate()"
        >
          <template #icon>
            <Loader2 v-if="updaterStore.isDownloading" class="w-3.5 h-3.5 mr-1 animate-spin" />
            <Download v-else class="w-3.5 h-3.5 mr-1" />
          </template>
        </Button>
      </div>
    </template>
  </Dialog>
</template>
