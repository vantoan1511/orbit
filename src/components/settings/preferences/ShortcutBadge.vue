<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useShortcutStore } from '@/stores/shortcutStore'
import type { ShortcutActionId } from '@/types/shortcuts'
import { eventToKeyCombo, formatKeyCombo } from '@/utils/shortcutHelpers'
import Button from 'primevue/button'

const props = defineProps<{
  actionId: ShortcutActionId
}>()

const emit = defineEmits<{
  (e: 'rebind', combo: string): void
}>()

const shortcutStore = useShortcutStore()
const badgeRef = ref<HTMLElement | null>(null)

const isMac =
  typeof navigator !== 'undefined' &&
  Boolean(navigator.platform && navigator.platform.toUpperCase().includes('MAC'))

const isCurrentListening = computed(() => {
  return shortcutStore.isListening && shortcutStore.listeningActionId === props.actionId
})

const displayKey = computed(() => {
  return shortcutStore.getDisplayShortcut(props.actionId)
})

function handleClick() {
  if (isCurrentListening.value) {
    shortcutStore.stopListening()
  } else {
    shortcutStore.startListening(props.actionId)
  }
}

function handleKeyDown(e: KeyboardEvent) {
  if (!isCurrentListening.value) return

  e.preventDefault()
  e.stopPropagation()

  if (e.key === 'Escape') {
    shortcutStore.stopListening()
    return
  }

  const combo = eventToKeyCombo(e)
  if (!combo) return

  const formatted = formatKeyCombo(combo, isMac)
  if (formatted) {
    emit('rebind', formatted)
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown, true)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown, true)
})
</script>

<template>
  <div class="inline-flex items-center">
    <Button
      ref="badgeRef"
      type="button"
      variant="text"
      size="small"
      class="group relative inline-flex items-center justify-center font-mono text-xs px-2.5 py-1 rounded-[var(--ui-radius-control)] transition-all duration-150 border cursor-pointer select-none"
      :class="[
        isCurrentListening
          ? 'bg-(--accent)/15 border-(--accent) text-(--accent) ring-2 ring-(--accent)/20 animate-pulse'
          : 'bg-(--bg-panel) hover:bg-(--bg-hover) border-(--border) text-primary'
      ]"
      v-tooltip.top="
        isCurrentListening ? 'Press key combination (Esc to cancel)' : 'Click to rebind shortcut'
      "
      @click="handleClick"
    >
      <span v-if="isCurrentListening" class="flex items-center gap-1.5 font-medium">
        <span class="w-1.5 h-1.5 rounded-full bg-(--accent) animate-ping" />
        Recording...
      </span>
      <span v-else class="font-medium tracking-wide">
        {{ displayKey }}
      </span>
    </Button>
  </div>
</template>
