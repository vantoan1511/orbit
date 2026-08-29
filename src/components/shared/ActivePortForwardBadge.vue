<script setup lang="ts">
import { kubernetesService } from '@/services/kubernetesService'
import type { ActivePortForward } from '@/types/kubernetes'
import { Cable } from '@lucide/vue'
import Button from 'primevue/button'
import Popover from 'primevue/popover'
import { ref } from 'vue'

const props = defineProps<{
  portForwards: ActivePortForward[]
}>()

const op = ref<InstanceType<typeof Popover> | null>(null)
const stopping = ref<Record<string, boolean>>({})

const toggle = (event: Event) => {
  op.value?.toggle(event)
}

const stopPortForward = async (id: string) => {
  stopping.value[id] = true
  try {
    await kubernetesService.stopPortForward({ id })
  } catch (e) {
    console.error('Failed to stop port forward', e)
  } finally {
    stopping.value[id] = false
  }
}
</script>

<template>
  <div v-if="props.portForwards.length > 0" class="inline-flex items-center">
    <div
      v-tooltip.right="'Active Port Forwards (Click to view/stop)'"
      class="group inline-flex items-center gap-1.5 px-1.5 py-0.5 rounded border border-emerald-500/30 bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 hover:bg-emerald-500/20 cursor-pointer transition-colors"
      @click.stop="toggle"
    >
      <Cable class="w-3 h-3" />
      <span class="text-[10px] font-mono font-semibold">{{ props.portForwards.length }}</span>
    </div>

    <Popover ref="op" class="shadow-lg min-w-64" @click.stop>
      <div class="flex flex-col gap-3 p-1">
        <div class="flex flex-col gap-0.5">
          <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
            Active Port Forwards
          </span>
          <span
            v-if="props.portForwards[0]"
            class="text-xs text-muted-color font-mono truncate max-w-60"
          >
            {{ props.portForwards[0].kind.toLowerCase() }}/{{ props.portForwards[0].name }}
          </span>
        </div>

        <div class="flex flex-col gap-2">
          <div
            v-for="pf in props.portForwards"
            :key="pf.id"
            class="flex items-center justify-between gap-3 p-2 rounded bg-(--bg-hover) border border-(--border)"
          >
            <div class="flex items-center gap-2 font-mono text-xs">
              <span class="text-primary font-semibold">127.0.0.1:{{ pf.localPort }}</span>
              <span class="text-muted-color">→</span>
              <span class="text-muted-color">{{ pf.remotePort }}</span>
            </div>
            <Button
              v-tooltip.right="'Stop Port Forward'"
              icon="pi pi-stop"
              severity="danger"
              variant="text"
              size="small"
              class="w-6 h-6 p-0 shrink-0"
              :loading="stopping[pf.id]"
              @click.stop="stopPortForward(pf.id)"
            />
          </div>
        </div>
      </div>
    </Popover>
  </div>
</template>
