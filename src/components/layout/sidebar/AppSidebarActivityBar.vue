<script setup lang="ts">
import { useTheme } from '@/composables/useTheme'
import { os } from '@/services/nativeService'
import { useNotificationStore } from '@/stores/notificationStore'
import { useProfileStore } from '@/stores/profileStore'
import { Bell, Moon, Settings, Sun, User } from '@lucide/vue'
import { Button } from 'primevue'
import { useRoute, useRouter } from 'vue-router'
import { GitHubIcon } from 'vue3-simple-icons'
import { categories, type CategoryId, type SidebarCategory } from './navigation'

defineProps<{
  activeTab: CategoryId | null
}>()

const emit = defineEmits<{
  (e: 'toggleCategory', category: SidebarCategory): void
}>()

const notificationStore = useNotificationStore()
const profileStore = useProfileStore()
const route = useRoute()
const router = useRouter()
const { isDark, toggleTheme } = useTheme()
</script>

<template>
  <div
    class="w-14 flex flex-col items-center py-3 border-r border-(--border) bg-(--bg-sidebar) shrink-0 select-none"
  >
    <!-- Orbit Brand Logo -->
    <div class="mb-4 flex items-center justify-center p-1">
      <img src="/logo.png" alt="Orbit Logo" class="w-7 h-7 object-contain" />
    </div>

    <!-- Main Activity Items (Categories) -->
    <div class="flex flex-col gap-1.5 w-full items-center">
      <Button
        v-for="cat in categories"
        :key="cat.id"
        v-tooltip.right="cat.name"
        variant="text"
        severity="secondary"
        :class="[
          'w-10 h-10',
          activeTab === cat.id
            ? 'bg-(--bg-active) text-(--text-primary)'
            : 'text-muted-color hover:bg-(--bg-hover)'
        ]"
        @click="emit('toggleCategory', cat)"
      >
        <template #icon>
          <component :is="cat.icon" class="w-5 h-5" />
        </template>
      </Button>
    </div>

    <!-- Bottom Actions Spacer -->
    <div class="flex-1"></div>

    <!-- Bottom Activity Actions -->
    <div class="flex flex-col gap-2 items-center w-full px-1">
      <!-- Settings Link -->
      <Button
        v-tooltip.right="'Settings'"
        variant="text"
        severity="secondary"
        :class="[
          'w-10 h-10',
          route.path === '/settings'
            ? 'bg-(--bg-active) text-(--text-primary)'
            : 'text-muted-color hover:bg-(--bg-hover)'
        ]"
        @click="router.push('/settings')"
      >
        <template #icon>
          <Settings class="w-5 h-5" />
        </template>
      </Button>

      <!-- Theme Toggle -->
      <Button
        v-tooltip.right="isDark ? 'Light Mode' : 'Dark Mode'"
        variant="text"
        severity="secondary"
        class="w-10 h-10 text-muted-color hover:bg-(--bg-hover)"
        @click="toggleTheme"
      >
        <template #icon>
          <Sun v-if="isDark" class="w-5 h-5" />
          <Moon v-else class="w-5 h-5" />
        </template>
      </Button>

      <!-- Notifications -->
      <div class="relative inline-flex">
        <Button
          v-tooltip.right="'Notifications'"
          variant="text"
          severity="secondary"
          class="w-10 h-10 text-muted-color hover:bg-(--bg-hover)"
          @click="notificationStore.toggleDrawer()"
        >
          <template #icon>
            <Bell class="w-5 h-5" />
          </template>
        </Button>
        <span
          v-if="notificationStore.unreadCount > 0"
          class="absolute top-2 right-2 w-2 h-2 rounded-full bg-rose-500 pointer-events-none"
        />
      </div>

      <!-- User Profile -->
      <Button
        v-tooltip.right="'User Profile'"
        variant="text"
        severity="secondary"
        class="w-10 h-10 text-muted-color hover:bg-(--bg-hover)"
        @click="profileStore.toggleDrawer()"
      >
        <template #icon>
          <User class="w-5 h-5" />
        </template>
      </Button>

      <!-- GitHub Docs -->
      <Button
        v-tooltip.right="'GitHub Repository'"
        variant="text"
        severity="secondary"
        class="w-10 h-10 text-muted-color hover:bg-(--bg-hover)"
        @click="os.open('https://github.com/vantoan1511/orbit')"
      >
        <template #icon>
          <GitHubIcon :size="20" />
        </template>
      </Button>
    </div>
  </div>
</template>
