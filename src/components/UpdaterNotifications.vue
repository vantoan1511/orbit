<script setup lang="ts">
import { useUpdaterStore } from '@/stores/updater'
import { watch, onMounted } from 'vue'
import { useToast } from 'primevue/usetoast'
import { app } from '@neutralinojs/lib'
import Button from 'primevue/button'

const updaterStore = useUpdaterStore()
const toast = useToast()

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
      toast.add({
        severity: 'success',
        summary: 'Update Downloaded',
        detail: 'Update is ready. Restarting application...',
        life: 3000
      })
      // Exit neutralino so the updater can restart it
      setTimeout(() => {
        app.exit()
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
          <Button size="small" label="Apply & Restart" @click="updaterStore.applyUpdate()"></Button>
        </div>
      </div>
    </template>
  </Toast>
</template>
