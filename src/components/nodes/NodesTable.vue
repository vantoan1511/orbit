<script setup lang="ts">
import { ref } from 'vue'

const nodes = ref([
  {
    name: 'prod-node-master-0',
    status: 'Ready',
    role: 'control-plane',
    version: 'v1.33.2',
    cpuPct: 35,
    cpuUsed: '2.8',
    cpuTotal: '8',
    memPct: 52,
    memUsed: '16.6',
    memTotal: '32',
    podsCount: 28,
    podsLimit: 110,
    uptime: '18d 4h',
    labels: ['os=linux', 'role=control-plane']
  },
  {
    name: 'prod-node-worker-01',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 62,
    cpuUsed: '9.9',
    cpuTotal: '16',
    memPct: 78,
    memUsed: '24.9',
    memTotal: '32',
    podsCount: 68,
    podsLimit: 110,
    uptime: '18d 3h',
    labels: ['os=linux', 'role=worker', 'gpu=true']
  },
  {
    name: 'prod-node-worker-02',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 48,
    cpuUsed: '7.6',
    cpuTotal: '16',
    memPct: 65,
    memUsed: '20.8',
    memTotal: '32',
    podsCount: 52,
    podsLimit: 110,
    uptime: '18d 3h',
    labels: ['os=linux', 'role=worker']
  },
  {
    name: 'prod-node-worker-03',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 75,
    cpuUsed: '12.0',
    cpuTotal: '16',
    memPct: 88,
    memUsed: '28.1',
    memTotal: '32',
    podsCount: 94,
    podsLimit: 110,
    uptime: '15d 1h',
    labels: ['os=linux', 'role=worker', 'storage=fast']
  },
  {
    name: 'prod-node-worker-04',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 15,
    cpuUsed: '1.2',
    cpuTotal: '8',
    memPct: 42,
    memUsed: '6.7',
    memTotal: '16',
    podsCount: 15,
    podsLimit: 110,
    uptime: '18d 4h',
    labels: ['os=linux', 'role=worker']
  },
  {
    name: 'prod-node-worker-05',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 54,
    cpuUsed: '4.3',
    cpuTotal: '8',
    memPct: 70,
    memUsed: '11.2',
    memTotal: '16',
    podsCount: 45,
    podsLimit: 110,
    uptime: '18d 2h',
    labels: ['os=linux', 'role=worker']
  },
  {
    name: 'prod-node-worker-06',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 41,
    cpuUsed: '3.2',
    cpuTotal: '8',
    memPct: 59,
    memUsed: '9.4',
    memTotal: '16',
    podsCount: 38,
    podsLimit: 110,
    uptime: '18d 4h',
    labels: ['os=linux', 'role=worker']
  },
  {
    name: 'prod-node-worker-07',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 82,
    cpuUsed: '13.1',
    cpuTotal: '16',
    memPct: 91,
    memUsed: '29.1',
    memTotal: '32',
    podsCount: 102,
    podsLimit: 110,
    uptime: '10d 12h',
    labels: ['os=linux', 'role=worker', 'gpu=true']
  },
  {
    name: 'prod-node-worker-08',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 29,
    cpuUsed: '2.3',
    cpuTotal: '8',
    memPct: 47,
    memUsed: '7.5',
    memTotal: '16',
    podsCount: 31,
    podsLimit: 110,
    uptime: '18d 4h',
    labels: ['os=linux', 'role=worker']
  },
  {
    name: 'prod-node-worker-09',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 51,
    cpuUsed: '8.1',
    cpuTotal: '16',
    memPct: 69,
    memUsed: '22.0',
    memTotal: '32',
    podsCount: 58,
    podsLimit: 110,
    uptime: '12d 8h',
    labels: ['os=linux', 'role=worker']
  },
  {
    name: 'prod-node-worker-10',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 44,
    cpuUsed: '3.5',
    cpuTotal: '8',
    memPct: 62,
    memUsed: '9.9',
    memTotal: '16',
    podsCount: 40,
    podsLimit: 110,
    uptime: '18d 4h',
    labels: ['os=linux', 'role=worker']
  },
  {
    name: 'prod-node-worker-11',
    status: 'Ready',
    role: 'worker',
    version: 'v1.33.2',
    cpuPct: 58,
    cpuUsed: '4.6',
    cpuTotal: '8',
    memPct: 75,
    memUsed: '12.0',
    memTotal: '16',
    podsCount: 48,
    podsLimit: 110,
    uptime: '18d 1h',
    labels: ['os=linux', 'role=worker']
  }
])
</script>

<template>
  <div
    class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6 shadow-sm transition-all duration-200"
  >
    <div class="flex items-center justify-between mb-6 flex-wrap gap-4">
      <div class="text-sm font-semibold text-[var(--text-primary)] uppercase tracking-wider">
        Nodes List
      </div>
      <div class="text-xs text-[var(--text-muted)] font-medium">
        Showing all 12 nodes in cluster
      </div>
    </div>

    <!-- PrimeVue DataTable -->
    <DataTable
      :value="nodes"
      paginator
      :rows="5"
      class="p-datatable-sm border border-[var(--border)] rounded-lg overflow-hidden"
      tableClass="w-full text-left text-xs border-collapse"
      paginatorTemplate="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
      currentPageReportTemplate="Showing {first} to {last} of {totalRecords} nodes"
    >
      <!-- Name Column -->
      <Column
        field="name"
        header="Name"
        sortable
        class="font-medium p-3 text-[var(--text-primary)]"
      >
        <template #body="{ data }">
          <span class="font-semibold">{{ data.name }}</span>
        </template>
      </Column>

      <!-- Status Column -->
      <Column field="status" header="Status" sortable class="p-3">
        <template #body="{ data }">
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
            <span class="text-[var(--text-secondary)] font-medium">{{ data.status }}</span>
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
                : 'bg-zinc-500/10 text-[var(--text-secondary)] border-[var(--border)]'
            "
          >
            {{ data.role }}
          </span>
        </template>
      </Column>

      <!-- Version Column -->
      <Column
        field="version"
        header="Version"
        class="p-3 font-mono text-[var(--text-muted)]"
      ></Column>

      <!-- CPU Column -->
      <Column field="cpuPct" header="CPU" sortable class="p-3 min-w-[140px]">
        <template #body="{ data }">
          <div class="flex flex-col gap-1 w-full">
            <div class="flex justify-between font-mono text-[var(--text-secondary)]">
              <span>{{ data.cpuPct }}%</span>
              <span class="text-[var(--text-muted)] text-[10px]"
                >{{ data.cpuUsed }} / {{ data.cpuTotal }}C</span
              >
            </div>
            <div class="w-full h-1.5 rounded-full bg-[var(--bg-hover)] overflow-hidden">
              <div
                class="h-full bg-blue-500 rounded-full"
                :style="{ width: data.cpuPct + '%' }"
              ></div>
            </div>
          </div>
        </template>
      </Column>

      <!-- Memory Column -->
      <Column field="memPct" header="Memory" sortable class="p-3 min-w-[140px]">
        <template #body="{ data }">
          <div class="flex flex-col gap-1 w-full">
            <div class="flex justify-between font-mono text-[var(--text-secondary)]">
              <span>{{ data.memPct }}%</span>
              <span class="text-[var(--text-muted)] text-[10px]"
                >{{ data.memUsed }} / {{ data.memTotal }}G</span
              >
            </div>
            <div class="w-full h-1.5 rounded-full bg-[var(--bg-hover)] overflow-hidden">
              <div
                class="h-full bg-indigo-500 rounded-full"
                :style="{ width: data.memPct + '%' }"
              ></div>
            </div>
          </div>
        </template>
      </Column>

      <!-- Pods Column -->
      <Column field="podsCount" header="Pods" sortable class="p-3 min-w-[120px]">
        <template #body="{ data }">
          <div class="flex flex-col gap-1 w-full">
            <div class="flex justify-between font-mono text-[var(--text-secondary)]">
              <span>{{ Math.round((data.podsCount / data.podsLimit) * 100) }}%</span>
              <span class="text-[var(--text-muted)] text-[10px]"
                >{{ data.podsCount }} / {{ data.podsLimit }}</span
              >
            </div>
            <div class="w-full h-1.5 rounded-full bg-[var(--bg-hover)] overflow-hidden">
              <div
                class="h-full bg-sky-500 rounded-full"
                :style="{ width: (data.podsCount / data.podsLimit) * 100 + '%' }"
              ></div>
            </div>
          </div>
        </template>
      </Column>

      <!-- Uptime Column -->
      <Column
        field="uptime"
        header="Uptime"
        sortable
        class="p-3 text-[var(--text-secondary)]"
      ></Column>

      <!-- Labels Column -->
      <Column field="labels" header="Labels" class="p-3 max-w-[200px]">
        <template #body="{ data }">
          <div class="flex flex-wrap gap-1">
            <span
              v-for="label in data.labels"
              :key="label"
              class="px-1.5 py-0.5 rounded bg-[var(--bg-hover)] text-[var(--text-secondary)] text-[10px] border border-[var(--border)] font-mono truncate"
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

<style scoped>
/* Scoped overrides to blend PrimeVue paginator with Orbit's variables */
:deep(.p-datatable) {
  background: transparent;
}
:deep(.p-datatable-thead > tr > th) {
  background: var(--bg-hover);
  color: var(--text-secondary);
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
:deep(.p-datatable-tbody > tr) {
  background: transparent;
  color: var(--text-secondary);
  border-bottom: 1px solid rgba(var(--border), 0.5);
  transition: background-color 0.2s;
}
:deep(.p-datatable-tbody > tr:hover) {
  background: var(--bg-hover) !important;
}
:deep(.p-paginator) {
  background: transparent;
  border-top: 1px solid var(--border);
  color: var(--text-secondary);
  padding: 0.75rem;
  font-size: 0.75rem;
}
:deep(.p-paginator-current) {
  color: var(--text-muted);
}
:deep(.p-paginator-page),
:deep(.p-paginator-first),
:deep(.p-paginator-prev),
:deep(.p-paginator-next),
:deep(.p-paginator-last) {
  color: var(--text-secondary);
  border-radius: 6px;
  min-width: 2rem;
  height: 2rem;
}
:deep(.p-paginator-page.p-highlight) {
  background: var(--bg-active);
  color: var(--text-primary);
  font-weight: 600;
}
</style>
