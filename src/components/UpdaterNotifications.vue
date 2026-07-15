<script setup lang="ts">
import { useUpdaterStore } from '@/stores/updater'
import { watch, onMounted, ref } from 'vue'
import { useToast } from 'primevue/usetoast'
import { app } from '@neutralinojs/lib'
import Button from 'primevue/button'
import Dialog from 'primevue/dialog'

const updaterStore = useUpdaterStore()
const toast = useToast()

const showRestartDialog = ref(false)
const countdown = ref(3)

onMounted(() => {
  updaterStore.initListeners()

  // Trigger check on startup:
  setTimeout(() => updaterStore.checkForUpdates(), 2000)
})

watch(
  () => updaterStore.hasUpdate,
  (hasUpdate) => {
    if (hasUpdate) {
      toast.add({
        severity: 'info',
        summary: 'Update Available',
        detail: 'A new version of Orbit is available.',
        group: 'updater',
        life: 15000
      })
    }
  }
)

watch(
  () => updaterStore.updateReady,
  (component) => {
    if (component === 'app') {
      showRestartDialog.value = true
      const interval = setInterval(() => {
        countdown.value--
        if (countdown.value <= 0) {
          clearInterval(interval)
          app.exit()
        }
      }, 1000)
    }
  }
)
</script>

<template>
  <Toast position="bottom-right" group="updater">
    <template #message="slotProps">
      <div class="flex flex-col items-start w-full">
        <div class="flex items-center gap-2">
          <i class="pi pi-download"></i>
          <span class="font-bold">{{ slotProps.message.summary }}</span>
        </div>
        <div class="font-medium text-sm my-2">{{ slotProps.message.detail }}</div>
        <div class="flex gap-2 mt-2" v-if="updaterStore.hasUpdate">
          <Button
            size="small"
            :label="
              updaterStore.isDownloading
                ? `Downloading... ${updaterStore.downloadProgress}%`
                : 'Apply & Restart'
            "
            :disabled="updaterStore.isDownloading"
            @click="updaterStore.applyUpdate()"
          ></Button>
        </div>
      </div>
    </template>
  </Toast>

  <Dialog v-model:visible="showRestartDialog" modal header="Update Ready" :closable="false" :style="{ width: '25rem' }">
    <div class="flex items-center gap-4 mb-4">
      <i class="pi pi-sync text-4xl text-primary"></i>
      <span class="m-0">Orbit has been successfully updated. Restarting in {{ countdown }} seconds...</span>
    </div>
  </Dialog>
</template>
