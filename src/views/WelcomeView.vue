<script setup lang="ts">
import { useCluster } from '@/composables/useCluster'
import { Cog, Compass, FolderOpen, Layers, Lock, Plus } from '@lucide/vue'

const { handleAddCluster } = useCluster()

const quickStartSteps = [
  {
    icon: FolderOpen,
    step: '1. Add a Cluster',
    description: 'Import your kubeconfig file or connect to a remote cluster.'
  },
  {
    icon: Layers,
    step: '2. Select a Cluster',
    description: 'Choose a cluster from the sidebar to activate and monitor it.'
  },
  {
    icon: Compass,
    step: '3. Explore Resources',
    description: 'Browse workloads, pods, services, network policies, and storage.'
  },
  {
    icon: Cog,
    step: '4. Inspect & Manage',
    description: 'View real-time logs, edit configurations, scale replicas, and track events.'
  }
]
</script>

<template>
  <div class="max-w-4xl mx-auto flex flex-col gap-8 select-none py-6">
    <!-- Header / Branding -->
    <div class="flex items-center gap-3.5">
      <div
        class="w-9 h-9 rounded-lg bg-(--bg-card) border border-(--border) flex items-center justify-center shrink-0"
      >
        <img src="/logo.png" alt="Orbit" class="w-5 h-5 object-contain" />
      </div>
      <div class="flex flex-col">
        <h1 class="text-xl font-bold tracking-tight text-primary font-ui">Orbit</h1>
        <p class="text-xs text-muted-color">Lightweight, native desktop Kubernetes dashboard</p>
      </div>
    </div>

    <!-- Main Content: 2-column asymmetric layout (IDE / VS Code style) -->
    <div class="grid grid-cols-1 md:grid-cols-12 gap-8 items-start">
      <!-- Left Column: Start Actions (col-span-5) -->
      <div class="md:col-span-5 flex flex-col gap-6">
        <div class="flex flex-col gap-3">
          <h2 class="text-xs font-semibold uppercase tracking-wider text-muted-color">Start</h2>
          <div class="flex flex-col gap-2">
            <Button
              id="welcome-add-cluster-btn"
              severity="secondary"
              size="small"
              class="justify-start text-xs font-medium py-2 px-3 gap-2"
              @click="handleAddCluster"
            >
              <Plus class="w-4 h-4 text-primary" />
              <span>Add Cluster</span>
            </Button>
          </div>
          <p class="text-xs text-muted-color leading-relaxed">
            Connect using your local kubeconfig or specify a cluster endpoint.
          </p>
        </div>

        <!-- Tip / Note -->
        <div
          class="p-3.5 rounded-lg bg-(--bg-hover)/30 text-xs text-muted-color flex flex-col gap-1.5"
        >
          <span class="font-medium text-primary">Already configured?</span>
          <span class="leading-relaxed">
            Select an existing cluster from the sidebar on the left to start inspecting resources.
          </span>
        </div>
      </div>

      <!-- Right Column: Walkthrough / Quick Start (col-span-7) -->
      <div class="md:col-span-7 flex flex-col gap-3">
        <h2 class="text-xs font-semibold uppercase tracking-wider text-muted-color">Walkthrough</h2>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div
            v-for="item in quickStartSteps"
            :key="item.step"
            class="p-3.5 rounded-lg bg-(--bg-hover)/20 hover:bg-(--bg-hover)/40 transition-colors flex flex-col gap-2"
          >
            <div class="flex items-center gap-2">
              <component :is="item.icon" class="w-4 h-4 shrink-0 text-primary" />
              <span class="text-xs font-semibold text-primary">{{ item.step }}</span>
            </div>
            <p class="text-xs text-muted-color leading-relaxed">
              {{ item.description }}
            </p>
          </div>
        </div>
      </div>
    </div>

    <!-- Security / Local-first note footer -->
    <div
      class="pt-6 border-t border-(--border) flex items-center justify-between text-xs text-muted-color"
    >
      <div class="flex items-center gap-2">
        <Lock class="w-3.5 h-3.5" />
        <span>Local-first architecture. Orbit operates entirely on your machine.</span>
      </div>
      <span class="font-mono text-[11px] opacity-75">Orbit Desktop</span>
    </div>
  </div>
</template>
