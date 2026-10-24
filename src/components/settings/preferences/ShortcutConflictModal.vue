<script setup lang="ts">
import { computed } from 'vue'
import Dialog from 'primevue/dialog'
import Button from 'primevue/button'
import { useShortcutStore } from '@/stores/shortcutStore'
import { AlertTriangle } from '@lucide/vue'

const shortcutStore = useShortcutStore()

const visible = computed({
  get: () => Boolean(shortcutStore.pendingConflict),
  set: (val: boolean) => {
    if (!val) {
      shortcutStore.cancelConflict()
    }
  }
})

const actionName = computed(() => {
  if (!shortcutStore.pendingConflict) return ''
  const def = shortcutStore.definitions.find(
    (d) => d.id === shortcutStore.pendingConflict?.actionId
  )
  return def?.name || shortcutStore.pendingConflict.actionId
})

const conflictingActionName = computed(() => {
  if (!shortcutStore.pendingConflict) return ''
  const def = shortcutStore.definitions.find(
    (d) => d.id === shortcutStore.pendingConflict?.conflictingActionId
  )
  return def?.name || shortcutStore.pendingConflict.conflictingActionId
})

const targetKey = computed(() => shortcutStore.pendingConflict?.targetKey || '')

async function handleConfirm() {
  await shortcutStore.confirmConflictSwap()
}

function handleCancel() {
  shortcutStore.cancelConflict()
}
</script>

<template>
  <Dialog
    v-model:visible="visible"
    modal
    header="Shortcut Conflict"
    :style="{ width: '420px' }"
    :closable="true"
    @hide="handleCancel"
  >
    <div class="flex items-start gap-3 py-2">
      <div class="p-2 rounded-full bg-(--warning-soft) text-(--warning) shrink-0">
        <AlertTriangle class="w-5 h-5" />
      </div>
      <div class="flex flex-col gap-1.5 text-xs">
        <p class="font-medium text-primary">
          <span class="font-mono px-1.5 py-0.5 rounded bg-(--bg-panel) border border-(--border)">{{
            targetKey
          }}</span>
          is already assigned to
          <span class="font-semibold text-primary">{{ conflictingActionName }}</span
          >.
        </p>
        <p class="text-muted-color">
          Assigning it to <span class="font-semibold text-primary">{{ actionName }}</span> will swap
          their shortcuts. Would you like to proceed?
        </p>
      </div>
    </div>

    <template #footer>
      <div class="flex justify-end gap-2 pt-2">
        <Button
          type="button"
          label="Cancel"
          severity="secondary"
          variant="text"
          size="small"
          @click="handleCancel"
        />
        <Button
          type="button"
          label="Swap & Assign"
          severity="primary"
          size="small"
          @click="handleConfirm"
        />
      </div>
    </template>
  </Dialog>
</template>
