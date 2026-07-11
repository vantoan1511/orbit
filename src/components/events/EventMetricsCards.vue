<script setup lang="ts">
import { computed } from 'vue'
import { Bell, CheckCircle, AlertTriangle, XCircle, Info } from '@lucide/vue'
import { mockEvents } from './mockEvents'

const events = mockEvents

const totalCount = computed(() => events.length)
const normalCount = computed(() => events.filter((e) => e.type === 'Normal').length)
const warningCount = computed(() => events.filter((e) => e.type === 'Warning').length)
const errorCount = computed(() => events.filter((e) => e.type === 'Error').length)
const otherCount = computed(
  () =>
    events.filter((e) => e.type !== 'Normal' && e.type !== 'Warning' && e.type !== 'Error').length
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
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-4 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div
        class="w-10 h-10 rounded-lg bg-(--bg-hover) flex items-center justify-center text-(--text-muted)"
      >
        <Bell class="w-5 h-5" />
      </div>
      <div>
        <div class="text-[11px] font-semibold text-(--text-muted) uppercase tracking-wider">
          Total Events
        </div>
        <div class="text-2xl font-bold text-(--text-primary) font-mono mt-0.5">
          {{ totalCount }}
        </div>
        <div class="text-[10px] text-(--text-muted) mt-0.5">Active cluster events</div>
      </div>
    </div>

    <!-- Card 2: Normal -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-4 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div
        class="w-10 h-10 rounded-lg bg-emerald-500/10 flex items-center justify-center text-emerald-400"
      >
        <CheckCircle class="w-5 h-5" />
      </div>
      <div>
        <div class="text-[11px] font-semibold text-(--text-muted) uppercase tracking-wider">
          Normal
        </div>
        <div class="text-2xl font-bold text-(--text-primary) font-mono mt-0.5">
          {{ normalCount }}
        </div>
        <div class="text-[10px] text-emerald-400 font-medium mt-0.5">{{ normalPct }}%</div>
      </div>
    </div>

    <!-- Card 3: Warning -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-4 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div
        class="w-10 h-10 rounded-lg bg-amber-500/10 flex items-center justify-center text-amber-400"
      >
        <AlertTriangle class="w-5 h-5" />
      </div>
      <div>
        <div class="text-[11px] font-semibold text-(--text-muted) uppercase tracking-wider">
          Warning
        </div>
        <div class="text-2xl font-bold text-(--text-primary) font-mono mt-0.5">
          {{ warningCount }}
        </div>
        <div class="text-[10px] text-amber-400 font-medium mt-0.5">{{ warningPct }}%</div>
      </div>
    </div>

    <!-- Card 4: Error -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-4 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div class="w-10 h-10 rounded-lg bg-red-500/10 flex items-center justify-center text-red-400">
        <XCircle class="w-5 h-5" />
      </div>
      <div>
        <div class="text-[11px] font-semibold text-(--text-muted) uppercase tracking-wider">
          Error
        </div>
        <div class="text-2xl font-bold text-(--text-primary) font-mono mt-0.5">
          {{ errorCount }}
        </div>
        <div class="text-[10px] text-red-400 font-medium mt-0.5">{{ errorPct }}%</div>
      </div>
    </div>

    <!-- Card 5: Other -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-4 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div
        class="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center text-blue-400"
      >
        <Info class="w-5 h-5" />
      </div>
      <div>
        <div class="text-[11px] font-semibold text-(--text-muted) uppercase tracking-wider">
          Other
        </div>
        <div class="text-2xl font-bold text-(--text-primary) font-mono mt-0.5">
          {{ otherCount }}
        </div>
        <div class="text-[10px] text-blue-400 font-medium mt-0.5">{{ otherPct }}%</div>
      </div>
    </div>
  </div>
</template>
