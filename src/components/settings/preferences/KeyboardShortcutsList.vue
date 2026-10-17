<script setup lang="ts">
import { computed, ref } from 'vue'
import { useShortcutStore } from '@/stores/shortcutStore'
import type { ShortcutActionId, ShortcutCategory } from '@/types/shortcuts'
import ShortcutBadge from './ShortcutBadge.vue'
import ShortcutConflictModal from './ShortcutConflictModal.vue'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'
import { RotateCcw, Search } from '@lucide/vue'
import { useToast } from 'primevue/usetoast'

const shortcutStore = useShortcutStore()
const toast = useToast()
const searchQuery = ref('')

const categories: ShortcutCategory[] = ['Navigation', 'Tabs', 'Workspace', 'Tools']

const filteredDefinitions = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) return shortcutStore.definitions

  return shortcutStore.definitions.filter((def) => {
    const binding = shortcutStore.getDisplayShortcut(def.id).toLowerCase()
    return (
      def.name.toLowerCase().includes(query) ||
      def.description.toLowerCase().includes(query) ||
      def.category.toLowerCase().includes(query) ||
      binding.includes(query)
    )
  })
})

const groupedDefinitions = computed(() => {
  const map = new Map<ShortcutCategory, typeof shortcutStore.definitions>()
  for (const cat of categories) {
    const items = filteredDefinitions.value.filter((d) => d.category === cat)
    if (items.length > 0) {
      map.set(cat, items)
    }
  }
  return map
})

async function handleRebind(actionId: ShortcutActionId, combo: string) {
  const def = shortcutStore.definitions.find((d) => d.id === actionId)
  const result = await shortcutStore.rebind(actionId, combo)

  if (result.success) {
    toast.add({
      severity: 'success',
      summary: 'Shortcut Updated',
      detail: `Bound ${def?.name || actionId} to ${combo}`,
      life: 2500
    })
  }
}

async function handleResetItem(actionId: ShortcutActionId) {
  const def = shortcutStore.definitions.find((d) => d.id === actionId)
  await shortcutStore.resetAction(actionId)
  toast.add({
    severity: 'info',
    summary: 'Shortcut Reset',
    detail: `Restored default for ${def?.name || actionId}`,
    life: 2000
  })
}
</script>

<template>
  <div class="flex flex-col gap-5">
    <!-- Filter Search Box -->
    <div class="relative w-full max-w-sm">
      <Search
        class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-color pointer-events-none"
      />
      <InputText
        v-model="searchQuery"
        placeholder="Filter shortcuts by name, action or key..."
        size="small"
        class="w-full pl-8 text-xs bg-(--bg-card)"
      />
    </div>

    <!-- Empty State when filtering -->
    <div
      v-if="groupedDefinitions.size === 0"
      class="p-6 text-center text-xs text-muted-color rounded-lg bg-(--bg-hover)/30"
    >
      No keyboard shortcuts match "{{ searchQuery }}".
    </div>

    <!-- Grouped Sections -->
    <div v-else class="flex flex-col gap-6">
      <div
        v-for="[cat, items] in groupedDefinitions.entries()"
        :key="cat"
        class="flex flex-col gap-2"
      >
        <span class="text-[11px] font-semibold text-muted-color uppercase tracking-wider px-1">
          {{ cat }}
        </span>

        <div class="flex flex-col rounded-lg bg-(--bg-card) overflow-hidden">
          <div
            v-for="(def, idx) in items"
            :key="def.id"
            class="flex items-center justify-between px-3 py-2.5 transition-colors hover:bg-(--bg-hover)/50"
            :class="[idx !== items.length - 1 ? 'border-b border-(--border)/60' : '']"
          >
            <!-- Action Title & Description -->
            <div class="flex flex-col gap-0.5 pr-4">
              <span class="text-xs font-medium text-primary">{{ def.name }}</span>
              <span class="text-[11px] text-muted-color leading-tight">{{ def.description }}</span>
            </div>

            <!-- Controls: Shortcut Badge & Reset -->
            <div class="flex items-center gap-2 shrink-0">
              <ShortcutBadge :action-id="def.id" @rebind="(combo) => handleRebind(def.id, combo)" />

              <Button
                v-if="shortcutStore.isModified(def.id)"
                type="button"
                severity="secondary"
                variant="text"
                size="small"
                class="w-7 h-7 p-0 text-muted-color hover:text-primary"
                v-tooltip.top="'Reset to default'"
                @click="handleResetItem(def.id)"
              >
                <template #icon>
                  <RotateCcw class="w-3.5 h-3.5" />
                </template>
              </Button>
              <div v-else class="w-7 h-7" />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Conflict Swap Modal -->
    <ShortcutConflictModal />
  </div>
</template>
