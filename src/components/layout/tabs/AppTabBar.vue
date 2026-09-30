<script setup lang="ts">
import { useTabDragDrop } from '@/composables/useTabDragDrop'
import { useTabsStore } from '@/stores/tabsStore'
import type { AppTab } from '@/types/tabs'
import { Plus } from '@lucide/vue'
import Button from 'primevue/button'
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import AppTabItem from './AppTabItem.vue'
import AppTabResourcePicker from './AppTabResourcePicker.vue'

const tabsStore = useTabsStore()
const router = useRouter()

const resourcePickerRef = ref<InstanceType<typeof AppTabResourcePicker> | null>(null)
const tabsContainerRef = ref<HTMLElement | null>(null)

const {
  isDraggingTab,
  getDropIndicator,
  handleDragStart,
  handleDragOver,
  handleDragLeave,
  handleDrop,
  handleDragEnd
} = useTabDragDrop({
  onReorder: (from, to) => tabsStore.reorderTabs(from, to)
})

const handleWheel = (e: WheelEvent) => {
  if (tabsContainerRef.value && e.deltaY !== 0) {
    tabsContainerRef.value.scrollLeft += e.deltaY
    e.preventDefault()
  }
}

const handleSelectTab = (tab: AppTab) => {
  tabsStore.setActiveTab(tab.id)
  void router.push(tab.route)
}

const handleMoveTabLeft = (tabId: string) => {
  tabsStore.moveTabLeft(tabId)
}

const handleMoveTabRight = (tabId: string) => {
  tabsStore.moveTabRight(tabId)
}

const handleCloseTab = (tabId: string) => {
  const wasActive = tabsStore.activeTabId === tabId
  tabsStore.closeTab(tabId)
  if (wasActive) {
    if (tabsStore.activeTab) {
      void router.push(tabsStore.activeTab.route)
    }
  }
}

const handleCloseOtherTabs = (tabId: string) => {
  tabsStore.closeOtherTabs(tabId)
  if (tabsStore.activeTab) {
    void router.push(tabsStore.activeTab.route)
  }
}

const handleCloseTabsToTheRight = (tabId: string) => {
  tabsStore.closeTabsToTheRight(tabId)
  if (tabsStore.activeTab) {
    void router.push(tabsStore.activeTab.route)
  }
}

const handleCloseAllTabs = () => {
  tabsStore.closeAllTabs()
}

const handleNewTabClick = (event: MouseEvent) => {
  resourcePickerRef.value?.toggle(event)
}

const handleResourceSelected = (payload: {
  route: string
  title: string
  iconName?: string
  category: string
}) => {
  const tab = tabsStore.openTab(payload)
  void router.push(tab.route)
}
</script>

<template>
  <div
    class="flex items-center select-none bg-(--bg-sidebar) h-8 text-xs shrink-0 z-10 w-full overflow-hidden border-b border-(--border)"
  >
    <!-- Scrollable Tab Items Strip -->
    <div
      ref="tabsContainerRef"
      class="flex items-center flex-1 h-full overflow-x-auto overflow-y-hidden no-scrollbar"
      @wheel="handleWheel"
    >
      <AppTabItem
        v-for="(tab, index) in tabsStore.tabs"
        :key="tab.id"
        :tab="tab"
        :index="index"
        :total-tabs="tabsStore.tabs.length"
        :is-active="tab.id === tabsStore.activeTabId"
        :is-dragging="isDraggingTab(tab.id)"
        :drop-indicator="getDropIndicator(index)"
        @select="handleSelectTab(tab)"
        @close="handleCloseTab(tab.id)"
        @close-others="handleCloseOtherTabs(tab.id)"
        @close-right="handleCloseTabsToTheRight(tab.id)"
        @close-all="handleCloseAllTabs"
        @move-left="handleMoveTabLeft(tab.id)"
        @move-right="handleMoveTabRight(tab.id)"
        @dragstart="handleDragStart(index, tab.id, $event)"
        @dragover="handleDragOver(index, $event)"
        @dragleave="handleDragLeave(index, $event)"
        @drop="handleDrop(index, $event)"
        @dragend="handleDragEnd"
      />

      <!-- New Tab (+) Button -->
      <div class="px-1.5 shrink-0 flex items-center">
        <Button
          variant="text"
          rounded
          size="small"
          class="w-6! h-6! p-0! text-muted-color hover:text-primary hover:bg-(--bg-hover)! cursor-pointer"
          :aria-label="'New Tab'"
          v-tooltip.bottom="'New Tab'"
          @click="handleNewTabClick"
        >
          <template #icon>
            <Plus class="w-3.5 h-3.5" />
          </template>
        </Button>
      </div>
    </div>

    <!-- Resource Picker Popover -->
    <AppTabResourcePicker ref="resourcePickerRef" @select-resource="handleResourceSelected" />
  </div>
</template>

<style scoped>
.no-scrollbar::-webkit-scrollbar {
  display: none;
}
.no-scrollbar {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
