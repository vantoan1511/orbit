<script setup lang="ts">
import ResourceTableSkeleton from '@/components/shared/ResourceTableSkeleton.vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { computed } from 'vue'

const k8sStore = useKubernetesStore()
const nodes = computed(() => k8sStore.nodes)
</script>

<template>
  <Card>
    <template #content>
      <div class="flex items-center justify-between mb-6 flex-wrap gap-4">
        <div class="text-sm font-semibold text-primary uppercase tracking-wider">Nodes List</div>
        <div class="text-xs text-muted-color font-medium">
          Showing all {{ nodes.length }} nodes in cluster
        </div>
      </div>

      <!-- PrimeVue DataTable -->
      <ResourceTableSkeleton v-if="k8sStore.nodesLoading" :columns="8" />
      <DataTable
        v-else
        :value="nodes"
        paginator
        :rows="5"
        class="border border-surface-200 dark:border-surface-700 rounded overflow-hidden"
        tableClass="w-full text-left text-sm border-collapse"
        paginatorTemplate="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
        currentPageReportTemplate="Showing {first} to {last} of {totalRecords} nodes"
      >
        <template #empty>
          <div class="text-center py-8 text-muted-color">No nodes found in cluster.</div>
        </template>
        <!-- Name Column -->
        <Column field="name" header="Name" sortable class="font-medium p-3">
          <template #body="{ data }">
            <span class="font-semibold">{{ data.name }}</span>
          </template>
        </Column>

        <!-- Status Column -->
        <Column field="status" header="Status" sortable class="p-3">
          <template #body="{ data }">
            <div class="flex items-center gap-2">
              <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
              <span class="font-semibold">{{ data.status }}</span>
            </div>
          </template>
        </Column>

        <!-- Role Column -->
        <Column field="role" header="Role" sortable class="p-3">
          <template #body="{ data }">
            <span
              class="px-2 py-0.5 rounded-full text-xs font-bold uppercase tracking-wider border"
              :class="
                data.role === 'control-plane'
                  ? 'bg-purple-500/10 text-purple-400 border-purple-500/20'
                  : 'bg-zinc-500/10 text-muted-color border-surface-200 dark:border-surface-700'
              "
            >
              {{ data.role }}
            </span>
          </template>
        </Column>

        <!-- Version Column -->
        <Column field="version" header="Version" class="p-3 font-mono text-muted-color"></Column>

        <!-- CPU Column -->
        <Column field="cpuPct" header="CPU" sortable class="p-3 min-w-35">
          <template #body="{ data }">
            <div class="flex flex-col gap-1 w-full">
              <div class="flex justify-between font-mono text-muted-color">
                <span>{{ Number(data.cpuPct.toFixed(1)) }}%</span>
                <span class="text-muted-color text-[10px]"
                  >{{ data.cpuUsed }} / {{ data.cpuTotal }}C</span
                >
              </div>
              <div
                class="w-full h-1.5 rounded-full bg-surface-200 dark:bg-surface-700 overflow-hidden"
              >
                <div
                  class="h-full bg-blue-500 rounded-full"
                  :style="{ width: data.cpuPct + '%' }"
                ></div>
              </div>
            </div>
          </template>
        </Column>

        <!-- Memory Column -->
        <Column field="memPct" header="Memory" sortable class="p-3 min-w-35">
          <template #body="{ data }">
            <div class="flex flex-col gap-1 w-full">
              <div class="flex justify-between font-mono text-muted-color">
                <span>{{ Number(data.memPct.toFixed(1)) }}%</span>
                <span class="text-muted-color text-[10px]"
                  >{{ data.memUsed }} / {{ data.memTotal }}G</span
                >
              </div>
              <div
                class="w-full h-1.5 rounded-full bg-surface-200 dark:bg-surface-700 overflow-hidden"
              >
                <div
                  class="h-full bg-indigo-500 rounded-full"
                  :style="{ width: data.memPct + '%' }"
                ></div>
              </div>
            </div>
          </template>
        </Column>

        <!-- Pods Column -->
        <Column field="podsCount" header="Pods" sortable class="p-3 min-w-30">
          <template #body="{ data }">
            <div class="flex flex-col gap-1 w-full">
              <div class="flex justify-between font-mono text-muted-color">
                <span>{{ Math.round((data.podsCount / data.podsLimit) * 100) }}%</span>
                <span class="text-muted-color text-[10px]"
                  >{{ data.podsCount }} / {{ data.podsLimit }}</span
                >
              </div>
              <div
                class="w-full h-1.5 rounded-full bg-surface-200 dark:bg-surface-700 overflow-hidden"
              >
                <div
                  class="h-full bg-sky-500 rounded-full"
                  :style="{ width: (data.podsCount / data.podsLimit) * 100 + '%' }"
                ></div>
              </div>
            </div>
          </template>
        </Column>

        <!-- Uptime Column -->
        <Column field="uptime" header="Uptime" sortable class="p-3 text-muted-color"></Column>

        <!-- Labels Column -->
        <Column field="labels" header="Labels" class="p-3 max-w-50">
          <template #body="{ data }">
            <div class="flex flex-wrap gap-1">
              <span
                v-for="label in data.labels"
                :key="label"
                class="px-1.5 py-0.5 rounded bg-surface-200 dark:bg-surface-700 text-muted-color text-xs font-mono truncate"
                :title="label"
              >
                {{ label }}
              </span>
            </div>
          </template>
        </Column>
      </DataTable>
    </template>
  </Card>
</template>
