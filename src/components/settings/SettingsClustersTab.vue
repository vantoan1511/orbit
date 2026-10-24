<script setup lang="ts">
import ClusterEditModal from '@/components/cluster/ClusterEditModal.vue'
import { useCluster } from '@/composables/useCluster'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { ClusterInfo } from '@/types/kubernetes'
import { filterClusters, getAllClusterTags, getClusterDisplayName } from '@/utils/clusterHelpers'
import { Check, Pencil, Plus, Server } from '@lucide/vue'
import { Button, Column, DataTable, IconField, InputIcon, InputText } from 'primevue'
import { computed, ref } from 'vue'

const k8sStore = useKubernetesStore()
const { activeCluster, handleAddCluster } = useCluster()

const searchQuery = ref('')
const selectedTag = ref<string | null>(null)
const editModalVisible = ref(false)
const selectedCluster = ref<ClusterInfo | null>(null)
const switchingClusterId = ref<string | null>(null)

const allTags = computed(() => getAllClusterTags(k8sStore.clusters))

const filteredClusters = computed(() =>
  filterClusters(k8sStore.clusters, searchQuery.value, selectedTag.value)
)

const handleSwitchCluster = async (clusterId: string) => {
  if (k8sStore.activeClusterId === clusterId) return
  switchingClusterId.value = clusterId
  try {
    await kubernetesService.switchCluster(clusterId)
  } finally {
    switchingClusterId.value = null
  }
}

const openEditModal = (cluster: ClusterInfo) => {
  selectedCluster.value = cluster
  editModalVisible.value = true
}

const toggleTag = (tag: string) => {
  if (selectedTag.value === tag) {
    selectedTag.value = null
  } else {
    selectedTag.value = tag
  }
}
</script>

<template>
  <div data-component="settings-clusters-tab" class="flex flex-col gap-4">
    <!-- Header / Micro-description -->
    <div class="flex items-center justify-between gap-4">
      <div class="flex flex-col gap-1">
        <h3 class="text-sm font-bold tracking-tight text-primary">Connected Clusters</h3>
        <p class="text-xs text-muted-color">
          Manage cluster aliases, environment colors, labels, and kubeconfig connections.
        </p>
      </div>

      <!-- Add Cluster Button -->
      <Button
        severity="contrast"
        size="small"
        class="h-8 text-xs font-semibold flex items-center gap-1.5"
        @click="handleAddCluster"
      >
        <Plus :size="14" />
        <span>Add Kubeconfig</span>
      </Button>
    </div>

    <!-- Toolbar: Search & Tag Filter -->
    <div class="flex items-center justify-between gap-3 flex-wrap">
      <div class="w-64">
        <IconField class="w-full">
          <InputIcon class="pi pi-search text-xs text-muted-color" />
          <InputText
            v-model="searchQuery"
            placeholder="Filter by name, context, tag..."
            class="w-full text-xs h-8 bg-(--bg-card) border-(--border)"
            size="small"
          />
          <InputIcon
            v-if="searchQuery"
            class="pi pi-times text-xs cursor-pointer text-muted-color hover:text-primary transition-colors"
            v-tooltip.top="'Clear filter'"
            aria-label="Clear filter"
            @click="searchQuery = ''"
          />
        </IconField>
      </div>

      <!-- Tag Filters -->
      <div v-if="allTags.length > 0" class="flex items-center gap-1.5 flex-wrap">
        <span class="text-xs text-muted-color mr-1">Tags:</span>
        <Button
          v-for="tag in allTags"
          :key="tag"
          size="small"
          variant="text"
          :class="[
            'text-[11px]! font-medium h-6! px-2! rounded-full! transition-colors',
            selectedTag === tag
              ? 'bg-(--accent)! text-white!'
              : 'bg-(--bg-hover)/60! text-muted-color hover:text-primary hover:bg-(--bg-hover)!'
          ]"
          @click="toggleTag(tag)"
        >
          <span>#{{ tag }}</span>
        </Button>
      </div>
    </div>

    <!-- Clusters DataTable (Flat single-container layout) -->
    <div class="border border-(--border) rounded-lg overflow-hidden bg-(--bg-card)">
      <DataTable
        :value="filteredClusters"
        data-key="id"
        size="small"
        class="text-xs"
        :pt="{
          headerRow: { class: 'bg-surface-100 dark:bg-surface-900 border-b border-(--border)' },
          row: {
            class:
              'hover:bg-(--bg-hover) transition-colors border-b border-(--border)/60 last:border-b-0'
          }
        }"
      >
        <template #empty>
          <div
            class="p-8 text-center text-muted-color flex flex-col items-center justify-center gap-2"
          >
            <Server :size="24" class="text-muted-color/60" />
            <span>{{
              k8sStore.clusters.length === 0
                ? 'No clusters discovered. Add a kubeconfig file to get started.'
                : 'No clusters match the search filters.'
            }}</span>
          </div>
        </template>

        <!-- Status & Color Column -->
        <Column header="Status" style="width: 110px">
          <template #body="{ data }">
            <div class="flex items-center gap-2">
              <span
                class="w-2.5 h-2.5 rounded-full shrink-0"
                :style="data.color ? { backgroundColor: data.color } : undefined"
                :class="[
                  !data.color &&
                    (k8sStore.activeClusterId === data.id
                      ? activeCluster?.status === 'healthy'
                        ? 'bg-(--success)'
                        : activeCluster?.status === 'warning'
                          ? 'bg-(--warning)'
                          : 'bg-(--danger)'
                      : 'bg-muted-color/40')
                ]"
                v-tooltip.top="
                  data.color ? `Custom indicator (${data.status})` : `Status: ${data.status}`
                "
              />
              <span class="capitalize text-xs font-medium text-muted-color">
                {{ data.status }}
              </span>
            </div>
          </template>
        </Column>

        <!-- Name & Context Column -->
        <Column header="Cluster / Context">
          <template #body="{ data }">
            <div class="flex flex-col gap-0.5">
              <div class="flex items-center gap-1.5">
                <span class="font-bold text-primary text-xs">
                  {{ getClusterDisplayName(data) }}
                </span>
                <span
                  v-if="k8sStore.activeClusterId === data.id"
                  class="text-[10px] font-semibold px-1.5 py-0.2 rounded bg-(--accent-soft) text-(--accent) border border-(--accent)/30"
                >
                  Active
                </span>
              </div>
              <span
                v-if="data.alias && data.alias !== data.name"
                class="font-mono text-[11px] text-muted-color truncate"
                v-tooltip.top="data.name"
              >
                {{ data.name }}
              </span>
            </div>
          </template>
        </Column>

        <!-- Server API URL Column -->
        <Column header="Server API">
          <template #body="{ data }">
            <span
              class="font-mono text-[11px] text-muted-color truncate max-w-[220px] block"
              v-tooltip.top="data.serverUrl || 'Default API Server'"
            >
              {{ data.serverUrl || 'Default API Server' }}
            </span>
          </template>
        </Column>

        <!-- Tags Column -->
        <Column header="Tags">
          <template #body="{ data }">
            <div v-if="data.tags && data.tags.length > 0" class="flex flex-wrap gap-1">
              <span
                v-for="tag in data.tags"
                :key="tag"
                class="text-[10px] font-medium px-1.5 py-0.5 rounded bg-(--bg-hover) text-muted-color border border-(--border)/60"
              >
                #{{ tag }}
              </span>
            </div>
            <span v-else class="text-muted-color/50 text-[11px]">—</span>
          </template>
        </Column>

        <!-- Actions Column -->
        <Column header="Actions" style="width: 140px; text-align: right">
          <template #body="{ data }">
            <div class="flex items-center justify-end gap-1.5">
              <Button
                v-if="k8sStore.activeClusterId !== data.id"
                size="small"
                variant="outlined"
                severity="secondary"
                class="h-7 text-xs px-2.5"
                :loading="switchingClusterId === data.id"
                @click="handleSwitchCluster(data.id)"
              >
                Switch
              </Button>
              <span
                v-else
                class="text-xs font-semibold text-(--accent) flex items-center gap-1 px-2 py-1"
              >
                <Check :size="13" />
                <span>Connected</span>
              </span>

              <Button
                size="small"
                variant="text"
                severity="secondary"
                v-tooltip.top="'Edit cluster details'"
                class="w-7! h-7! p-0!"
                @click="openEditModal(data)"
              >
                <template #icon>
                  <Pencil :size="13" />
                </template>
              </Button>
            </div>
          </template>
        </Column>
      </DataTable>
    </div>

    <!-- Cluster Edit Modal -->
    <ClusterEditModal v-model:visible="editModalVisible" :cluster="selectedCluster" />
  </div>
</template>
