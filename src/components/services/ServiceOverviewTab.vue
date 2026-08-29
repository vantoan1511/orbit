<script setup lang="ts">
import ActivePortForwardsList from '@/components/shared/ActivePortForwardsList.vue'
import KeyValueBadgeList from '@/components/shared/KeyValueBadgeList.vue'
import ReactiveAge from '@/components/shared/ReactiveAge.vue'
import type { ServiceInfo } from '@/types/kubernetes'
import { ExternalLink } from '@lucide/vue'
import Tag from 'primevue/tag'

defineProps<{
  service: ServiceInfo
}>()

const getTypeSeverity = (type: string) => {
  switch (type) {
    case 'LoadBalancer':
      return 'info'
    case 'ClusterIP':
      return 'success'
    case 'NodePort':
      return 'warn'
    case 'ExternalName':
      return 'contrast'
    default:
      return 'secondary'
  }
}
</script>

<template>
  <div class="space-y-6">
    <!-- General Section with Zonal Background & Proportional Spacing -->
    <div>
      <h3 class="text-[10px] font-bold text-muted-color uppercase tracking-wider mb-3">
        General Configuration
      </h3>
      <div class="bg-(--bg-hover)/40 rounded-xl p-4 text-xs space-y-4">
        <div class="grid grid-cols-2 gap-4">
          <div class="flex flex-col gap-1">
            <span class="text-muted-color font-medium">Namespace</span>
            <span class="font-mono text-primary">{{ service.namespace }}</span>
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-muted-color font-medium">Type</span>
            <div>
              <Tag
                :severity="getTypeSeverity(service.type)"
                :value="service.type"
                class="text-xs"
              />
            </div>
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-muted-color font-medium">Cluster IP</span>
            <span class="font-mono text-primary">{{ service.clusterIP || '-' }}</span>
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-muted-color font-medium">External IP</span>
            <div class="flex items-center gap-1.5">
              <span class="font-mono text-primary">{{ service.externalIP || '-' }}</span>
              <ExternalLink
                v-if="service.externalIP && service.externalIP !== '-'"
                class="w-3 h-3 text-violet-400 hover:text-violet-300 cursor-pointer"
              />
            </div>
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-muted-color font-medium">Session Affinity</span>
            <span class="text-primary">{{ service.sessionAffinity || 'None' }}</span>
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-muted-color font-medium">Internal Traffic Policy</span>
            <span class="text-primary">{{ service.internalTrafficPolicy || '-' }}</span>
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-muted-color font-medium">Age</span>
            <span class="text-primary font-mono"><ReactiveAge :age="service.age" /></span>
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-muted-color font-medium">Created</span>
            <span class="text-primary">{{ service.created || '-' }}</span>
          </div>
        </div>

        <div class="pt-2 border-t border-(--border)/50 flex flex-col gap-1">
          <span class="text-muted-color font-medium">UID</span>
          <span class="font-mono text-[10px] text-muted-color break-all">{{ service.uid }}</span>
        </div>
      </div>
    </div>

    <!-- Active Port Forwards -->
    <ActivePortForwardsList kind="Service" :namespace="service.namespace" :name="service.name" />

    <!-- Pod Selectors -->
    <KeyValueBadgeList
      title="Pod Selectors"
      :items="service.selector"
      variant="tag"
      empty-message="No pod selectors defined (headless, selectorless, or ExternalName service)."
    />

    <!-- Labels Section -->
    <KeyValueBadgeList :items="service.labels" title="Labels" variant="tag" />
  </div>
</template>
