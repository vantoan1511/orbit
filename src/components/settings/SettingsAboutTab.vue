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
  <div class="flex flex-col gap-10 max-w-2xl">
    <!-- About Section -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <h4 class="text-xs font-semibold text-primary uppercase tracking-wider">About</h4>
        <p class="text-[11px] text-muted-color">Application details and system metadata.</p>
      </div>

      <div class="w-full md:w-2/3 xl:w-3/4 flex flex-col gap-6">
        <div class="flex items-center gap-4">
          <div
            class="w-12 h-12 bg-(--bg-hover)/60 rounded-xl flex items-center justify-center border border-(--border)"
          >
            <!-- Orbit Logo -->
            <Compass class="w-6 h-6 text-primary" />
          </div>
          <div>
            <h3 class="text-base font-bold text-primary">Orbit</h3>
            <p class="text-xs text-muted-color">The native Kubernetes dashboard</p>
          </div>
        </div>

        <div class="flex flex-col gap-2">
          <div class="flex justify-between items-center py-1.5 text-xs">
            <span class="font-medium text-muted-color">Version</span>
            <span class="text-primary font-mono">{{ appVersion || 'Loading...' }}</span>
          </div>
          <div class="flex justify-between items-center py-1.5 text-xs">
            <span class="font-medium text-muted-color">Architecture</span>
            <span class="text-primary font-mono">x64</span>
          </div>
          <div class="flex justify-between items-center py-1.5 text-xs">
            <span class="font-medium text-muted-color">Author</span>
            <span class="text-primary">vantoan1511</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Updates Section -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <h4 class="text-xs font-semibold text-primary uppercase tracking-wider">Updates</h4>
        <p class="text-[11px] text-muted-color">Check for new releases and patches.</p>
      </div>

      <div class="w-full md:w-2/3 xl:w-3/4 flex flex-col gap-4">
        <div class="flex items-center justify-between gap-4">
          <Button
            @click="updaterStore.checkForUpdates()"
            :loading="updaterStore.isChecking"
            size="small"
            class="text-xs font-medium"
          >
            <RefreshCw class="w-3.5 h-3.5" :class="{ 'animate-spin': updaterStore.isChecking }" />
            <span>Check for Updates</span>
          </Button>

          <div v-if="!updaterStore.isChecking && updaterStore.manifest" class="text-xs">
            <span
              v-if="updaterStore.hasUpdate"
              class="text-emerald-500 font-medium flex items-center gap-1"
            >
              <CheckCircle2 class="w-4 h-4" /> Update Available
            </span>
            <span v-else class="text-muted-color flex items-center gap-1">
              <Check class="w-4 h-4" /> Orbit is up to date
            </span>
          </div>
        </div>

        <!-- Show progress if downloading -->
        <div v-if="updaterStore.isDownloading" class="flex flex-col gap-1.5">
          <div class="flex justify-between text-xs mb-1">
            <span class="text-muted-color">Downloading update...</span>
            <span class="text-primary font-mono">{{ updaterStore.downloadProgress }}%</span>
          </div>
          <div class="w-full bg-(--bg-hover) rounded-full h-1.5 overflow-hidden">
            <div
              class="bg-primary h-1.5 rounded-full transition-all duration-300"
              :style="{ width: `${updaterStore.downloadProgress}%` }"
            ></div>
          </div>
        </div>

        <!-- Show apply actions if ready but not applied -->
        <div
          v-if="!updaterStore.isDownloading && updaterStore.hasUpdate"
          class="p-4 bg-sky-500/10 rounded-lg flex items-start gap-3"
        >
          <Info class="w-4 h-4 text-sky-400 mt-0.5 shrink-0" />
          <div class="flex flex-col gap-2">
            <div>
              <h4 class="text-xs font-semibold text-sky-400">Update Available</h4>
              <p class="text-[11px] text-muted-color mt-0.5">
                Version {{ updaterStore.manifest?.version }} is ready to install. Restart required.
              </p>
            </div>
            <div class="flex gap-2">
              <Button
                size="small"
                severity="secondary"
                variant="outlined"
                label="Release Notes"
                class="text-xs font-medium"
                @click="updaterStore.showUpdateDialog = true"
              />
              <Button
                size="small"
                severity="info"
                label="Update & Restart"
                class="text-xs font-medium"
                @click="updaterStore.applyUpdate()"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
