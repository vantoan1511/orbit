<script setup lang="ts">
import { useUpdaterStore } from '@/stores/updater'
import Button from 'primevue/button'

const updaterStore = useUpdaterStore()
</script>

<template>
  <div class="flex flex-col gap-6 max-w-2xl">
    <!-- About Section -->
    <div class="bg-(--surface-card) border border-(--border) rounded-xl p-6">
      <div class="flex items-center gap-4 mb-4">
        <div
          class="w-16 h-16 bg-(--surface-ground) rounded-2xl flex items-center justify-center border border-(--border)"
        >
          <!-- Orbit Logo Placeholder -->
          <i class="pi pi-compass text-3xl text-(--primary-500)"></i>
        </div>
        <div>
          <h3 class="text-xl font-bold text-(--text-primary)">Orbit</h3>
          <p class="text-sm text-(--text-muted)">The native Kubernetes dashboard</p>
        </div>
      </div>

      <div class="flex flex-col gap-3">
        <div class="flex justify-between items-center py-2 border-b border-(--border)">
          <span class="text-sm font-medium text-(--text-secondary)">Version</span>
          <span class="text-sm text-(--text-primary)">1.0.0</span>
        </div>
        <div class="flex justify-between items-center py-2 border-b border-(--border)">
          <span class="text-sm font-medium text-(--text-secondary)">Architecture</span>
          <span class="text-sm text-(--text-primary)">x64</span>
        </div>
        <div class="flex justify-between items-center py-2">
          <span class="text-sm font-medium text-(--text-secondary)">Author</span>
          <span class="text-sm text-(--text-primary)">vantoan1511</span>
        </div>
      </div>
    </div>

    <!-- Updates Section -->
    <div class="bg-(--surface-card) border border-(--border) rounded-xl p-6 flex flex-col gap-4">
      <div>
        <h3 class="text-lg font-semibold text-(--text-primary) mb-1">Software Updates</h3>
        <p class="text-sm text-(--text-muted)">
          Check for the latest features, bug fixes, and performance improvements.
        </p>
      </div>

      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Button
            @click="updaterStore.checkForUpdates()"
            :loading="updaterStore.isChecking"
            icon="pi pi-refresh"
            label="Check for Updates"
            size="small"
          />
        </div>
        <div v-if="!updaterStore.isChecking && updaterStore.manifest" class="text-sm">
          <span v-if="updaterStore.hasUpdate" class="text-green-500 font-medium">
            <i class="pi pi-check-circle mr-1"></i> Update Available
          </span>
          <span v-else class="text-(--text-muted)">
            <i class="pi pi-check mr-1"></i> Orbit is up to date
          </span>
        </div>
      </div>

      <!-- Show progress if downloading -->
      <div v-if="updaterStore.isDownloading" class="mt-4">
        <div class="flex justify-between text-sm mb-1">
          <span class="text-(--text-secondary)">Downloading update...</span>
          <span class="text-(--text-primary)">{{ updaterStore.downloadProgress }}%</span>
        </div>
        <div class="w-full bg-(--surface-ground) rounded-full h-1.5 border border-(--border)">
          <div
            class="bg-(--primary-500) h-1.5 rounded-full transition-all duration-300"
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
          <i class="pi pi-info-circle text-blue-500 mt-0.5"></i>
          <div>
            <h4 class="text-sm font-semibold text-blue-400 mb-1">Update Available</h4>
            <p class="text-xs text-(--text-muted) mb-3">
              Version {{ updaterStore.manifest?.version }} is ready to install. This requires
              restarting Orbit.
            </p>
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
</template>
