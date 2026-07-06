<script setup lang="ts">
// Data for Pod Health
const podHealth = {
  running: {
    count: 264,
    pct: 84,
    color: 'bg-emerald-500',
    text: 'text-emerald-500',
    dot: 'bg-emerald-500'
  },
  pending: {
    count: 18,
    pct: 6,
    color: 'bg-amber-500',
    text: 'text-amber-500',
    dot: 'bg-amber-500'
  },
  failed: { count: 6, pct: 2, color: 'bg-rose-500', text: 'text-rose-500', dot: 'bg-rose-500' },
  crashLoop: { count: 16, pct: 5, color: 'bg-red-600', text: 'text-red-600', dot: 'bg-red-600' }
}

// Data for Node Health
const nodeHealth = {
  ready: {
    count: 12,
    pct: 100,
    color: 'bg-emerald-500',
    text: 'text-emerald-500',
    dot: 'bg-emerald-500'
  },
  notReady: { count: 0, pct: 0, color: 'bg-rose-500', text: 'text-rose-500', dot: 'bg-rose-500' },
  cordoned: { count: 1, pct: 8, color: 'bg-sky-500', text: 'text-sky-500', dot: 'bg-sky-500' }
}
</script>

<template>
  <div class="flex flex-col gap-6">
    <!-- Pod Health Card -->
    <div
      class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 shadow-sm transition-all duration-200 flex flex-col justify-between h-[13.5rem]"
    >
      <div>
        <div class="text-xs font-semibold text-[var(--text-primary)] uppercase tracking-wider mb-4">
          Pod Health
        </div>

        <!-- Metrics Grid -->
        <div class="grid grid-cols-4 gap-2 mb-6">
          <div class="flex flex-col">
            <div class="flex items-center gap-1.5 text-xs text-[var(--text-secondary)] font-medium">
              <span class="w-2.5 h-2.5 rounded-full" :class="podHealth.running.dot"></span>
              <span>Running</span>
            </div>
            <div class="text-xl font-bold mt-1.5 text-[var(--text-primary)]">264</div>
            <div class="text-[10px] text-[var(--text-muted)] font-medium mt-0.5">84%</div>
          </div>

          <div class="flex flex-col">
            <div class="flex items-center gap-1.5 text-xs text-[var(--text-secondary)] font-medium">
              <span class="w-2.5 h-2.5 rounded-full" :class="podHealth.pending.dot"></span>
              <span>Pending</span>
            </div>
            <div class="text-xl font-bold mt-1.5 text-[var(--text-primary)]">18</div>
            <div class="text-[10px] text-[var(--text-muted)] font-medium mt-0.5">6%</div>
          </div>

          <div class="flex flex-col">
            <div class="flex items-center gap-1.5 text-xs text-[var(--text-secondary)] font-medium">
              <span class="w-2.5 h-2.5 rounded-full" :class="podHealth.failed.dot"></span>
              <span>Failed</span>
            </div>
            <div class="text-xl font-bold mt-1.5 text-[var(--text-primary)]">6</div>
            <div class="text-[10px] text-[var(--text-muted)] font-medium mt-0.5">2%</div>
          </div>

          <div class="flex flex-col">
            <div class="flex items-center gap-1.5 text-xs text-[var(--text-secondary)] font-medium">
              <span class="w-2.5 h-2.5 rounded-full" :class="podHealth.crashLoop.dot"></span>
              <span class="truncate">CrashLoop</span>
            </div>
            <div class="text-xl font-bold mt-1.5 text-[var(--text-primary)]">16</div>
            <div class="text-[10px] text-[var(--text-muted)] font-medium mt-0.5">5%</div>
          </div>
        </div>
      </div>

      <!-- Segmented Progress Bar -->
      <div class="w-full h-3 rounded-full bg-[var(--bg-hover)] overflow-hidden flex">
        <div
          :style="{ width: podHealth.running.pct + '%' }"
          :class="podHealth.running.color"
          title="Running"
        ></div>
        <div
          :style="{ width: podHealth.pending.pct + '%' }"
          :class="podHealth.pending.color"
          title="Pending"
        ></div>
        <div
          :style="{ width: podHealth.failed.pct + '%' }"
          :class="podHealth.failed.color"
          title="Failed"
        ></div>
        <div
          :style="{ width: podHealth.crashLoop.pct + '%' }"
          :class="podHealth.crashLoop.color"
          title="CrashLoop"
        ></div>
      </div>
    </div>

    <!-- Node Health Card -->
    <div
      class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 shadow-sm transition-all duration-200 flex flex-col justify-between h-[13.5rem]"
    >
      <div>
        <div class="text-xs font-semibold text-[var(--text-primary)] uppercase tracking-wider mb-4">
          Node Health
        </div>

        <!-- Metrics Grid -->
        <div class="grid grid-cols-3 gap-4 mb-6">
          <div class="flex flex-col">
            <div class="flex items-center gap-1.5 text-xs text-[var(--text-secondary)] font-medium">
              <span class="w-2.5 h-2.5 rounded-full" :class="nodeHealth.ready.dot"></span>
              <span>Ready</span>
            </div>
            <div class="text-xl font-bold mt-1.5 text-[var(--text-primary)]">12</div>
            <div class="text-[10px] text-[var(--text-muted)] font-medium mt-0.5">100%</div>
          </div>

          <div class="flex flex-col">
            <div class="flex items-center gap-1.5 text-xs text-[var(--text-secondary)] font-medium">
              <span class="w-2.5 h-2.5 rounded-full" :class="nodeHealth.notReady.dot"></span>
              <span>NotReady</span>
            </div>
            <div class="text-xl font-bold mt-1.5 text-[var(--text-primary)]">0</div>
            <div class="text-[10px] text-[var(--text-muted)] font-medium mt-0.5">0%</div>
          </div>

          <div class="flex flex-col">
            <div class="flex items-center gap-1.5 text-xs text-[var(--text-secondary)] font-medium">
              <span class="w-2.5 h-2.5 rounded-full" :class="nodeHealth.cordoned.dot"></span>
              <span>Cordoned</span>
            </div>
            <div class="text-xl font-bold mt-1.5 text-[var(--text-primary)]">1</div>
            <div class="text-[10px] text-[var(--text-muted)] font-medium mt-0.5">8%</div>
          </div>
        </div>
      </div>

      <!-- Segmented Progress Bar -->
      <div class="w-full h-3 rounded-full bg-[var(--bg-hover)] overflow-hidden flex">
        <div :style="{ width: '92%' }" :class="nodeHealth.ready.color" title="Ready"></div>
        <div :style="{ width: '8%' }" :class="nodeHealth.cordoned.color" title="Cordoned"></div>
      </div>
    </div>
  </div>
</template>
