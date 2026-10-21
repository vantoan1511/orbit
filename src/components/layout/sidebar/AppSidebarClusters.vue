<script setup lang="ts">
import ClusterEditModal from '@/components/cluster/ClusterEditModal.vue'
import { useCluster } from '@/composables/useCluster'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { ClusterInfo } from '@/types/kubernetes'
import { filterClusters, getAllClusterTags, getClusterDisplayName } from '@/utils/clusterHelpers'
import { Check, Pencil, Plus, Search, Settings2, X } from '@lucide/vue'
import { Button, InputText } from 'primevue'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

const emit = defineEmits<{
  (e: 'clusterSwitched'): void
}>()

const router = useRouter()
const k8sStore = useKubernetesStore()
const { activeCluster, isRefreshing, handleAddCluster } = useCluster()

const searchQuery = ref('')
const selectedTag = ref<string | null>(null)
const editModalVisible = ref(false)
const clusterToEdit = ref<ClusterInfo | null>(null)

const allTags = computed(() => getAllClusterTags(k8sStore.clusters))

const filteredClusters = computed(() =>
  filterClusters(k8sStore.clusters, searchQuery.value, selectedTag.value)
)

const handleSwitchCluster = async (clusterId: string) => {
  if (k8sStore.activeClusterId === clusterId) return
  await kubernetesService.switchCluster(clusterId)
  emit('clusterSwitched')
}

const openEditModal = (cluster: ClusterInfo, event?: Event) => {
  if (event) {
    event.stopPropagation()
  }
  clusterToEdit.value = cluster
  editModalVisible.value = true
}

const toggleTag = (tag: string) => {
  if (selectedTag.value === tag) {
    selectedTag.value = null
  } else {
    selectedTag.value = tag
  }
}

const clearFilters = () => {
  searchQuery.value = ''
  selectedTag.value = null
}
</script>

<template>
  <div data-component="sidebar-clusters" class="flex-1 overflow-hidden p-3 flex flex-col gap-2.5">
    <!-- Top Search Bar -->
    <div v-if="k8sStore.clusters.length > 0" class="relative shrink-0">
      <Search :size="13" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-color pointer-events-none" />
      <InputText
        v-model="searchQuery"
        placeholder="Filter clusters or tags..."
        class="w-full text-xs h-7.5 pl-8! pr-7! bg-(--bg-hover)/40! border-(--border)!"
        size="small"
      />
      <Button
        v-if="searchQuery"
        variant="text"
        severity="secondary"
        size="small"
        class="w-5! h-5! p-0! absolute right-1.5 top-1/2 -translate-y-1/2 text-muted-color hover:text-primary"
        @click="searchQuery = ''"
      >
        <template #icon>
          <X :size="12" />
        </template>
      </Button>
    </div>

    <!-- Quick Filter Tag Pills -->
    <div
      v-if="allTags.length > 0"
      class="flex items-center gap-1 overflow-x-auto no-scrollbar shrink-0 py-0.5"
    >
      <Button
        v-for="tag in allTags"
        :key="tag"
        size="small"
        variant="text"
        :class="[
          'text-[10px]! font-medium h-5! px-1.5! rounded-full! transition-colors shrink-0',
          selectedTag === tag
            ? 'bg-(--accent)! text-white!'
            : 'bg-(--bg-hover)/60! text-muted-color hover:text-primary hover:bg-(--bg-hover)!'
        ]"
        @click="toggleTag(tag)"
      >
        <span>#{{ tag }}</span>
      </Button>
    </div>

    <!-- Cluster List -->
    <div class="flex flex-col gap-1.5 flex-1 overflow-y-auto pr-0.5">
      <div
        v-for="cluster in filteredClusters"
        :key="cluster.id"
        class="group relative flex flex-col gap-1 p-2 rounded-lg border transition-all cursor-pointer select-none"
        :class="[
          k8sStore.activeClusterId === cluster.id
            ? 'border-(--accent)/40 bg-(--accent-soft)'
            : 'border-transparent bg-(--bg-hover)/30 hover:bg-(--bg-hover)/70 hover:border-(--border)'
        ]"
        @click="handleSwitchCluster(cluster.id)"
      >
        <!-- Cluster Header: Color / Status Dot + Name + Actions -->
        <div class="flex items-center justify-between gap-1.5 min-w-0">
          <div class="flex items-center gap-2 min-w-0 flex-1">
            <!-- Environment Color Indicator or Status Dot -->
            <span
              class="w-2.5 h-2.5 rounded-full shrink-0 shadow-xs"
              :style="cluster.color ? { backgroundColor: cluster.color } : undefined"
              :class="[
                !cluster.color && (
                  k8sStore.activeClusterId === cluster.id
                    ? activeCluster?.status === 'healthy'
                      ? 'bg-(--success)'
                      : 'bg-(--danger)'
                    : 'bg-muted-color/40'
                )
              ]"
              v-tooltip.top="cluster.color ? `Environment indicator (${cluster.status})` : `Status: ${cluster.status}`"
            />

            <!-- Friendly Display Name / Alias -->
            <span
              class="font-semibold text-xs tracking-tight truncate text-primary"
              :class="{ 'text-(--accent) font-bold': k8sStore.activeClusterId === cluster.id }"
            >
              {{ getClusterDisplayName(cluster) }}
            </span>
          </div>

          <!-- Right side icons: Active Check & Edit button -->
          <div class="flex items-center gap-1 shrink-0">
            <Check
              v-if="k8sStore.activeClusterId === cluster.id"
              :size="13"
              class="text-(--accent) shrink-0"
            />
            <Button
              variant="text"
              severity="secondary"
              size="small"
              v-tooltip.top="'Edit cluster metadata'"
              class="w-5! h-5! p-0! opacity-0 group-hover:opacity-100 transition-opacity text-muted-color hover:text-primary"
              @click.stop="openEditModal(cluster, $event)"
            >
              <template #icon>
                <Pencil :size="11" />
              </template>
            </Button>
          </div>
        </div>

        <!-- Raw Context Subtitle (shown when alias is defined) -->
        <div v-if="cluster.alias && cluster.alias !== cluster.name" class="pl-4.5 truncate">
          <span class="font-mono text-[10px] text-muted-color truncate block" :title="cluster.name">
            {{ cluster.name }}
          </span>
        </div>

        <!-- Tag Chips -->
        <div v-if="cluster.tags && cluster.tags.length > 0" class="pl-4.5 flex flex-wrap gap-1 pt-0.5">
          <span
            v-for="tag in cluster.tags"
            :key="tag"
            class="text-[9px] font-medium px-1.5 py-0.2 rounded bg-(--bg-hover) text-muted-color border border-(--border)/60"
          >
            #{{ tag }}
          </span>
        </div>
      </div>

      <!-- Empty state: No clusters configured -->
      <div v-if="k8sStore.clusters.length === 0" class="text-xs text-muted-color px-2 py-4 text-center">
        No clusters added yet
      </div>

      <!-- Empty state: Filters produced no results -->
      <div
        v-else-if="filteredClusters.length === 0"
        class="text-xs text-muted-color px-2 py-6 text-center flex flex-col items-center gap-2"
      >
        <span>No matching clusters found</span>
        <Button
          variant="text"
          severity="secondary"
          size="small"
          class="text-xs text-(--accent) hover:underline p-0!"
          @click="clearFilters"
        >
          Clear filters
        </Button>
      </div>
    </div>

    <!-- Bottom Actions Toolbar -->
    <div class="mt-auto shrink-0 flex flex-col gap-1.5 pt-2 border-t border-(--border)">
      <Button fluid severity="contrast" size="small" class="h-8 text-xs font-semibold flex items-center justify-center gap-1.5" @click="handleAddCluster">
        <Plus :size="14" />
        <span>Add cluster</span>
      </Button>

      <Button
        fluid
        variant="text"
        severity="secondary"
        size="small"
        class="h-7 text-xs text-muted-color hover:text-primary flex items-center justify-center gap-1.5"
        @click="router.push('/settings?tab=clusters')"
      >
        <Settings2 :size="13" />
        <span>Manage clusters</span>
      </Button>
    </div>

    <!-- Cluster Edit Modal -->
    <ClusterEditModal
      v-model:visible="editModalVisible"
      :cluster="clusterToEdit"
    />
  </div>
</template>
