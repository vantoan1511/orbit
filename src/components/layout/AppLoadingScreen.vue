<script setup lang="ts">
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import { Lightbulb, Rocket } from '@lucide/vue'
import { onMounted, onUnmounted, ref } from 'vue'

const emit = defineEmits<{
  (e: 'complete'): void
}>()

const k8sStore = useKubernetesStore()

const facts = [
  'Kubernetes was originally designed by Google and is now an open source CNCF project.',
  'The name Kubernetes originates from Greek, meaning helmsman or pilot.',
  'K8s is an abbreviation derived by replacing the 8 letters between K and s.',
  'Kubernetes 1.0 was released on July 21, 2015 along with the CNCF.',
  'Borg and Omega, Google internal container systems, directly inspired Kubernetes.',
  'Kubernetes supports multiple container runtimes through the Container Runtime Interface (CRI).',
  'Etcd is the consistent and highly-available key value store used as backing store.',
  'A Pod is the smallest execution unit in Kubernetes, containing one or more containers.'
]

const tips = [
  'Tip: You can quickly switch between clusters using the cluster switcher in the top bar.',
  'Tip: Right-click on any resource in a table to open the quick action menu.',
  'Tip: Use the namespace selector to filter workloads by specific namespaces.',
  'Tip: Click on any Pod or Workload to open details drawer and inspect live YAML or logs.',
  'Tip: Orbit connects directly to your local kubeconfig without transmitting credentials.',
  'Tip: Customize visible columns in resource tables using the table settings button.',
  'Tip: You can search across resource names, images, and labels using quick search.',
  'Tip: Switch between dark and light themes in the Settings menu.'
]

const currentFactIndex = ref(0)
const currentTipIndex = ref(0)
const progress = ref(0)

let rolloutInterval: number | undefined
let progressInterval: number | undefined
let finishTimeout: number | undefined

const startTime = Date.now()
const MIN_DISPLAY_TIME_MS = 500
const ENGINE_TIMEOUT_MS = 15000

onMounted(() => {
  // Roll out facts and tips every 3 seconds
  rolloutInterval = window.setInterval(() => {
    currentFactIndex.value = Math.floor(Math.random() * facts.length)
    currentTipIndex.value = Math.floor(Math.random() * tips.length)
  }, 3000)

  // Smooth progress bar logic
  progressInterval = window.setInterval(() => {
    const elapsed = Date.now() - startTime
    const engineReady = k8sStore.isEngineReady

    if (!engineReady) {
      if (elapsed > ENGINE_TIMEOUT_MS) {
        window.clearInterval(progressInterval)
        finishTimeout = window.setTimeout(() => {
          k8sStore.setAppLoading(false)
          emit('complete')
          window.setTimeout(() => {
            events.dispatch(OrbitEvents.EngineTimeout, {})
          }, 500)
        }, 350)
        return
      }

      if (progress.value < 90) {
        const step = Math.max(1, Math.floor((90 - progress.value) * 0.15))
        progress.value = Math.min(90, progress.value + step)
      }
    } else if (elapsed < MIN_DISPLAY_TIME_MS) {
      // Gradually creep towards 90%
      if (progress.value < 90) {
        const step = Math.max(1, Math.floor((90 - progress.value) * 0.15))
        progress.value = Math.min(90, progress.value + step)
      }
    } else {
      // Engine ready & minimum display time reached -> animate to 100%
      if (progress.value < 100) {
        progress.value = Math.min(100, progress.value + 10)
      } else {
        window.clearInterval(progressInterval)
        finishTimeout = window.setTimeout(() => {
          k8sStore.setAppLoading(false)
          emit('complete')
        }, 350)
      }
    }
  }, 80)
})

onUnmounted(() => {
  if (rolloutInterval) window.clearInterval(rolloutInterval)
  if (progressInterval) window.clearInterval(progressInterval)
  if (finishTimeout) window.clearTimeout(finishTimeout)
})
</script>

<template>
  <div
    class="flex flex-col items-center justify-center h-screen w-screen text-primary font-sans select-none overflow-hidden p-8"
  >
    <!-- Center branding -->
    <div class="flex flex-col items-center justify-center">
      <div class="relative w-32 h-32 flex items-center justify-center mb-4">
        <img src="/logo.png" alt="Orbit" class="w-24 h-24 object-contain relative z-10" />
      </div>

      <h1 class="text-4xl font-bold tracking-tight text-primary font-ui mb-2">Orbit</h1>

      <div class="flex flex-col items-center gap-1.5 text-xs font-semibold tracking-[0.2em] mb-12">
        <span class="text-muted-color">EXPLORE. MONITOR. MANAGE.</span>
        <span class="text-primary">KUBERNETES.</span>
      </div>
    </div>

    <!-- Above bar: Fact -->
    <div class="flex items-start gap-3 w-full max-w-md min-h-12 text-left mb-6">
      <Lightbulb class="w-5 h-5 text-primary shrink-0 mt-0.5" />
      <Transition name="rollout" mode="out-in">
        <p :key="currentFactIndex" class="text-xs text-muted-color leading-relaxed">
          {{ facts[currentFactIndex] }}
        </p>
      </Transition>
    </div>

    <!-- Progress bar -->
    <div class="w-full max-w-md h-2 rounded-full overflow-hidden my-2">
      <div
        class="h-full bg-primary rounded-full transition-all duration-300 ease-out"
        :style="{ width: `${progress}%` }"
      ></div>
    </div>

    <!-- Below bar: Tip -->
    <div class="flex items-start gap-3 w-full max-w-md min-h-12 text-left mt-6">
      <Rocket class="w-5 h-5 text-primary shrink-0 mt-0.5" />
      <Transition name="rollout" mode="out-in">
        <p :key="currentTipIndex" class="text-xs text-muted-color leading-relaxed">
          {{ tips[currentTipIndex] }}
        </p>
      </Transition>
    </div>
  </div>
</template>

<style scoped>
.rollout-enter-active,
.rollout-leave-active {
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

.rollout-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.rollout-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
