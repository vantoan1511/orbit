import { ref, computed, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useKubernetesStore } from '@/stores/kubernetesStore'

export function useLogSelection() {
  const route = useRoute()
  const k8sStore = useKubernetesStore()

  const selectedNamespace = ref<string>((route.query.namespace as string) || 'default')
  const selectedWorkloadName = ref<string>((route.query.workload as string) || '')
  const selectedWorkloadKind = ref<string>((route.query.kind as string) || 'Deployment')
  const selectedPodName = ref<string>('All')
  const selectedContainerName = ref<string>('All')
  const tailLines = ref<number>(100)

  // Options
  const namespaces = computed(() => k8sStore.namespaces.filter((n) => n !== 'All Namespaces'))

  const workloads = computed(() => {
    const ns = selectedNamespace.value
    if (selectedWorkloadKind.value === 'Deployment') {
      return k8sStore.deployments.filter((d) => d.namespace === ns).map((d) => d.name)
    } else if (selectedWorkloadKind.value === 'StatefulSet') {
      return k8sStore.statefulSets.filter((s) => s.namespace === ns).map((s) => s.name)
    } else if (selectedWorkloadKind.value === 'DaemonSet') {
      return k8sStore.daemonSets.filter((d) => d.namespace === ns).map((d) => d.name)
    } else if (selectedWorkloadKind.value === 'ReplicaSet') {
      return k8sStore.replicaSets.filter((r) => r.namespace === ns).map((r) => r.name)
    } else if (selectedWorkloadKind.value === 'Job') {
      return k8sStore.jobs.filter((j) => j.namespace === ns).map((j) => j.name)
    } else if (selectedWorkloadKind.value === 'CronJob') {
      return k8sStore.cronJobs.filter((c) => c.namespace === ns).map((c) => c.name)
    }
    return []
  })

  const workloadKinds = [
    'Deployment',
    'StatefulSet',
    'DaemonSet',
    'ReplicaSet',
    'Job',
    'CronJob',
    'Pod'
  ]

  const workloadPods = computed(() => {
    if (!selectedWorkloadName.value) return []
    if (selectedWorkloadKind.value === 'Pod') {
      const pod = k8sStore.pods.find(
        (p) => p.name === selectedWorkloadName.value && p.namespace === selectedNamespace.value
      )
      return pod ? [pod] : []
    }
    return k8sStore.pods.filter(
      (p) =>
        p.namespace === selectedNamespace.value &&
        p.name.startsWith(selectedWorkloadName.value + '-')
    )
  })

  const podOptions = computed(() => {
    return ['All', ...workloadPods.value.map((p) => p.name)]
  })

  const containerOptions = computed(() => {
    const options = new Set<string>()
    if (selectedPodName.value && selectedPodName.value !== 'All') {
      const pod = k8sStore.pods.find((p) => p.name === selectedPodName.value)
      pod?.containers?.forEach((c) => options.add(c.name))
    } else {
      workloadPods.value.forEach((p) => {
        p.containers?.forEach((c) => options.add(c.name))
      })
    }
    return ['All', ...Array.from(options)]
  })

  const tailLinesOptions = [
    { label: '50 lines', value: 50 },
    { label: '100 lines', value: 100 },
    { label: '250 lines', value: 250 },
    { label: '500 lines', value: 500 },
    { label: '1000 lines', value: 1000 }
  ]

  watch(selectedWorkloadKind, () => {
    selectedWorkloadName.value = workloads.value[0] || ''
  })

  watch(selectedNamespace, () => {
    selectedWorkloadName.value = workloads.value[0] || ''
  })

  return {
    selectedNamespace,
    selectedWorkloadName,
    selectedWorkloadKind,
    selectedPodName,
    selectedContainerName,
    tailLines,
    namespaces,
    workloads,
    workloadKinds,
    workloadPods,
    podOptions,
    containerOptions,
    tailLinesOptions
  }
}
