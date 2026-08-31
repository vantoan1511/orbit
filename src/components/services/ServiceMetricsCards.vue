<script setup lang="ts">
import { computed } from 'vue'
import { Card } from 'primevue'
import { Network, Circle, Cpu, Cloud, Globe } from '@lucide/vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { KUBERNETES_SERVICE_TYPE } from '@/constants/kubernetes'

const k8sStore = useKubernetesStore()

const totalServices = computed(() => k8sStore.services.length)
const clusterIPCount = computed(
  () => k8sStore.services.filter((s) => s.type === KUBERNETES_SERVICE_TYPE.ClusterIP).length
)
const nodePortCount = computed(
  () => k8sStore.services.filter((s) => s.type === KUBERNETES_SERVICE_TYPE.NodePort).length
)
const loadBalancerCount = computed(
  () => k8sStore.services.filter((s) => s.type === KUBERNETES_SERVICE_TYPE.LoadBalancer).length
)
const externalNameCount = computed(
  () => k8sStore.services.filter((s) => s.type === KUBERNETES_SERVICE_TYPE.ExternalName).length
)

const clusterIPPct = computed(() => {
  if (totalServices.value === 0) return '0.0%'
  return ((clusterIPCount.value / totalServices.value) * 100).toFixed(1) + '%'
})

const nodePortPct = computed(() => {
  if (totalServices.value === 0) return '0.0%'
  return ((nodePortCount.value / totalServices.value) * 100).toFixed(1) + '%'
})

const loadBalancerPct = computed(() => {
  if (totalServices.value === 0) return '0.0%'
  return ((loadBalancerCount.value / totalServices.value) * 100).toFixed(1) + '%'
})

const externalNamePct = computed(() => {
  if (totalServices.value === 0) return '0.0%'
  return ((externalNameCount.value / totalServices.value) * 100).toFixed(1) + '%'
})

const namespacesCount = computed(() => {
  const set = new Set(k8sStore.services.map((s) => s.namespace))
  return set.size
})
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-6">
    <!-- Card 1: Total Services -->
    <Card>
      <template #content>
        <div class="flex items-center gap-4">
          <div
            class="w-10 h-10 rounded-lg bg-(--bg-hover) flex items-center justify-center text-muted-color shrink-0"
          >
            <Network class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[11px] font-semibold text-muted-color uppercase tracking-wider">
              Total Services
            </div>
            <div class="text-2xl font-bold text-primary font-mono mt-0.5">
              {{ totalServices }}
            </div>
            <div class="text-[10px] text-muted-color mt-0.5 truncate">
              Across {{ namespacesCount }} namespaces
            </div>
          </div>
        </div>
      </template>
    </Card>

    <!-- Card 2: Cluster IP -->
    <Card>
      <template #content>
        <div class="flex items-center gap-4">
          <div
            class="w-10 h-10 rounded-lg bg-emerald-500/10 flex items-center justify-center text-emerald-400 shrink-0"
          >
            <Circle class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[11px] font-semibold text-muted-color uppercase tracking-wider">
              Cluster IP
            </div>
            <div class="text-2xl font-bold text-primary font-mono mt-0.5">
              {{ clusterIPCount }}
            </div>
            <div class="text-[10px] text-emerald-400 font-medium mt-0.5 truncate">
              {{ clusterIPPct }}
            </div>
          </div>
        </div>
      </template>
    </Card>

    <!-- Card 3: NodePort -->
    <Card>
      <template #content>
        <div class="flex items-center gap-4">
          <div
            class="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center text-blue-400 shrink-0"
          >
            <Cpu class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[11px] font-semibold text-muted-color uppercase tracking-wider">
              NodePort
            </div>
            <div class="text-2xl font-bold text-primary font-mono mt-0.5">
              {{ nodePortCount }}
            </div>
            <div class="text-[10px] text-blue-400 font-medium mt-0.5 truncate">
              {{ nodePortPct }}
            </div>
          </div>
        </div>
      </template>
    </Card>

    <!-- Card 4: LoadBalancer -->
    <Card>
      <template #content>
        <div class="flex items-center gap-4">
          <div
            class="w-10 h-10 rounded-lg bg-violet-500/10 flex items-center justify-center text-violet-400 shrink-0"
          >
            <Cloud class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[11px] font-semibold text-muted-color uppercase tracking-wider">
              LoadBalancer
            </div>
            <div class="text-2xl font-bold text-primary font-mono mt-0.5">
              {{ loadBalancerCount }}
            </div>
            <div class="text-[10px] text-violet-400 font-medium mt-0.5 truncate">
              {{ loadBalancerPct }}
            </div>
          </div>
        </div>
      </template>
    </Card>

    <!-- Card 5: External Name -->
    <Card>
      <template #content>
        <div class="flex items-center gap-4">
          <div
            class="w-10 h-10 rounded-lg bg-amber-500/10 flex items-center justify-center text-amber-400 shrink-0"
          >
            <Globe class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[11px] font-semibold text-muted-color uppercase tracking-wider">
              External Name
            </div>
            <div class="text-2xl font-bold text-primary font-mono mt-0.5">
              {{ externalNameCount }}
            </div>
            <div class="text-[10px] text-amber-400 font-medium mt-0.5 truncate">
              {{ externalNamePct }}
            </div>
          </div>
        </div>
      </template>
    </Card>
  </div>
</template>
