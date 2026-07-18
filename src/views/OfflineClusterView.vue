<script setup lang="ts">
import { useCluster } from '@/composables/useCluster'
import { ServerOff } from '@lucide/vue'

const { isRefreshing, refreshCluster } = useCluster()
</script>

<template>
  <div class="flex flex-col items-center justify-center min-h-[60vh] text-center p-6">
    <div class="p-4 bg-rose-500/10 border border-rose-500/20 rounded-full mb-6">
      <ServerOff class="w-12 h-12 text-rose-500" />
    </div>

    <h2 class="text-2xl font-bold text-(--text-primary) mb-2 tracking-tight">Cluster is Offline</h2>

    <p class="text-sm text-(--text-secondary) max-w-md mb-6 leading-relaxed">
      Unable to connect to the selected cluster. Please check if your Kubernetes cluster is running,
      verify your network settings, and check that your kubeconfig credentials are still valid.
    </p>

    <Button icon="pi pi-sync" label="Reconnect" :loading="isRefreshing" @click="refreshCluster" />
  </div>
</template>
