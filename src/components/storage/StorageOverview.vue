<script setup lang="ts">
import StoragePVTable from './StoragePVTable.vue'
import StoragePVCTable from './StoragePVCTable.vue'
import StorageClassesSidebar from './StorageClassesSidebar.vue'
</script>

<template>
  <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
    <!-- Tables (Left 2/3) -->
    <div class="lg:col-span-2 flex flex-col gap-6">
      <StoragePVTable />
      <StoragePVCTable />
    </div>

    <!-- Sidebar (Right 1/3) -->
    <div class="lg:col-span-1">
      <StorageClassesSidebar />
    </div>
  </div>
</template>
