<script setup lang="ts">
import { ref } from 'vue'
import Button from 'primevue/button'
import { Hexagon, FileText, Database, CheckCircle2, Trash2 } from '@lucide/vue'
import { useToast } from 'primevue/usetoast'
import { useConfirm } from 'primevue/useconfirm'

const toast = useToast()
const confirm = useConfirm()

const currentContext = ref({
  name: 'production-us-east-1',
  namespace: 'default',
  server: 'https://api.us-east-1.k8s.example.com',
  user: 'kubernetes-admin-prod',
  status: 'connected'
})

const kubeconfigInfo = ref({
  contextsCount: 3,
  clustersCount: 3,
  usersCount: 2,
  fileSize: '12.4 KB'
})

const dataManagementInfo = ref({
  cachedObjects: 1482,
  cacheSize: '1.8 MB',
  lastSynced: 'Just now'
})

const handleClearCache = () => {
  confirm.require({
    message: 'Are you sure you want to clear the local object cache?',
    header: 'Clear Cache',
    accept: () => {
      dataManagementInfo.value.cachedObjects = 0
      dataManagementInfo.value.cacheSize = '0 KB'
      toast.add({
        severity: 'success',
        summary: 'Cache Cleared',
        detail: 'Local cache cleared successfully.',
        life: 3000
      })
    }
  })
}

const handleClearAllData = () => {
  confirm.require({
    message: 'This will reset all application data. Continue?',
    header: 'Reset All Data',
    accept: () => {
      toast.add({
        severity: 'success',
        summary: 'Database Cleared',
        detail: 'All local database storage cleared successfully.',
        life: 3000
      })
    }
  })
}
</script>

<template>
  <div class="flex flex-col gap-6">
    <!-- Card 1: Current Context -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 shadow-sm flex flex-col gap-4"
    >
      <div class="flex items-center gap-2 pb-3 border-b border-(--border)">
        <Hexagon class="w-4 h-4 text-violet-500" />
        <h4 class="text-xs font-bold text-primary uppercase tracking-wider">Current Context</h4>
      </div>

      <div class="flex flex-col gap-3">
        <div class="flex items-center justify-between">
          <span class="text-xs text-muted-color">Context Name</span>
          <span class="text-xs font-semibold text-primary truncate max-w-44">{{
            currentContext.name
          }}</span>
        </div>

        <div class="flex items-center justify-between">
          <span class="text-xs text-muted-color">Active Namespace</span>
          <span
            class="text-xs font-mono font-semibold bg-violet-500/10 text-violet-400 border border-violet-500/20 px-2 py-0.5 rounded text-[10px]"
          >
            {{ currentContext.namespace }}
          </span>
        </div>

        <div class="flex flex-col gap-1">
          <span class="text-xs text-muted-color">Server Address</span>
          <span
            class="text-[10px] font-mono text-muted-color bg-(--bg-hover)/20 p-2 rounded border border-(--border) truncate"
          >
            {{ currentContext.server }}
          </span>
        </div>

        <div class="flex items-center justify-between">
          <span class="text-xs text-muted-color">Auth User</span>
          <span class="text-xs font-semibold text-primary truncate max-w-40">{{
            currentContext.user
          }}</span>
        </div>

        <div class="flex items-center justify-between pt-2 border-t border-(--border)">
          <span class="text-xs text-muted-color">Status</span>
          <span class="flex items-center gap-1.5 text-emerald-500 font-semibold text-xs">
            <CheckCircle2 class="w-3.5 h-3.5" />
            Connected
          </span>
        </div>
      </div>
    </div>

    <!-- Card 2: Kubeconfig Details -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 shadow-sm flex flex-col gap-4"
    >
      <div class="flex items-center gap-2 pb-3 border-b border-(--border)">
        <FileText class="w-4 h-4 text-sky-500" />
        <h4 class="text-xs font-bold text-primary uppercase tracking-wider">Kubeconfig Details</h4>
      </div>

      <div class="flex flex-col gap-3">
        <div class="flex items-center justify-between">
          <span class="text-xs text-muted-color">Total Contexts</span>
          <span class="text-xs font-semibold text-primary">{{ kubeconfigInfo.contextsCount }}</span>
        </div>

        <div class="flex items-center justify-between">
          <span class="text-xs text-muted-color">Total Clusters</span>
          <span class="text-xs font-semibold text-primary">{{ kubeconfigInfo.clustersCount }}</span>
        </div>

        <div class="flex items-center justify-between">
          <span class="text-xs text-muted-color">Total Users</span>
          <span class="text-xs font-semibold text-primary">{{ kubeconfigInfo.usersCount }}</span>
        </div>

        <div class="flex items-center justify-between">
          <span class="text-xs text-muted-color">File Size</span>
          <span class="text-xs font-semibold text-primary">{{ kubeconfigInfo.fileSize }}</span>
        </div>
      </div>
    </div>

    <!-- Card 3: Data Management -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 shadow-sm flex flex-col gap-4"
    >
      <div class="flex items-center gap-2 pb-3 border-b border-(--border)">
        <Database class="w-4 h-4 text-emerald-500" />
        <h4 class="text-xs font-bold text-primary uppercase tracking-wider">Data Management</h4>
      </div>

      <div class="flex flex-col gap-3">
        <div class="flex items-center justify-between">
          <span class="text-xs text-muted-color">Cached Objects</span>
          <span class="text-xs font-semibold text-primary">{{
            dataManagementInfo.cachedObjects
          }}</span>
        </div>

        <div class="flex items-center justify-between">
          <span class="text-xs text-muted-color">Cache Size</span>
          <span class="text-xs font-semibold text-primary">{{ dataManagementInfo.cacheSize }}</span>
        </div>

        <div class="flex items-center justify-between">
          <span class="text-xs text-muted-color">Last Synced</span>
          <span class="text-xs font-semibold text-primary">{{
            dataManagementInfo.lastSynced
          }}</span>
        </div>

        <div class="flex flex-col gap-2 pt-2 border-t border-(--border)">
          <Button
            size="small"
            severity="secondary"
            variant="outlined"
            fluid
            class="text-xs font-semibold"
            @click="handleClearCache"
          >
            <Trash2 class="w-3.5 h-3.5 text-rose-500 mr-1" />
            <span>Clear Local Cache</span>
          </Button>

          <Button
            size="small"
            severity="danger"
            variant="outlined"
            fluid
            class="text-xs font-semibold"
            @click="handleClearAllData"
          >
            <span>Clear All Data</span>
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
