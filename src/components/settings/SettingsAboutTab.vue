<script setup lang="ts">
import { app } from '@/services/nativeService'
import { useUpdaterStore } from '@/stores/updater'
import { Check, CheckCircle2, Compass, Info, RefreshCw } from '@lucide/vue'
import Button from 'primevue/button'
import { onMounted, ref } from 'vue'

const updaterStore = useUpdaterStore()
const appVersion = ref('')

onMounted(async () => {
  try {
    const config = await app.getConfig()
    appVersion.value = config.version
  } catch (error) {
    console.error('Failed to get app version:', error)
  }
})
</script>

<template>
  <div class="flex flex-col gap-6 max-w-2xl">
    <!-- About Section -->
    <div class="bg-(--bg-card) border border-(--border) rounded-xl p-6">
      <div class="flex items-center gap-4 mb-4">
        <div
          class="w-16 h-16 bg-(--bg-app) rounded-2xl flex items-center justify-center border border-(--border)"
        >
          <!-- Orbit Logo -->
          <Compass class="w-8 h-8 text-primary" />
        </div>
        <div>
          <h3 class="text-xl font-bold text-primary">Orbit</h3>
          <p class="text-sm text-muted-color">The native Kubernetes dashboard</p>
        </div>
      </div>

      <div class="flex flex-col gap-3">
        <div class="flex justify-between items-center py-2 border-b border-(--border)">
          <span class="text-sm font-medium text-muted-color">Version</span>
          <span class="text-sm text-primary">{{ appVersion || 'Loading...' }}</span>
        </div>
        <div class="flex justify-between items-center py-2 border-b border-(--border)">
          <span class="text-sm font-medium text-muted-color">Architecture</span>
          <span class="text-sm text-primary">x64</span>
        </div>
        <div class="flex justify-between items-center py-2">
          <span class="text-sm font-medium text-muted-color">Author</span>
          <span class="text-sm text-primary">vantoan1511</span>
        </div>
      </div>
    </div>

    <!-- Updates Section -->
    <div class="bg-(--bg-card) border border-(--border) rounded-xl p-6 flex flex-col gap-4">
      <div>
        <h3 class="text-lg font-semibold text-primary mb-1">Software Updates</h3>
        <p class="text-sm text-muted-color">
          Check for the latest features, bug fixes, and performance improvements.
        </p>
      </div>

      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Button
            @click="updaterStore.checkForUpdates()"
            :loading="updaterStore.isChecking"
            size="small"
          >
            <RefreshCw class="w-3.5 h-3.5" :class="{ 'animate-spin': updaterStore.isChecking }" />
            <span>Check for Updates</span>
          </Button>
        </div>
        <div v-if="!updaterStore.isChecking && updaterStore.manifest" class="text-sm">
          <span v-if="updaterStore.hasUpdate" class="text-emerald-500 font-medium flex items-center gap-1">
            <CheckCircle2 class="w-4 h-4" /> Update Available
          </span>
          <span v-else class="text-muted-color flex items-center gap-1">
            <Check class="w-4 h-4" /> Orbit is up to date
          </span>
        </div>
      </div>

      <!-- Show progress if downloading -->
      <div v-if="updaterStore.isDownloading" class="mt-4">
        <div class="flex justify-between text-sm mb-1">
          <span class="text-muted-color">Downloading update...</span>
          <span class="text-primary">{{ updaterStore.downloadProgress }}%</span>
        </div>
        <div class="w-full bg-(--bg-app) rounded-full h-1.5 border border-(--border)">
          <div
            class="bg-primary h-1.5 rounded-full transition-all duration-300"
            :style="{ width: `${updaterStore.downloadProgress}%` }"
          ></div>
        </div>
      </div>

      <!-- Show apply actions if ready but not applied -->
      <div
        v-if="!updaterStore.isDownloading && updaterStore.hasUpdate"
        class="mt-4 p-4 bg-blue-500/10 border border-blue-500/20 rounded-lg"
      >
        <div class="flex items-start gap-3">
          <Info class="w-4 h-4 text-blue-500 mt-0.5 shrink-0" />
          <div>
            <h4 class="text-sm font-semibold text-blue-400 mb-1">Update Available</h4>
            <p class="text-xs text-muted-color mb-3">
              Version {{ updaterStore.manifest?.version }} is ready to install. This requires
              restarting Orbit.
            </p>
            <div class="flex gap-2">
              <Button
                size="small"
                severity="secondary"
                variant="outlined"
                label="Release Notes"
                @click="updaterStore.showUpdateDialog = true"
              />
              <Button
                size="small"
                severity="info"
                label="Update & Restart"
                @click="updaterStore.applyUpdate()"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
