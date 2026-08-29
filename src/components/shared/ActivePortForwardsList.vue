<script setup lang="ts">
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { Cable } from '@lucide/vue'
import Button from 'primevue/button'
import { computed, ref } from 'vue'

const props = defineProps<{
  kind: string
  namespace: string
  name: string
}>()

const k8sStore = useKubernetesStore()

const portForwards = computed(() => {
  if (!props.name) return []
  return k8sStore.activePortForwards.filter(
    (pf) =>
      pf.kind.toLowerCase() === props.kind.toLowerCase() &&
      (!props.namespace || !pf.namespace || pf.namespace === props.namespace) &&
      pf.name === props.name
  )
})

const stopping = ref<Record<string, boolean>>({})

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
  <div v-if="portForwards.length > 0">
    <div class="text-[10px] font-bold text-muted-color uppercase tracking-wider mb-2.5">
      Active Port Forwards
    </div>
    <div class="flex flex-col gap-2">
      <div
        v-for="pf in portForwards"
        :key="pf.id"
        class="flex items-center justify-between gap-3 p-3 rounded-xl border border-emerald-500/30 bg-emerald-500/10 text-xs"
      >
        <div class="flex items-center gap-2.5 font-mono">
          <Cable class="w-4 h-4 text-emerald-600 dark:text-emerald-400 shrink-0" />
          <div class="flex items-center gap-1.5">
            <span class="text-primary font-semibold">127.0.0.1:{{ pf.localPort }}</span>
            <span class="text-muted-color">→</span>
            <span class="text-muted-color">{{ pf.remotePort }}</span>
          </div>
        </div>
        <Button
          v-tooltip.top="'Stop Port Forward'"
          icon="pi pi-stop"
          severity="danger"
          variant="text"
          size="small"
          class="w-6 h-6 p-0 shrink-0"
          :loading="stopping[pf.id]"
          @click="stopPortForward(pf.id)"
        />
      </div>
    </div>
  </div>
</template>
