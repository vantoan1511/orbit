<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import Column from 'primevue/column'

const k8sStore = useKubernetesStore()

const columns = [
  { field: 'name', header: 'Name', visible: true },
  { field: 'provisioner', header: 'Provisioner', visible: true },
  { field: 'reclaimPolicy', header: 'Reclaim Policy', visible: true },
  { field: 'volumeBindingMode', header: 'Volume Binding Mode', visible: true },
  { field: 'allowVolumeExpansion', header: 'Allow Volume Expansion', visible: true },
  { field: 'age', header: 'Age', visible: true }
]

const handleRefresh = async () => {
  try {
    await k8sStore.fetchStorageClasses()
  } catch (error) {
    console.error('Error fetching Storage Classes:', error)
  }
}
</script>

<template>
  <GenericResourceTable
    :data="k8sStore.storageClasses"
    :initialColumns="columns"
    :hideNamespaceFilter="true"
    :hideNamespaceColumn="true"
    :hideStatusFilter="true"
    :hideStatusColumn="true"
    :hideSystemNamespaceToggle="true"
    :searchFields="['name', 'provisioner']"
    kind="StorageClass"
    searchPlaceholder="Search Storage Classes..."
    emptyMessage="No Storage Classes found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} classes"
    :loading="k8sStore.storageClassesLoading"
    @refresh="handleRefresh"
  >
    <template #default="{ visibleCols }">
      <!-- Provisioner Column -->
      <Column
        v-if="visibleCols['provisioner']"
        field="provisioner"
        header="Provisioner"
        sortable
        class="p-3"
        bodyClass="font-mono text-muted-color"
      ></Column>

      <!-- Reclaim Policy Column -->
      <Column
        v-if="visibleCols['reclaimPolicy']"
        field="reclaimPolicy"
        header="Reclaim Policy"
        sortable
        class="p-3"
        bodyClass="text-muted-color"
      ></Column>

      <!-- Volume Binding Mode Column -->
      <Column
        v-if="visibleCols['volumeBindingMode']"
        field="volumeBindingMode"
        header="Volume Binding Mode"
        sortable
        class="p-3"
        bodyClass="text-muted-color"
      ></Column>

      <!-- Allow Volume Expansion Column -->
      <Column
        v-if="visibleCols['allowVolumeExpansion']"
        field="allowVolumeExpansion"
        header="Allow Volume Expansion"
        sortable
        class="p-3"
      >
        <template #body="{ data }">
          <span
            class="px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider"
            :class="
              data.allowVolumeExpansion
                ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                : 'bg-zinc-500/10 text-zinc-400 border border-zinc-500/20'
            "
          >
            {{ data.allowVolumeExpansion ? 'True' : 'False' }}
          </span>
        </template>
      </Column>
    </template>
  </GenericResourceTable>
</template>
