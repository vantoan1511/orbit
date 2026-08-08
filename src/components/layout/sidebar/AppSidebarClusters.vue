<script setup lang="ts">
import { useCluster } from '@/composables/useCluster'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { Check, Plus } from '@lucide/vue'
import { Button } from 'primevue'

const emit = defineEmits<{
  (e: 'clusterSwitched'): void
}>()

const k8sStore = useKubernetesStore()
const { activeCluster, isRefreshing, handleAddCluster } = useCluster()

const handleSwitchCluster = async (clusterId: string) => {
  await kubernetesService.switchCluster(clusterId)
  emit('clusterSwitched')
}
</script>

<template>
  <div class="flex-1 overflow-y-auto p-3 flex flex-col gap-2">
    <div class="flex flex-col gap-1.5 flex-1">
      <Button
        v-for="cluster in k8sStore.clusters"
        :key="cluster.id"
        :loading="k8sStore.activeClusterId === cluster.id && isRefreshing"
        :severity="
          k8sStore.activeClusterId === cluster.id
            ? activeCluster?.status === 'healthy'
              ? 'success'
              : 'danger'
            : 'secondary'
        "
        fluid
        class="truncate justify-start font-semibold text-xs py-2"
        variant="text"
        @click="handleSwitchCluster(cluster.id)"
      >
        <Check v-if="k8sStore.activeClusterId === cluster.id" :size="14" class="shrink-0 mr-1" />
        <span class="truncate">{{ cluster.name }}</span>
      </Button>

      <!-- Empty state -->
      <p v-if="k8sStore.clusters.length === 0" class="text-xs text-muted-color px-2 py-1">
        No clusters added yet
      </p>
    </div>

    <!-- Add Cluster Button -->
    <Button fluid severity="contrast" size="small" class="mt-auto" @click="handleAddCluster">
      <Plus :size="14" />
      <span class="text-xs font-semibold">Add cluster</span>
    </Button>
  </div>
</template>
