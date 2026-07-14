<script setup lang="ts">
import { useUpdaterStore } from '@/stores/updater'
import { watch, onMounted } from 'vue'
import { useToast } from 'primevue/usetoast'
import { useConfirm } from 'primevue/useconfirm'
import { app } from '@neutralinojs/lib'
import Button from 'primevue/button'

const updaterStore = useUpdaterStore()
const toast = useToast()
const confirm = useConfirm()

onMounted(() => {
  updaterStore.initListeners()

  // Optionally trigger check on startup:
  // setTimeout(() => updaterStore.checkForUpdates(), 2000)
})

watch(
  () => updaterStore.hasResourcesUpdate,
  (hasUpdate) => {
    if (hasUpdate) {
      toast.add({
        severity: 'info',
        summary: 'Update Available',
        detail: 'A new resource update is available. Click to apply.',
        life: 10000,
        group: 'updater'
      })
    }
  }
)

watch(
  () => updaterStore.hasEngineUpdate,
  (hasUpdate) => {
    if (hasUpdate) {
      confirm.require({
        message: 'A new engine update is available. Would you like to update now?',
        header: 'Engine Update',
        icon: 'pi pi-info-circle',
        acceptLabel: 'Update & Restart',
        rejectLabel: 'Later',
        accept: () => {
          updaterStore.triggerEngineUpdate()
        }
      })
    }
  }
)

watch(
  () => updaterStore.updateReady,
  (component) => {
    if (component === 'resources') {
      toast.add({
        severity: 'success',
        summary: 'Update Applied',
        detail: 'Resources updated successfully. Reloading...',
        life: 3000
      })
      setTimeout(() => {
        window.location.reload()
      }, 1500)
    } else if (component === 'engine') {
      toast.add({
        severity: 'success',
        summary: 'Update Downloaded',
        detail: 'Engine update is ready. Restarting application...',
        life: 3000
      })
      // Exit neutralino so the updater can restart it
      setTimeout(() => {
        app.exit()
      }, 1000)
    }
  }
)

const applyResourceUpdate = () => {
  toast.removeGroup('updater')
  updaterStore.applyResourceUpdate()
  toast.add({
    severity: 'info',
    summary: 'Updating',
    detail: 'Downloading resource update...',
    life: 3000
  })
}
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
        <div class="flex gap-2 mt-2" v-if="updaterStore.hasResourcesUpdate">
          <Button size="small" label="Apply" @click="applyResourceUpdate"></Button>
        </div>
      </div>
    </template>
  </Toast>
</template>
