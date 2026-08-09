<script setup lang="ts">
import { useCluster } from '@/composables/useCluster.ts'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import OfflineClusterView from '@/views/OfflineClusterView.vue'
import WelcomeView from '@/views/WelcomeView.vue'
import { useRoute } from 'vue-router'
import AppFooter from './AppFooter.vue'
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
    <div v-else class="flex flex-col h-screen w-screen overflow-hidden text-primary font-sans">
      <div class="flex-1 flex overflow-hidden">
        <!-- Sidebar -->
        <AppSidebar />

        <!-- Main Content Area -->
        <div class="flex-1 flex flex-col h-full overflow-hidden">
          <AppHeader />
          <main class="flex-1 h-full overflow-y-auto relative">
            <div class="p-8">
              <template v-if="k8sStore.activeClusterId !== null || route.path === '/settings'">
                <OfflineClusterView
                  v-if="
                    activeCluster &&
                    activeCluster.status !== 'healthy' &&
                    route.path !== '/settings'
                  "
                />
                <RouterView v-else v-slot="{ Component }">
                  <transition name="page" mode="out-in">
                    <component :is="Component" />
                  </transition>
                </RouterView>
              </template>
              <WelcomeView v-else />
            </div>
          </main>
        </div>
      </div>

      <!-- Footer -->
      <AppFooter />
    </div>
  </Transition>
</template>
