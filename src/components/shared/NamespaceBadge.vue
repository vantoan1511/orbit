<script setup lang="ts">
const props = defineProps<{
  namespace: string
}>()
</script>

<template>
  <span
    class="font-mono px-2 py-0.5 rounded text-[10px]"
    :class="[
      props.namespace === 'kube-system'
        ? 'bg-zinc-500/10 text-zinc-400 border border-zinc-500/20'
        : props.namespace === 'monitoring'
          ? 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
          : props.namespace === 'logging'
            ? 'bg-cyan-500/10 text-cyan-400 border border-cyan-500/20'
            : 'bg-purple-500/10 text-purple-400 border border-purple-500/20'
    ]"
  >
    {{ props.namespace }}
  </span>
</template>
