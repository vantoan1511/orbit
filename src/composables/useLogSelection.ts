import { ref, computed, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'

export function useLogSelection() {
  const route = useRoute()
  const k8sStore = useKubernetesStore()

  const selectedNamespace = ref<string>((route.query.namespace as string) || 'default')
  const selectedWorkloadName = ref<string>((route.query.workload as string) || '')
  const selectedWorkloadKind = ref<string>(
    (route.query.kind as string) || KUBERNETES_RESOURCE_KIND.Deployment
  )
  const selectedPodName = ref<string>((route.query.pod as string) || 'All')
  const selectedContainerName = ref<string>((route.query.container as string) || 'All')
  const tailLines = ref<number>(100)

  watch(
    () => route.query,
    (query) => {
      if (query.namespace) selectedNamespace.value = query.namespace as string
      if (query.kind) selectedWorkloadKind.value = query.kind as string
      if (query.workload) selectedWorkloadName.value = query.workload as string
      if (query.pod) selectedPodName.value = query.pod as string
      if (query.container) selectedContainerName.value = query.container as string
    },
    { immediate: true, deep: true }
  )

  // Options
  const namespaces = computed(() => k8sStore.namespaces.filter((n) => n !== 'All Namespaces'))

  const workloads = computed(() => {
    const ns = selectedNamespace.value
    if (selectedWorkloadKind.value === KUBERNETES_RESOURCE_KIND.Deployment) {
      return k8sStore.deployments.filter((d) => d.namespace === ns).map((d) => d.name)
    } else if (selectedWorkloadKind.value === KUBERNETES_RESOURCE_KIND.StatefulSet) {
      return k8sStore.statefulSets.filter((s) => s.namespace === ns).map((s) => s.name)
    } else if (selectedWorkloadKind.value === KUBERNETES_RESOURCE_KIND.DaemonSet) {
      return k8sStore.daemonSets.filter((d) => d.namespace === ns).map((d) => d.name)
    } else if (selectedWorkloadKind.value === KUBERNETES_RESOURCE_KIND.ReplicaSet) {
      return k8sStore.replicaSets.filter((r) => r.namespace === ns).map((r) => r.name)
    } else if (selectedWorkloadKind.value === KUBERNETES_RESOURCE_KIND.Job) {
      return k8sStore.jobs.filter((j) => j.namespace === ns).map((j) => j.name)
    } else if (selectedWorkloadKind.value === KUBERNETES_RESOURCE_KIND.CronJob) {
      return k8sStore.cronJobs.filter((c) => c.namespace === ns).map((c) => c.name)
    }
    return []
  })

  const workloadKinds = [
    KUBERNETES_RESOURCE_KIND.Deployment,
    KUBERNETES_RESOURCE_KIND.StatefulSet,
    KUBERNETES_RESOURCE_KIND.DaemonSet,
    KUBERNETES_RESOURCE_KIND.ReplicaSet,
    KUBERNETES_RESOURCE_KIND.Job,
    KUBERNETES_RESOURCE_KIND.CronJob,
    KUBERNETES_RESOURCE_KIND.Pod
  ]

  const workloadPods = computed(() => {
    if (!selectedWorkloadName.value) return []
    if (selectedWorkloadKind.value === KUBERNETES_RESOURCE_KIND.Pod) {
      const pod = k8sStore.pods.find(
        (p) => p.name === selectedWorkloadName.value && p.namespace === selectedNamespace.value
      )
      return pod ? [pod] : []
    }
    return k8sStore.pods.filter(
      (p) =>
        p.namespace === selectedNamespace.value &&
        p.name.startsWith(`${selectedWorkloadName.value}-`)
    )
  })

  const podOptions = computed(() => ['All', ...workloadPods.value.map((p) => p.name)])

  const containerOptions = computed(() => {
    const pName = selectedPodName.value
    if (pName !== 'All') {
      const pod = workloadPods.value.find((p) => p.name === pName)
      return ['All', ...(pod?.containers?.map((c) => c.name) || [])]
    }
    const cNames = new Set<string>()
    workloadPods.value.forEach((p) => p.containers?.forEach((c) => cNames.add(c.name)))
    return ['All', ...Array.from(cNames)]
  })

  const tailLinesOptions = [
    { label: '50 lines', value: 50 },
    { label: '100 lines', value: 100 },
    { label: '500 lines', value: 500 },
    { label: '1000 lines', value: 1000 },
    { label: 'All', value: -1 }
  ]

  watch(selectedWorkloadKind, () => {
    if (
      selectedWorkloadKind.value !== KUBERNETES_RESOURCE_KIND.Pod &&
      !workloads.value.includes(selectedWorkloadName.value)
    ) {
      selectedWorkloadName.value = workloads.value[0] || ''
    }
  })

  watch(selectedNamespace, () => {
    if (
      selectedWorkloadKind.value !== KUBERNETES_RESOURCE_KIND.Pod &&
      !workloads.value.includes(selectedWorkloadName.value)
    ) {
      selectedWorkloadName.value = workloads.value[0] || ''
    }
  })

  watch(podOptions, () => {
    if (!podOptions.value.includes(selectedPodName.value)) {
      selectedPodName.value = 'All'
    }
  })

  watch(containerOptions, () => {
    if (!containerOptions.value.includes(selectedContainerName.value)) {
      selectedContainerName.value = 'All'
    }
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
