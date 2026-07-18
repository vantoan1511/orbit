<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import WelcomeView from '@/views/WelcomeView.vue'
import AppHeader from './AppHeader.vue'
import AppSidebar from './AppSidebar.vue'

const k8sStore = useKubernetesStore()
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden bg-(--bg-app) text-(--text-primary) font-sans">
    <!-- Sidebar -->
    <AppSidebar />

    <!-- Main Content Area -->
    <div class="flex-1 flex flex-col h-full overflow-hidden">
      <!-- Header -->
      <AppHeader />

      <!-- Scrollable Content -->
      <main class="flex-1 overflow-y-auto p-8">
        <template v-if="k8sStore.activeClusterId !== null">
          <RouterView v-slot="{ Component }">
            <transition name="page" mode="out-in">
              <component :is="Component" />
            </transition>
          </RouterView>
        </template>
        <WelcomeView v-else />
      </main>
    </div>
  </div>
</template>
