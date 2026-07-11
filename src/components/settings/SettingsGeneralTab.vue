<script setup lang="ts">
import GeneralSettingsForm from './GeneralSettingsForm.vue'
import SettingsInfoCards from './SettingsInfoCards.vue'
</script>

<template>
  <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 items-start">
    <!-- Left Column: Form -->
    <div class="lg:col-span-2 flex flex-col gap-6">
      <GeneralSettingsForm />
    </div>

    <!-- Right Column: Info Cards -->
    <div class="flex flex-col gap-6">
      <SettingsInfoCards />
    </div>
  </div>
</template>
