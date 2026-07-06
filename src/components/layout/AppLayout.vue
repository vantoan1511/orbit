<script setup lang="ts">
import AppSidebar from './AppSidebar.vue'
import AppHeader from './AppHeader.vue'
</script>

<template>
  <div
    class="flex h-screen w-screen overflow-hidden bg-[var(--bg-app)] text-[var(--text-primary)] font-sans"
  >
    <!-- Sidebar -->
    <AppSidebar />

    <!-- Main Content Area -->
    <div class="flex-1 flex flex-col h-full overflow-hidden">
      <!-- Header -->
      <AppHeader />

      <!-- Scrollable Content -->
      <main class="flex-1 overflow-y-auto p-8">
        <RouterView />
      </main>
    </div>
  </div>
</template>
