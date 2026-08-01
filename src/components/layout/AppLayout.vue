<script setup lang="ts">
import { useCluster } from '@/composables/useCluster.ts'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import OfflineClusterView from '@/views/OfflineClusterView.vue'
import WelcomeView from '@/views/WelcomeView.vue'
import { useRoute } from 'vue-router'
import AppHeader from './AppHeader.vue'
import AppLoadingScreen from './AppLoadingScreen.vue'
import AppSidebar from './AppSidebar.vue'

const k8sStore = useKubernetesStore()
const { activeCluster } = useCluster()
const route = useRoute()
</script>

<template>
  <Transition name="page" mode="out-in">
    <AppLoadingScreen v-if="k8sStore.isAppLoading" @complete="k8sStore.setAppLoading(false)" />
    <div v-else class="flex h-screen w-screen overflow-hidden text-(--text-primary) font-sans">
      <!-- Sidebar -->
      <AppSidebar />

      <!-- Main Content Area -->
      <div class="flex-1 flex flex-col h-full overflow-hidden">
        <!-- Header -->
        <AppHeader />

        <main class="bg-(--bg-app) flex-1 overflow-y-auto p-8">
          <template v-if="k8sStore.activeClusterId !== null || route.path === '/settings'">
            <OfflineClusterView
              v-if="
                activeCluster && activeCluster.status !== 'healthy' && route.path !== '/settings'
              "
            />
            <RouterView v-else v-slot="{ Component }">
              <transition name="page" mode="out-in">
                <component :is="Component" />
              </transition>
            </RouterView>
          </template>
          <WelcomeView v-else />
        </main>
      </div>
    </div>
  </Transition>
</template>
