<script setup lang="ts">
import HighlightedText from '@/components/shared/HighlightedText.vue'
import { useSearchStore } from '@/stores/searchStore'
import type { SearchCategory } from '@/types/search'
import { Search, X } from '@lucide/vue'
import Button from 'primevue/button'
import Dialog from 'primevue/dialog'
import InputText from 'primevue/inputtext'
import Tag from 'primevue/tag'
import { nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const searchStore = useSearchStore()

const searchInputRef = ref<
  { $el?: HTMLInputElement; focus?: () => void; select?: () => void } | HTMLInputElement | null
>(null)
const resultItemRefs = ref<HTMLElement[]>([])
const scrollContainerRef = ref<HTMLElement | null>(null)

const isMac =
  typeof navigator !== 'undefined' &&
  navigator.platform &&
  navigator.platform.toUpperCase().indexOf('MAC') >= 0

const categories: Array<{ id: SearchCategory; label: string }> = [
  { id: 'all', label: 'All' },
  { id: 'resources', label: 'Resources' },
  { id: 'logs', label: 'Logs' },
  { id: 'navigation', label: 'Navigation' }
]

function cycleCategory(reverse = false) {
  const currentIndex = categories.findIndex((c) => c.id === searchStore.activeCategory)
  if (currentIndex === -1) return
  const nextIndex = reverse
    ? (currentIndex - 1 + categories.length) % categories.length
    : (currentIndex + 1) % categories.length
  const nextCat = categories[nextIndex]
  if (nextCat) {
    searchStore.setCategory(nextCat.id)
  }
}

function handleKeydown(e: KeyboardEvent) {
  const isModifier = isMac ? e.metaKey : e.ctrlKey

  // Ctrl+Shift+P or Cmd+Shift+P
  if (isModifier && e.shiftKey && (e.key === 'P' || e.key === 'p')) {
    e.preventDefault()
    e.stopPropagation()
    searchStore.toggle()
    return
  }

  // Ctrl+P or Cmd+P (Quick Open alias)
  if (isModifier && !e.shiftKey && (e.key === 'P' || e.key === 'p')) {
    e.preventDefault()
    e.stopPropagation()
    searchStore.toggle()
    return
  }

  // Intercept keys when search dialog is open
  if (searchStore.isOpen) {
    if (e.key === 'Escape') {
      e.preventDefault()
      e.stopPropagation()
      searchStore.close()
      return
    }

    if (e.key === 'ArrowDown') {
      e.preventDefault()
      searchStore.selectNext()
      return
    }

    if (e.key === 'ArrowUp') {
      e.preventDefault()
      searchStore.selectPrev()
      return
    }

    if (e.key === 'Enter') {
      if (e.isComposing) return
      e.preventDefault()
      searchStore.executeSelected()
      return
    }

    if (e.key === 'Tab') {
      e.preventDefault()
      cycleCategory(e.shiftKey)
      return
    }
  }
}

function clearSearch() {
  searchStore.searchQuery = ''
  searchStore.selectedIndex = 0
}

// When dialog opens, automatically focus and select search input
watch(
  () => searchStore.isOpen,
  async (open) => {
    if (open) {
      await nextTick()
      const target = searchInputRef.value
      const el =
        target && typeof target === 'object' && '$el' in target
          ? (target.$el as HTMLInputElement | undefined)
          : (target as HTMLInputElement | undefined)
      if (el && typeof el.focus === 'function') {
        el.focus()
        if (typeof el.select === 'function') {
          el.select()
        }
      }
    }
  }
)

// Scroll active item into view as selectedIndex changes
watch(
  () => searchStore.selectedIndex,
  (idx) => {
    const el = resultItemRefs.value[idx]
    if (el && typeof el.scrollIntoView === 'function') {
      el.scrollIntoView({ block: 'nearest' })
    }
  }
)

// Reset stale item refs and scroll position when query or category changes
watch(
  () => [searchStore.searchQuery, searchStore.activeCategory],
  () => {
    resultItemRefs.value = []
    if (scrollContainerRef.value) {
      scrollContainerRef.value.scrollTop = 0
    }
  }
)

const setResultItemRef = (el: unknown, index: number) => {
  if (el instanceof HTMLElement) {
    resultItemRefs.value[index] = el
  } else if (
    el &&
    typeof el === 'object' &&
    '$el' in el &&
    (el as { $el: unknown }).$el instanceof HTMLElement
  ) {
    resultItemRefs.value[index] = (el as { $el: HTMLElement }).$el
  }
}

onMounted(() => {
  searchStore.setNavigationHandler((target) => {
    void router.push(target)
  })
  window.addEventListener('keydown', handleKeydown, true)
})

onUnmounted(() => {
  searchStore.setNavigationHandler(null)
  window.removeEventListener('keydown', handleKeydown, true)
})

const getStatusSeverity = (status?: string): 'success' | 'warn' | 'danger' | 'info' => {
  if (!status) return 'info'
  const s = status.toLowerCase()
  if (s === 'running' || s === 'ready' || s === 'active' || s === 'completed' || s === 'healthy') {
    return 'success'
  }
  if (s === 'pending' || s === 'waiting' || s === 'suspended' || s === 'terminating') {
    return 'warn'
  }
  if (s === 'failed' || s === 'crashloopbackoff' || s === 'error' || s === 'evicted') {
    return 'danger'
  }
  return 'info'
}

const categoryLabels: Record<string, string> = {
  resources: 'Resources',
  logs: 'Logs',
  navigation: 'Navigation'
}

function shouldShowCategoryHeader(idx: number): boolean {
  if (searchStore.activeCategory !== 'all') return false
  const results = searchStore.filteredResults
  if (idx === 0) return true
  return results[idx]?.category !== results[idx - 1]?.category
}
</script>

<template>
  <Dialog
    v-model:visible="searchStore.isOpen"
    modal
    dismissableMask
    closeOnEscape
    lazy
    :showHeader="false"
    position="top"
    class="search-everywhere-modal !border-none"
    :style="{ width: '640px', maxWidth: '94vw', marginTop: '10vh' }"
    :pt="{
      mask: { class: 'bg-black/50 backdrop-blur-[2px]' },
      content: {
        class:
          'p-0 overflow-hidden rounded-lg border border-(--border) bg-(--bg-card) shadow-(--shadow)'
      }
    }"
  >
    <div class="flex flex-col select-none text-sm">
      <!-- Search Input Bar -->
      <div class="flex items-center gap-3 px-4 py-3 border-b border-(--border) bg-(--bg-card)">
        <Search
          class="w-4 h-4 shrink-0 transition-colors"
          :class="searchStore.searchQuery ? 'text-primary' : 'text-muted-color'"
        />
        <InputText
          ref="searchInputRef"
          v-model="searchStore.searchQuery"
          type="text"
          placeholder="Search resources, logs, navigation..."
          class="flex-1 border-0 shadow-none bg-transparent p-0 text-sm font-medium text-primary focus:ring-0 placeholder:text-muted-color/60 outline-none"
          @input="searchStore.selectedIndex = 0"
        />
        <Button
          v-if="searchStore.searchQuery"
          type="button"
          variant="text"
          severity="secondary"
          size="small"
          rounded
          class="w-6! h-6! p-0! text-muted-color hover:text-primary"
          v-tooltip.top="'Clear search'"
          aria-label="Clear search"
          @click="clearSearch"
        >
          <X class="w-3.5 h-3.5" />
        </Button>
        <kbd
          class="px-1.5 py-0.5 text-[10px] font-mono font-semibold bg-(--bg-hover) text-muted-color rounded border border-(--border)"
        >
          ESC
        </kbd>
      </div>

      <!-- Category Filter Pills -->
      <div
        class="flex items-center gap-1.5 px-3 py-1.5 border-b border-(--border) bg-(--bg-hover)/30 text-xs overflow-x-auto"
      >
        <Button
          v-for="cat in categories"
          :key="cat.id"
          type="button"
          unstyled
          class="flex items-center gap-1.5 px-2.5 py-1 rounded-md transition-colors text-xs font-medium cursor-pointer select-none"
          :class="[
            searchStore.activeCategory === cat.id
              ? 'bg-primary text-slate-50 dark:text-slate-950 shadow-xs font-semibold'
              : 'text-muted-color hover:text-primary hover:bg-(--bg-hover)'
          ]"
          @click="searchStore.setCategory(cat.id)"
        >
          <span>{{ cat.label }}</span>
          <span
            class="text-[10px] px-1.5 py-0.5 rounded-full font-mono font-semibold"
            :class="[
              searchStore.activeCategory === cat.id
                ? 'bg-white/20 text-slate-50 dark:bg-black/15 dark:text-slate-950'
                : 'bg-(--bg-hover) text-muted-color'
            ]"
          >
            {{ searchStore.categoryCounts[cat.id] }}
          </span>
        </Button>
      </div>

      <!-- Results Scroll Area -->
      <div ref="scrollContainerRef" class="max-h-96 overflow-y-auto p-1.5 flex flex-col gap-0.5">
        <!-- Empty State -->
        <div
          v-if="searchStore.filteredResults.length === 0"
          class="flex flex-col items-center justify-center p-8 text-center text-muted-color"
        >
          <Search class="w-8 h-8 mb-2 text-muted-color/40" />
          <p class="text-sm font-semibold text-primary">No results found</p>
          <p class="text-xs mt-1 max-w-xs text-muted-color leading-relaxed">
            No matching resources, logs, or views found for
            <span class="text-primary font-medium">"{{ searchStore.searchQuery }}"</span>
          </p>
          <Button
            v-if="searchStore.activeCategory !== 'all'"
            label="Search in All Categories"
            variant="text"
            severity="secondary"
            size="small"
            class="mt-3 text-xs"
            @click="searchStore.setCategory('all')"
          />
        </div>

        <!-- Result Item Rows with Category Headers -->
        <template v-for="(item, idx) in searchStore.filteredResults" :key="item.id">
          <!-- Category Eyebrow Header -->
          <div
            v-if="shouldShowCategoryHeader(idx)"
            class="px-3 pt-2.5 pb-1 text-[10px] font-semibold font-mono tracking-wider text-muted-color uppercase select-none"
          >
            {{ categoryLabels[item.category] || item.category }}
          </div>

          <!-- Result Row -->
          <div
            :ref="(el) => setResultItemRef(el, idx)"
            class="flex items-center gap-3 px-3 py-2 rounded-md cursor-pointer transition-colors group select-none"
            :class="[
              idx === searchStore.selectedIndex
                ? 'bg-(--bg-hover) text-primary font-medium'
                : 'text-secondary hover:bg-(--bg-hover)/60'
            ]"
            @mouseenter="searchStore.selectedIndex = idx"
            @click="searchStore.executeSelected()"
          >
            <!-- Kind / Category Icon -->
            <div
              class="w-7 h-7 rounded-md flex items-center justify-center shrink-0 bg-(--bg-hover)/50 group-hover:bg-(--bg-hover) transition-colors"
            >
              <component
                :is="item.icon"
                v-if="item.icon"
                class="w-4 h-4 shrink-0"
                :class="item.iconColorClass || 'text-primary'"
              />
            </div>

            <!-- Content Details -->
            <div class="flex flex-col min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <span class="text-xs font-semibold text-primary truncate">
                  <HighlightedText :text="item.title" :query="searchStore.searchQuery" />
                </span>
                <Tag
                  v-if="item.status"
                  :value="item.status"
                  :severity="getStatusSeverity(item.status)"
                  class="text-[10px] px-1.5 py-0 h-4 uppercase font-semibold"
                />
              </div>
              <div
                v-if="item.subtitle"
                class="text-[11px] text-muted-color truncate font-mono mt-0.5"
              >
                <HighlightedText :text="item.subtitle" :query="searchStore.searchQuery" />
              </div>
            </div>

            <!-- Category Badge & Action Hint -->
            <div class="flex items-center gap-2 shrink-0">
              <span
                class="text-[10px] font-mono text-muted-color uppercase tracking-wider px-1.5 py-0.5 rounded bg-(--bg-hover)/50"
              >
                {{ item.category }}
              </span>
              <kbd
                v-if="idx === searchStore.selectedIndex"
                class="hidden sm:inline-flex items-center px-1.5 py-0.5 text-[10px] font-mono font-semibold bg-(--bg-card) text-muted-color rounded border border-(--border) shadow-xs"
              >
                ↵
              </kbd>
            </div>
          </div>
        </template>
      </div>

      <!-- Footer Keyboard Shortcut Bar -->
      <div
        class="flex items-center justify-between px-3.5 py-2 border-t border-(--border) bg-(--bg-card) text-[11px] text-muted-color select-none"
      >
        <div class="flex items-center gap-3">
          <span class="flex items-center gap-1">
            <kbd class="px-1 py-0.5 font-mono bg-(--bg-hover) rounded border border-(--border)"
              >↑</kbd
            >
            <kbd class="px-1 py-0.5 font-mono bg-(--bg-hover) rounded border border-(--border)"
              >↓</kbd
            >
            <span>Navigate</span>
          </span>
          <span class="flex items-center gap-1">
            <kbd class="px-1 py-0.5 font-mono bg-(--bg-hover) rounded border border-(--border)"
              >↵</kbd
            >
            <span>Open</span>
          </span>
          <span class="flex items-center gap-1">
            <kbd class="px-1 py-0.5 font-mono bg-(--bg-hover) rounded border border-(--border)"
              >Tab</kbd
            >
            <span>Category</span>
          </span>
          <span class="flex items-center gap-1">
            <kbd class="px-1 py-0.5 font-mono bg-(--bg-hover) rounded border border-(--border)"
              >Esc</kbd
            >
            <span>Close</span>
          </span>
        </div>

        <div class="hidden sm:flex items-center gap-1 text-[10px] font-mono">
          <span
            >{{ searchStore.filteredResults.length }}
            {{ searchStore.filteredResults.length === 1 ? 'result' : 'results' }}</span
          >
        </div>
      </div>
    </div>
  </Dialog>
</template>
