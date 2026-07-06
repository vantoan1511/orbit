<script setup lang="ts">
import { 
  AlertTriangle, 
  RefreshCw, 
  Server, 
  Key, 
  FileText, 
  HardDrive, 
  CheckCircle2, 
  ArrowRight 
} from "@lucide/vue";

const warnings = [
  {
    id: 1,
    reason: "CrashLoopBackOff",
    message: "Pod api-7d6f8b7d6c-5k2jh in backend",
    time: "2m ago"
  },
  {
    id: 2,
    reason: "FailedScheduling",
    message: "0/12 nodes are available: Insufficient cpu.",
    time: "7m ago"
  },
  {
    id: 3,
    reason: "ImagePullBackOff",
    message: "Pod worker-5f4d5b7d9c-x8z2k in jobs",
    time: "15m ago"
  },
  {
    id: 4,
    reason: "Unhealthy",
    message: "Readiness probe failed: HTTP probe failed",
    time: "18m ago"
  },
  {
    id: 5,
    reason: "FailedMount",
    message: "MountVolume.SetUp failed for volume \"data\"",
    time: "23m ago"
  }
];

const events = [
  {
    id: 1,
    icon: RefreshCw,
    iconColor: "text-blue-400 bg-blue-500/10",
    message: "Deployment backend/api updated",
    time: "2m ago"
  },
  {
    id: 2,
    icon: Server,
    iconColor: "text-violet-400 bg-violet-500/10",
    message: "Node ip-10-0-1-23.ec2.internal joined the cluster",
    time: "6m ago"
  },
  {
    id: 3,
    icon: Key,
    iconColor: "text-amber-400 bg-amber-500/10",
    message: "Secret db-credentials modified",
    time: "11m ago"
  },
  {
    id: 4,
    icon: RefreshCw,
    iconColor: "text-emerald-400 bg-emerald-500/10",
    message: "Pod redis-0 restarted",
    time: "16m ago"
  },
  {
    id: 5,
    icon: FileText,
    iconColor: "text-indigo-400 bg-indigo-500/10",
    message: "ConfigMap app-config created",
    time: "18m ago"
  },
  {
    id: 6,
    icon: HardDrive,
    iconColor: "text-sky-400 bg-sky-500/10",
    message: "PersistentVolume pvc-2f8b7 bound",
    time: "25m ago"
  },
  {
    id: 7,
    icon: CheckCircle2,
    iconColor: "text-emerald-400 bg-emerald-500/10",
    message: "Job cleanup-171 completed",
    time: "32m ago"
  }
];
</script>

<template>
  <div class="grid grid-cols-1 xl:grid-cols-2 gap-6">
    <!-- Recent Warnings -->
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 shadow-sm transition-all duration-200 flex flex-col justify-between">
      <div>
        <div class="flex items-center justify-between mb-4">
          <div class="text-xs font-semibold text-[var(--text-primary)] uppercase tracking-wider">
            Recent Warnings
          </div>
          <button class="text-xs text-[var(--accent)] hover:underline flex items-center gap-1 font-medium">
            <span>View all</span>
          </button>
        </div>

        <div class="divide-y divide-[var(--border)]">
          <div 
            v-for="warning in warnings" 
            :key="warning.id" 
            class="py-3 flex items-start gap-3.5 first:pt-0 last:pb-0"
          >
            <div class="p-2 rounded-lg bg-rose-500/10 text-rose-500 mt-0.5 flex-shrink-0">
              <AlertTriangle class="w-4 h-4" />
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between gap-2">
                <span class="text-xs font-bold text-[var(--text-primary)] font-mono">{{ warning.reason }}</span>
                <span class="text-[10px] text-[var(--text-muted)] font-mono whitespace-nowrap">{{ warning.time }}</span>
              </div>
              <p class="text-xs text-[var(--text-secondary)] mt-1 truncate">{{ warning.message }}</p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Recent Events -->
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 shadow-sm transition-all duration-200 flex flex-col justify-between">
      <div>
        <div class="flex items-center justify-between mb-4">
          <div class="text-xs font-semibold text-[var(--text-primary)] uppercase tracking-wider">
            Recent Events
          </div>
          <button class="text-xs text-[var(--accent)] hover:underline flex items-center gap-1 font-medium">
            <span>View all</span>
          </button>
        </div>

        <div class="divide-y divide-[var(--border)]">
          <div 
            v-for="event in events" 
            :key="event.id" 
            class="py-2.5 flex items-center gap-3.5 first:pt-0 last:pb-0"
          >
            <div class="p-1.5 rounded-lg flex-shrink-0" :class="event.iconColor">
              <component :is="event.icon" class="w-4 h-4" />
            </div>
            <div class="flex-1 min-w-0 flex items-center justify-between gap-4">
              <p class="text-xs text-[var(--text-secondary)] truncate font-medium">{{ event.message }}</p>
              <span class="text-[10px] text-[var(--text-muted)] font-mono whitespace-nowrap">{{ event.time }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
