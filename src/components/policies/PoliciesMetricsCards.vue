<script setup lang="ts">
import { computed } from 'vue'
import { Shield, ShieldAlert, ShieldOff, AlertTriangle, FileText } from '@lucide/vue'
import { mockPolicies } from './mockPolicies'

const policies = mockPolicies

const totalCount = computed(() => policies.length)
const enforcedCount = computed(() => policies.filter((p) => p.status === 'Enforced').length)
const auditCount = computed(() => policies.filter((p) => p.status === 'Audit').length)
const disabledCount = computed(() => policies.filter((p) => p.status === 'Disabled').length)
const violationsCount = computed(() => policies.reduce((acc, p) => acc + p.violations, 0))

const enforcedPct = computed(() =>
  totalCount.value ? ((enforcedCount.value / totalCount.value) * 100).toFixed(1) : '0.0'
)
const auditPct = computed(() =>
  totalCount.value ? ((auditCount.value / totalCount.value) * 100).toFixed(1) : '0.0'
)
const disabledPct = computed(() =>
  totalCount.value ? ((disabledCount.value / totalCount.value) * 100).toFixed(1) : '0.0'
)
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-6">
    <!-- Card 1: Total Policies -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-4 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div
        class="w-10 h-10 rounded-lg bg-(--bg-hover) flex items-center justify-center text-(--text-muted)"
      >
        <FileText class="w-5 h-5" />
      </div>
      <div>
        <div class="text-[11px] font-semibold text-(--text-muted) uppercase tracking-wider">
          Total Policies
        </div>
        <div class="text-2xl font-bold text-(--text-primary) font-mono mt-0.5">
          {{ totalCount }}
        </div>
        <div class="text-[10px] text-(--text-muted) mt-0.5">Active cluster policies</div>
      </div>
    </div>

    <!-- Card 2: Enforced -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-4 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div
        class="w-10 h-10 rounded-lg bg-emerald-500/10 flex items-center justify-center text-emerald-400"
      >
        <Shield class="w-5 h-5" />
      </div>
      <div>
        <div class="text-[11px] font-semibold text-(--text-muted) uppercase tracking-wider">
          Enforced
        </div>
        <div class="text-2xl font-bold text-(--text-primary) font-mono mt-0.5">
          {{ enforcedCount }}
        </div>
        <div class="text-[10px] text-emerald-400 font-medium mt-0.5">{{ enforcedPct }}%</div>
      </div>
    </div>

    <!-- Card 3: Audit Only -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-4 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div
        class="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center text-blue-400"
      >
        <ShieldAlert class="w-5 h-5" />
      </div>
      <div>
        <div class="text-[11px] font-semibold text-(--text-muted) uppercase tracking-wider">
          Audit Only
        </div>
        <div class="text-2xl font-bold text-(--text-primary) font-mono mt-0.5">
          {{ auditCount }}
        </div>
        <div class="text-[10px] text-blue-400 font-medium mt-0.5">{{ auditPct }}%</div>
      </div>
    </div>

    <!-- Card 4: Disabled -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-4 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div
        class="w-10 h-10 rounded-lg bg-gray-500/10 flex items-center justify-center text-gray-400"
      >
        <ShieldOff class="w-5 h-5" />
      </div>
      <div>
        <div class="text-[11px] font-semibold text-(--text-muted) uppercase tracking-wider">
          Disabled
        </div>
        <div class="text-2xl font-bold text-(--text-primary) font-mono mt-0.5">
          {{ disabledCount }}
        </div>
        <div class="text-[10px] text-gray-400 font-medium mt-0.5">{{ disabledPct }}%</div>
      </div>
    </div>

    <!-- Card 5: Violations -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-4 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div class="w-10 h-10 rounded-lg bg-red-500/10 flex items-center justify-center text-red-400">
        <AlertTriangle class="w-5 h-5" />
      </div>
      <div>
        <div class="text-[11px] font-semibold text-(--text-muted) uppercase tracking-wider">
          Violations
        </div>
        <div class="text-2xl font-bold text-(--text-primary) font-mono mt-0.5">
          {{ violationsCount }}
        </div>
        <div class="text-[10px] text-red-400 font-medium mt-0.5">Last 7 days</div>
      </div>
    </div>
  </div>
</template>
