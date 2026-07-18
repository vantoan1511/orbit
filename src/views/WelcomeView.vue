<script setup lang="ts">
import { useCluster } from '@/composables/useCluster'
import { Cog, Compass, FolderOpen, Layers, Lock } from '@lucide/vue'

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
    description: 'Choose a cluster from the list to make it active.'
  },
  {
    icon: Compass,
    step: '3. Explore Resources',
    description: 'Browse and monitor workloads, services, pods, and more.'
  },
  {
    icon: Cog,
    step: '4. Manage with Ease',
    description: 'Edit configurations, view logs, and manage your cluster.'
  }
]
</script>

<template>
  <div class="flex flex-col h-full overflow-y-auto">
    <!-- Hero section -->
    <div
      class="flex flex-col items-center justify-center flex-1 gap-8 text-center px-8 py-12 select-none min-h-130"
    >
      <!-- Animated orbit logo -->
      <div class="relative w-28 h-28 flex items-center justify-center">
        <!-- Outer orbit ring -->
        <div
          class="absolute inset-0 rounded-full border-2 border-blue-500/30 animate-spin"
          style="animation-duration: 8s"
        ></div>
        <!-- Inner orbit ring -->
        <div
          class="absolute inset-3 rounded-full border border-blue-500/20 animate-spin"
          style="animation-duration: 5s; animation-direction: reverse"
        ></div>
        <!-- Satellite dot -->
        <div
          class="absolute top-0 left-1/2 w-3 h-3 rounded-full bg-blue-500 shadow-lg shadow-blue-500/50 animate-spin"
          style="animation-duration: 8s; transform-origin: 50% 56px"
        ></div>
        <!-- Center logo -->
        <img src="/logo.png" alt="Orbit" class="w-14 h-14 object-contain relative z-10" />
      </div>

      <!-- Headline -->
      <div class="flex flex-col gap-2 max-w-md">
        <h1 class="text-3xl font-bold tracking-tight text-(--text-primary) font-ui">
          Welcome to Orbit
        </h1>
        <p class="text-sm text-(--text-secondary) leading-relaxed">
          Orbit helps you explore, monitor, and manage<br />
          your Kubernetes clusters from a single, intuitive interface.
        </p>
      </div>

      <!-- Tip card -->
      <div
        class="flex items-start gap-3 px-6 py-4 rounded-xl border border-(--border) bg-(--bg-card) max-w-md w-full text-left"
      >
        <span class="text-xl leading-none mt-0.5">👈</span>
        <div class="flex flex-col gap-1">
          <span class="text-sm font-semibold text-(--text-primary)"
            >Click on a cluster to start browsing</span
          >
          <span class="text-xs text-(--text-secondary)">
            Select a cluster from the list on the left to view its overview and start managing your
            resources.
          </span>
        </div>
      </div>

      <!-- Or divider -->
      <div class="flex items-center gap-4 text-(--text-muted) text-xs w-72">
        <div class="flex-1 h-px bg-(--border)"></div>
        <span>or</span>
        <div class="flex-1 h-px bg-(--border)"></div>
      </div>

      <!-- Add Cluster CTA -->
      <div class="flex flex-col items-center gap-3">
        <Button
          fluid
          id="welcome-add-cluster-btn"
          icon="pi pi-plus"
          label="Add Cluster"
          severity="secondary"
          @click="handleAddCluster"
        />
        <p class="text-sm font-semibold text-(--text-secondary)">
          No clusters yet? Add one to get started.
        </p>
        <p class="text-xs text-(--text-muted) max-w-xs">
          You can add a cluster using your kubeconfig file or connect to a remote cluster.
        </p>
      </div>
    </div>

    <!-- Quick start section -->
    <div class="border-t border-(--border) bg-(--bg-card) px-8 py-6 select-none">
      <h2 class="text-sm font-semibold text-(--text-primary) mb-5">Quick start</h2>
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-6">
        <div v-for="item in quickStartSteps" :key="item.step" class="flex flex-col gap-2">
          <component :is="item.icon" class="w-5 h-5 text-blue-500" />
          <span class="text-sm font-semibold text-(--text-primary)">{{ item.step }}</span>
          <p class="text-xs text-(--text-secondary) leading-relaxed">{{ item.description }}</p>
        </div>
      </div>
    </div>

    <!-- Privacy footer -->
    <div
      class="border-t border-(--border) px-8 py-3 flex items-center justify-center gap-2 text-xs text-(--text-muted) select-none"
    >
      <Lock class="w-3.5 h-3.5" />
      <span>
        Your data stays local. Orbit doesn't collect or share any of your cluster information.
      </span>
    </div>
  </div>
</template>
