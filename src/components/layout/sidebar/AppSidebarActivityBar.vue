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
        :severity="activeTab === cat.id ? 'primary' : 'secondary'"
        :class="[
          activeTab === cat.id
            ? 'border-l-3! border-l-primary-900 dark:border-l-primary-50 rounded-none!'
            : 'text-muted-color'
        ]"
        v-tooltip.right="cat.name"
        variant="text"
        class="w-full!"
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
        rounded
        variant="text"
        :class="[
          'w-10 h-10 flex! items-center! justify-center!',
          route.path === '/settings'
            ? 'bg-primary-100! dark:bg-primary-900/40! text-primary-600! dark:text-primary-400!'
            : 'text-muted-color'
        ]"
        @click="router.push('/settings')"
      >
        <Settings class="w-5 h-5" />
      </Button>

      <!-- Theme Toggle -->
      <Button
        v-tooltip.right="isDark ? 'Light Mode' : 'Dark Mode'"
        rounded
        variant="text"
        class="w-10 h-10 text-muted-color flex! items-center! justify-center!"
        @click="toggleTheme"
      >
        <Sun v-if="isDark" class="w-5 h-5" />
        <Moon v-else class="w-5 h-5" />
      </Button>

      <!-- Notifications -->
      <div class="relative inline-flex">
        <Button
          v-tooltip.right="'Notifications'"
          rounded
          variant="text"
          class="w-10 h-10 text-muted-color flex! items-center! justify-center!"
          badge-severity="danger"
          :badge="
            notificationStore.unreadCount > 0 ? notificationStore.unreadCount.toString() : undefined
          "
          @click="notificationStore.toggleDrawer()"
        >
          <Bell class="w-5 h-5" />
        </Button>
      </div>

      <!-- User Profile -->
      <Button
        v-tooltip.right="'User Profile'"
        rounded
        variant="text"
        class="w-10 h-10 text-muted-color flex! items-center! justify-center!"
        @click="profileStore.toggleDrawer()"
      >
        <User class="w-5 h-5" />
      </Button>

      <!-- GitHub Docs -->
      <Button
        v-tooltip.right="'GitHub Repository'"
        rounded
        variant="text"
        class="w-10 h-10 text-muted-color flex! items-center! justify-center!"
        @click="os.open('https://github.com/vantoan1511/orbit')"
      >
        <GitHubIcon :size="20" />
      </Button>
    </div>
  </div>
</template>
