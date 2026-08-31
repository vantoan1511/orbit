<script setup lang="ts">
import { computed } from 'vue'
import { Card } from 'primevue'
import { Bell, CheckCircle, AlertTriangle, XCircle, Info } from '@lucide/vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { KUBERNETES_EVENT_TYPE } from '@/constants/kubernetes'

const k8sStore = useKubernetesStore()
const events = computed(() => k8sStore.events)

const totalCount = computed(() => events.value.length)
const normalCount = computed(
  () => events.value.filter((e) => e.type === KUBERNETES_EVENT_TYPE.Normal).length
)
const warningCount = computed(
  () => events.value.filter((e) => e.type === KUBERNETES_EVENT_TYPE.Warning).length
)
const errorCount = computed(() => events.value.filter((e) => e.type === 'Error').length)
const otherCount = computed(
  () =>
    events.value.filter(
      (e) =>
        e.type !== KUBERNETES_EVENT_TYPE.Normal &&
        e.type !== KUBERNETES_EVENT_TYPE.Warning &&
        e.type !== 'Error'
    ).length
)

const normalPct = computed(() =>
  totalCount.value ? ((normalCount.value / totalCount.value) * 100).toFixed(1) : '0.0'
)
const warningPct = computed(() =>
  totalCount.value ? ((warningCount.value / totalCount.value) * 100).toFixed(1) : '0.0'
)
const errorPct = computed(() =>
  totalCount.value ? ((errorCount.value / totalCount.value) * 100).toFixed(1) : '0.0'
)
const otherPct = computed(() =>
  totalCount.value ? ((otherCount.value / totalCount.value) * 100).toFixed(1) : '0.0'
)
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-6">
    <!-- Card 1: Total Events -->
    <Card>
      <template #content>
        <div class="flex items-center gap-4">
          <div
            class="w-10 h-10 rounded-lg bg-(--bg-hover) flex items-center justify-center text-muted-color shrink-0"
          >
            <Bell class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[11px] font-semibold text-muted-color uppercase tracking-wider">
              Total Events
            </div>
            <div class="text-2xl font-bold text-primary font-mono mt-0.5">
              {{ totalCount }}
            </div>
            <div class="text-[10px] text-muted-color mt-0.5 truncate">Active cluster events</div>
          </div>
        </div>
      </template>
    </Card>

    <!-- Card 2: Normal -->
    <Card>
      <template #content>
        <div class="flex items-center gap-4">
          <div
            class="w-10 h-10 rounded-lg bg-emerald-500/10 flex items-center justify-center text-emerald-400 shrink-0"
          >
            <CheckCircle class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[11px] font-semibold text-muted-color uppercase tracking-wider">
              Normal
            </div>
            <div class="text-2xl font-bold text-primary font-mono mt-0.5">
              {{ normalCount }}
            </div>
            <div class="text-[10px] text-emerald-400 font-medium mt-0.5 truncate">
              {{ normalPct }}%
            </div>
          </div>
        </div>
      </template>
    </Card>

    <!-- Card 3: Warning -->
    <Card>
      <template #content>
        <div class="flex items-center gap-4">
          <div
            class="w-10 h-10 rounded-lg bg-amber-500/10 flex items-center justify-center text-amber-400 shrink-0"
          >
            <AlertTriangle class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[11px] font-semibold text-muted-color uppercase tracking-wider">
              Warning
            </div>
            <div class="text-2xl font-bold text-primary font-mono mt-0.5">
              {{ warningCount }}
            </div>
            <div class="text-[10px] text-amber-400 font-medium mt-0.5 truncate">
              {{ warningPct }}%
            </div>
          </div>
        </div>
      </template>
    </Card>

    <!-- Card 4: Error -->
    <Card>
      <template #content>
        <div class="flex items-center gap-4">
          <div
            class="w-10 h-10 rounded-lg bg-red-500/10 flex items-center justify-center text-red-400 shrink-0"
          >
            <XCircle class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[11px] font-semibold text-muted-color uppercase tracking-wider">
              Error
            </div>
            <div class="text-2xl font-bold text-primary font-mono mt-0.5">
              {{ errorCount }}
            </div>
            <div class="text-[10px] text-red-400 font-medium mt-0.5 truncate">{{ errorPct }}%</div>
          </div>
        </div>
      </template>
    </Card>

    <!-- Card 5: Other -->
    <Card>
      <template #content>
        <div class="flex items-center gap-4">
          <div
            class="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center text-blue-400 shrink-0"
          >
            <Info class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[11px] font-semibold text-muted-color uppercase tracking-wider">
              Other
            </div>
            <div class="text-2xl font-bold text-primary font-mono mt-0.5">
              {{ otherCount }}
            </div>
            <div class="text-[10px] text-blue-400 font-medium mt-0.5 truncate">{{ otherPct }}%</div>
          </div>
        </div>
      </template>
    </Card>
  </div>
</template>
