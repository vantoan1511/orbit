<script setup lang="ts">
import { useNotificationStore } from '@/stores/notificationStore'
import type { NotificationSeverity } from '@/types/notification'
import {
  AlertTriangle,
  BellOff,
  Check,
  CheckCheck,
  CircleCheck,
  Info,
  Trash2,
  XCircle
} from '@lucide/vue'
import Button from 'primevue/button'
import Drawer from 'primevue/drawer'
import Tab from 'primevue/tab'
import TabList from 'primevue/tablist'
import TabPanel from 'primevue/tabpanel'
import TabPanels from 'primevue/tabpanels'
import Tabs from 'primevue/tabs'
import { computed, ref } from 'vue'

const notificationStore = useNotificationStore()
const activeTab = ref('all')

const filteredNotifications = computed(() => {
  if (activeTab.value === 'unread') {
    return notificationStore.sortedNotifications.filter((n) => !n.read)
  }
  return notificationStore.sortedNotifications
})

const getSeverityIcon = (severity: NotificationSeverity) => {
  switch (severity) {
    case 'success':
      return CircleCheck
    case 'error':
      return XCircle
    case 'warn':
      return AlertTriangle
    case 'info':
    default:
      return Info
  }
}

const getSeverityColorClass = (severity: NotificationSeverity) => {
  switch (severity) {
    case 'success':
      return 'text-emerald-500 bg-emerald-500/10 border-emerald-500/20'
    case 'error':
      return 'text-rose-500 bg-rose-500/10 border-rose-500/20'
    case 'warn':
      return 'text-amber-500 bg-amber-500/10 border-amber-500/20'
    case 'info':
    default:
      return 'text-sky-400 bg-sky-500/10 border-sky-500/20'
  }
}

const formatRelativeTime = (timestamp: number): string => {
  const diffMs = Date.now() - timestamp
  const seconds = Math.floor(diffMs / 1000)
  if (seconds < 60) return 'just now'
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes}m ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  return `${days}d ago`
}
</script>

<template>
  <Drawer
    v-model:visible="notificationStore.isDrawerOpen"
    position="right"
    class="w-full sm:max-w-md border-l border-(--border) bg-(--bg-card) p-0"
    :style="{ width: '28rem' }"
  >
    <template #header>
      <div class="flex items-center justify-between w-full pr-2">
        <div class="flex items-center gap-2">
          <span class="font-bold text-lg text-(--text-primary) font-ui">Notifications</span>
          <span
            v-if="notificationStore.unreadCount > 0"
            class="text-xs font-semibold px-2 py-0.5 rounded-full bg-rose-500/10 text-rose-500 border border-rose-500/20"
          >
            {{ notificationStore.unreadCount }} new
          </span>
        </div>
      </div>
    </template>

    <div class="flex flex-col h-full">
      <!-- Toolbar Header Actions -->
      <div
        class="px-6 py-2 border-b border-(--border) flex items-center justify-between bg-(--bg-sidebar)/50"
      >
        <div class="flex items-center gap-2">
          <Button
            size="small"
            variant="text"
            severity="secondary"
            class="text-xs font-medium"
            :disabled="notificationStore.unreadCount === 0"
            @click="notificationStore.markAllAsRead"
          >
            <CheckCheck class="w-3.5 h-3.5 mr-1 text-(--text-muted)" />
            <span>Mark all read</span>
          </Button>
        </div>

        <Button
          size="small"
          variant="text"
          severity="danger"
          class="text-xs font-medium"
          :disabled="notificationStore.notifications.length === 0"
          @click="notificationStore.clearAll"
        >
          <Trash2 class="w-3.5 h-3.5 mr-1" />
          <span>Clear all</span>
        </Button>
      </div>

      <!-- Filter Tabs -->
      <Tabs v-model:value="activeTab" class="flex-1 flex flex-col min-h-0">
        <TabList class="px-6 border-b border-(--border)">
          <Tab value="all" class="px-4 py-2 text-xs font-semibold">
            All ({{ notificationStore.notifications.length }})
          </Tab>
          <Tab value="unread" class="px-4 py-2 text-xs font-semibold">
            Unread ({{ notificationStore.unreadCount }})
          </Tab>
        </TabList>

        <TabPanels class="flex-1 overflow-y-auto p-4 bg-transparent border-none">
          <TabPanel value="all" class="p-0">
            <!-- Empty State -->
            <div
              v-if="filteredNotifications.length === 0"
              class="flex flex-col items-center justify-center p-12 text-center text-(--text-muted)"
            >
              <BellOff class="w-10 h-10 mb-3 opacity-40" />
              <p class="text-sm font-semibold text-(--text-primary)">No notifications</p>
              <p class="text-xs mt-1 max-w-xs">
                When background events, errors, or command updates occur, they will appear here.
              </p>
            </div>

            <!-- List -->
            <div v-else class="flex flex-col gap-2.5">
              <div
                v-for="item in filteredNotifications"
                :key="item.id"
                class="group relative flex items-start gap-3 p-3.5 rounded-xl border transition-all duration-200 cursor-pointer select-none"
                :class="[
                  item.read
                    ? 'bg-(--bg-card) border-(--border) opacity-80 hover:opacity-100 hover:bg-(--bg-hover)/60'
                    : 'bg-(--bg-hover) border-(--primary)/30 shadow-sm'
                ]"
                @click="notificationStore.markAsRead(item.id)"
              >
                <!-- Severity Icon Badge -->
                <div
                  class="p-2 rounded-lg border shrink-0 flex items-center justify-center mt-0.5"
                  :class="getSeverityColorClass(item.severity)"
                >
                  <component :is="getSeverityIcon(item.severity)" class="w-4 h-4" />
                </div>

                <!-- Text Content -->
                <div class="flex-1 min-w-0">
                  <div class="flex items-center justify-between gap-2">
                    <div class="flex items-center gap-1.5 min-w-0">
                      <span
                        v-if="!item.read"
                        class="w-2 h-2 rounded-full bg-rose-500 shrink-0"
                        title="Unread"
                      ></span>
                      <span class="text-xs font-bold text-(--text-primary) truncate">
                        {{ item.title }}
                      </span>
                    </div>

                    <div class="flex items-center gap-1 shrink-0">
                      <span class="text-[10px] font-medium text-(--text-muted)">
                        {{ formatRelativeTime(item.timestamp) }}
                      </span>
                      <Button
                        icon="pi pi-times"
                        variant="text"
                        rounded
                        size="small"
                        class="w-5 h-5 p-0 text-(--text-muted) hover:text-rose-400 opacity-0 group-hover:opacity-100 transition-opacity ml-1"
                        @click.stop="notificationStore.removeNotification(item.id)"
                      />
                    </div>
                  </div>

                  <p class="text-xs text-(--text-secondary) mt-1 wrap-break-word leading-relaxed">
                    {{ item.message }}
                  </p>
                </div>
              </div>
            </div>
          </TabPanel>

          <TabPanel value="unread" class="p-0">
            <!-- Empty State for Unread -->
            <div
              v-if="filteredNotifications.length === 0"
              class="flex flex-col items-center justify-center p-12 text-center text-(--text-muted)"
            >
              <Check class="w-10 h-10 mb-3 opacity-40 text-emerald-500" />
              <p class="text-sm font-semibold text-(--text-primary)">All caught up!</p>
              <p class="text-xs mt-1">You have no unread notifications.</p>
            </div>

            <!-- Unread List -->
            <div v-else class="flex flex-col gap-2.5">
              <div
                v-for="item in filteredNotifications"
                :key="item.id"
                class="group relative flex items-start gap-3 p-3.5 rounded-xl border bg-(--bg-hover) border-(--border)/30 transition-all duration-200 cursor-pointer select-none"
                @click="notificationStore.markAsRead(item.id)"
              >
                <!-- Severity Icon Badge -->
                <div
                  class="p-2 rounded-lg border shrink-0 flex items-center justify-center mt-0.5"
                  :class="getSeverityColorClass(item.severity)"
                >
                  <component :is="getSeverityIcon(item.severity)" class="w-4 h-4" />
                </div>

                <!-- Text Content -->
                <div class="flex-1 min-w-0">
                  <div class="flex items-center justify-between gap-2">
                    <div class="flex items-center gap-1.5 min-w-0">
                      <span class="w-2 h-2 rounded-full bg-rose-500 shrink-0" title="Unread"></span>
                      <span class="text-xs font-bold text-(--text-primary) truncate">
                        {{ item.title }}
                      </span>
                    </div>

                    <div class="flex items-center gap-1 shrink-0">
                      <span class="text-[10px] font-medium text-(--text-muted)">
                        {{ formatRelativeTime(item.timestamp) }}
                      </span>
                      <Button
                        icon="pi pi-times"
                        variant="text"
                        rounded
                        size="small"
                        class="w-5 h-5 p-0 text-(--text-muted) hover:text-rose-400 opacity-0 group-hover:opacity-100 transition-opacity ml-1"
                        @click.stop="notificationStore.removeNotification(item.id)"
                      />
                    </div>
                  </div>

                  <p class="text-xs text-(--text-secondary) mt-1 wrap-break-word leading-relaxed">
                    {{ item.message }}
                  </p>
                </div>
              </div>
            </div>
          </TabPanel>
        </TabPanels>
      </Tabs>
    </div>
  </Drawer>
</template>
