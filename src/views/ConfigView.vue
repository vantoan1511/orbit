<script setup lang="ts">
import ResourceTabsLayout, { type ResourceTab } from '@/components/shared/ResourceTabsLayout.vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { useDialog } from 'primevue/usedialog'
import { onMounted } from 'vue'
import ConfigDataTable from '../components/config/ConfigDataTable.vue'
import ConfigMetricsCards from '../components/config/ConfigMetricsCards.vue'
import CreateConfigMapDialog from '../components/config/CreateConfigMapDialog.vue'
import CreateSecretDialog from '../components/config/CreateSecretDialog.vue'

const k8sStore = useKubernetesStore()
const dialog = useDialog()

const openCreateConfigMapDialog = () => {
  dialog.open(CreateConfigMapDialog, {
    props: {
      header: 'Create ConfigMap',
      style: {
        width: '460px'
      },
      modal: true
    }
  })
}

const openCreateSecretDialog = () => {
  dialog.open(CreateSecretDialog, {
    props: {
      header: 'Create Secret',
      style: {
        width: '460px'
      },
      modal: true
    }
  })
}

const tabs: ResourceTab[] = [
  {
    id: 'configmaps',
    createAction: { handler: openCreateConfigMapDialog }
  },
  {
    id: 'secrets',
    createAction: { handler: openCreateSecretDialog }
  }
]

onMounted(async () => {
  await k8sStore.fetchConfigMaps()
  await k8sStore.fetchSecrets()
})
</script>

<template>
  <ResourceTabsLayout title="ConfigMaps & Secrets" default-tab="configmaps" :tabs="tabs">
    <!-- ConfigMaps Tab Panel -->
    <template #tab-configmaps>
      <div class="flex flex-col gap-6">
        <ConfigMetricsCards activeTab="configmaps" />
        <ConfigDataTable activeTab="configmaps" />
      </div>
    </template>

    <!-- Secrets Tab Panel -->
    <template #tab-secrets>
      <div class="flex flex-col gap-6">
        <ConfigMetricsCards activeTab="secrets" />
        <ConfigDataTable activeTab="secrets" />
      </div>
    </template>
  </ResourceTabsLayout>
</template>
