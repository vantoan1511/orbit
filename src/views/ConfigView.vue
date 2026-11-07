<script setup lang="ts">
import ResourceTabsLayout, { type ResourceTab } from '@/components/shared/ResourceTabsLayout.vue'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
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

onMounted(() => {
  void k8sStore.ensureKindLoaded(KUBERNETES_RESOURCE_KIND.ConfigMap).catch(() => {})
  void k8sStore.ensureKindLoaded(KUBERNETES_RESOURCE_KIND.Secret).catch(() => {})
})
</script>

<template>
  <ResourceTabsLayout title="ConfigMaps & Secrets" default-tab="configmaps" :tabs="tabs">
    <!-- ConfigMaps Tab Panel -->
    <template #tab-configmaps>
      <div class="flex flex-col gap-6 flex-1 min-h-0">
        <ConfigMetricsCards activeTab="configmaps" class="shrink-0" />
        <ConfigDataTable activeTab="configmaps" class="flex-1 min-h-0" />
      </div>
    </template>

    <!-- Secrets Tab Panel -->
    <template #tab-secrets>
      <div class="flex flex-col gap-6 flex-1 min-h-0">
        <ConfigMetricsCards activeTab="secrets" class="shrink-0" />
        <ConfigDataTable activeTab="secrets" class="flex-1 min-h-0" />
      </div>
    </template>
  </ResourceTabsLayout>
</template>
