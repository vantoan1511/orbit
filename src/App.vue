<script setup lang="ts">
import { onMounted } from 'vue'
import AppLayout from './components/layout/AppLayout.vue'

onMounted(() => {
  // Initialize dark mode by default
  document.documentElement.classList.add('my-app-dark')
  document.documentElement.setAttribute('data-theme', 'dark')
})
</script>

<template>
  <AppLayout />
</template>

<style>
/* Global scrollbar styling matching modern dark theme */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--border-strong);
}
</style>
