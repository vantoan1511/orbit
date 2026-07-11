<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { computed } from 'vue'

const k8sStore = useKubernetesStore()
const nodes = computed(() => k8sStore.nodes)
</script>

<template>
  <div
    class="bg-(--bg-card) border border-(--border) rounded-xl p-6 shadow-sm transition-all duration-200"
  >
    <div class="flex items-center justify-between mb-6 flex-wrap gap-4">
      <div class="text-sm font-semibold text-(--text-primary) uppercase tracking-wider">
        Nodes List
      </div>
      <div class="text-xs text-(--text-muted) font-medium">Showing all {{ nodes.length }} nodes in cluster</div>
    </div>

    <!-- PrimeVue DataTable -->
    <DataTable
      :value="nodes"
      paginator
      :rows="5"
      class="p-datatable-sm border border-(--border) rounded-lg overflow-hidden"
      tableClass="w-full text-left text-xs border-collapse"
      paginatorTemplate="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
      currentPageReportTemplate="Showing {first} to {last} of {totalRecords} nodes"
    >
      <!-- Name Column -->
      <Column field="name" header="Name" sortable class="font-medium p-3 text-(--text-primary)">
        <template #body="{ data }">
          <span class="font-semibold">{{ data.name }}</span>
        </template>
      </Column>

      <!-- Status Column -->
      <Column field="status" header="Status" sortable class="p-3">
        <template #body="{ data }">
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
            <span class="text-(--text-secondary) font-medium">{{ data.status }}</span>
          </div>
        </template>
      </Column>

      <!-- Role Column -->
      <Column field="role" header="Role" sortable class="p-3">
        <template #body="{ data }">
          <span
            class="px-2 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider border"
            :class="
              data.role === 'control-plane'
                ? 'bg-purple-500/10 text-purple-400 border-purple-500/20'
                : 'bg-zinc-500/10 text-(--text-secondary) border-(--border)'
            "
          >
            {{ data.role }}
          </span>
        </template>
      </Column>

      <!-- Version Column -->
      <Column field="version" header="Version" class="p-3 font-mono text-(--text-muted)"></Column>

      <!-- CPU Column -->
      <Column field="cpuPct" header="CPU" sortable class="p-3 min-w-35">
        <template #body="{ data }">
          <div class="flex flex-col gap-1 w-full">
            <div class="flex justify-between font-mono text-(--text-secondary)">
              <span>{{ data.cpuPct }}%</span>
              <span class="text-(--text-muted) text-[10px]"
                >{{ data.cpuUsed }} / {{ data.cpuTotal }}C</span
              >
            </div>
            <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
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
            <div class="flex justify-between font-mono text-(--text-secondary)">
              <span>{{ data.memPct }}%</span>
              <span class="text-(--text-muted) text-[10px]"
                >{{ data.memUsed }} / {{ data.memTotal }}G</span
              >
            </div>
            <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
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
            <div class="flex justify-between font-mono text-(--text-secondary)">
              <span>{{ Math.round((data.podsCount / data.podsLimit) * 100) }}%</span>
              <span class="text-(--text-muted) text-[10px]"
                >{{ data.podsCount }} / {{ data.podsLimit }}</span
              >
            </div>
            <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
              <div
                class="h-full bg-sky-500 rounded-full"
                :style="{ width: (data.podsCount / data.podsLimit) * 100 + '%' }"
              ></div>
            </div>
          </div>
        </template>
      </Column>

      <!-- Uptime Column -->
      <Column field="uptime" header="Uptime" sortable class="p-3 text-(--text-secondary)"></Column>

      <!-- Labels Column -->
      <Column field="labels" header="Labels" class="p-3 max-w-50">
        <template #body="{ data }">
          <div class="flex flex-wrap gap-1">
            <span
              v-for="label in data.labels"
              :key="label"
              class="px-1.5 py-0.5 rounded bg-(--bg-hover) text-(--text-secondary) text-[10px] border border-(--border) font-mono truncate"
              :title="label"
            >
              {{ label }}
            </span>
          </div>
        </template>
      </Column>
    </DataTable>
  </div>
</template>
