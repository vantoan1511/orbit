import test from 'node:test'
import assert from 'node:assert/strict'
import { ref } from 'vue'
import { kubernetesService } from '../../services/kubernetesService.ts'
import { useLogStream } from '../useLogStream.ts'

test('useLogStream initializes with isRefreshing false', () => {
  const selectedNamespace = ref('default')
  const selectedWorkloadName = ref('my-app')
  const selectedWorkloadKind = ref('Deployment')
  const selectedPodName = ref('All')
  const selectedContainerName = ref('All')
  const tailLines = ref(500)

  const { isRefreshing } = useLogStream({
    selectedNamespace,
    selectedWorkloadName,
    selectedWorkloadKind,
    selectedPodName,
    selectedContainerName,
    tailLines
  })

  assert.equal(isRefreshing.value, false)
})

test('useLogStream.refreshLogs stops previous logs and restarts stream', async () => {
  const selectedNamespace = ref('default')
  const selectedWorkloadName = ref('nginx')
  const selectedWorkloadKind = ref('Deployment')
  const selectedPodName = ref('nginx-pod-1')
  const selectedContainerName = ref('nginx-main')
  const tailLines = ref(250)

  const originalStopLogs = kubernetesService.stopLogs
  const originalStreamLogs = kubernetesService.streamLogs

  let stopLogsCalled = 0
  let streamedParams: Parameters<typeof kubernetesService.streamLogs>[0] | null = null

  kubernetesService.stopLogs = async () => {
    stopLogsCalled++
  }
  kubernetesService.streamLogs = async (
    params: Parameters<typeof kubernetesService.streamLogs>[0]
  ) => {
    streamedParams = params
  }

  try {
    const { refreshLogs, isPaused } = useLogStream({
      selectedNamespace,
      selectedWorkloadName,
      selectedWorkloadKind,
      selectedPodName,
      selectedContainerName,
      tailLines
    })

    isPaused.value = true

    await refreshLogs()

    assert.equal(isPaused.value, false, 'refreshLogs should unpause stream')
    assert.equal(stopLogsCalled, 1, 'refreshLogs should call stopLogs')
    assert.deepEqual(streamedParams, {
      namespace: 'default',
      workload: 'nginx',
      kind: 'Deployment',
      pod: 'nginx-pod-1',
      container: 'nginx-main',
      tailLines: 250
    })
  } finally {
    kubernetesService.stopLogs = originalStopLogs
    kubernetesService.streamLogs = originalStreamLogs
  }
})

test('useLogStream.refreshLogs guards against concurrent invocations', async () => {
  const selectedNamespace = ref('default')
  const selectedWorkloadName = ref('nginx')
  const selectedWorkloadKind = ref('Deployment')
  const selectedPodName = ref('All')
  const selectedContainerName = ref('All')
  const tailLines = ref(500)

  const originalStopLogs = kubernetesService.stopLogs
  const originalStreamLogs = kubernetesService.streamLogs

  let streamCalls = 0
  kubernetesService.stopLogs = async () => {}
  kubernetesService.streamLogs = async () => {
    streamCalls++
  }

  try {
    const { refreshLogs, isRefreshing } = useLogStream({
      selectedNamespace,
      selectedWorkloadName,
      selectedWorkloadKind,
      selectedPodName,
      selectedContainerName,
      tailLines
    })

    // Simulate already refreshing
    isRefreshing.value = true

    await refreshLogs()

    assert.equal(streamCalls, 0, 'Concurrent refreshLogs call should be ignored')
  } finally {
    kubernetesService.stopLogs = originalStopLogs
    kubernetesService.streamLogs = originalStreamLogs
  }
})

test('useLogStream.refreshLogs resets isFollowing to true', async () => {
  const selectedNamespace = ref('default')
  const selectedWorkloadName = ref('nginx')
  const selectedWorkloadKind = ref('Deployment')
  const selectedPodName = ref('All')
  const selectedContainerName = ref('All')
  const tailLines = ref(500)

  const originalStopLogs = kubernetesService.stopLogs
  const originalStreamLogs = kubernetesService.streamLogs

  kubernetesService.stopLogs = async () => {}
  kubernetesService.streamLogs = async () => {}

  try {
    const { refreshLogs, isFollowing } = useLogStream({
      selectedNamespace,
      selectedWorkloadName,
      selectedWorkloadKind,
      selectedPodName,
      selectedContainerName,
      tailLines
    })

    isFollowing.value = false

    await refreshLogs()

    assert.equal(isFollowing.value, true, 'refreshLogs should reset isFollowing to true')
  } finally {
    kubernetesService.stopLogs = originalStopLogs
    kubernetesService.streamLogs = originalStreamLogs
  }
})

test('useLogStream.refreshLogs does not call streamLogs when workload is empty', async () => {
  const selectedNamespace = ref('default')
  const selectedWorkloadName = ref('')
  const selectedWorkloadKind = ref('Deployment')
  const selectedPodName = ref('All')
  const selectedContainerName = ref('All')
  const tailLines = ref(500)

  const originalStopLogs = kubernetesService.stopLogs
  const originalStreamLogs = kubernetesService.streamLogs

  let streamCalls = 0
  kubernetesService.stopLogs = async () => {}
  kubernetesService.streamLogs = async () => {
    streamCalls++
  }

  try {
    const { refreshLogs } = useLogStream({
      selectedNamespace,
      selectedWorkloadName,
      selectedWorkloadKind,
      selectedPodName,
      selectedContainerName,
      tailLines
    })

    await refreshLogs()

    assert.equal(streamCalls, 0, 'Should not stream logs if workload name is empty')
  } finally {
    kubernetesService.stopLogs = originalStopLogs
    kubernetesService.streamLogs = originalStreamLogs
  }
})

test('useLogStream.refreshLogs resets isRefreshing to false after delay', async () => {
  const selectedNamespace = ref('default')
  const selectedWorkloadName = ref('nginx')
  const selectedWorkloadKind = ref('Deployment')
  const selectedPodName = ref('All')
  const selectedContainerName = ref('All')
  const tailLines = ref(500)

  const originalStopLogs = kubernetesService.stopLogs
  const originalStreamLogs = kubernetesService.streamLogs

  kubernetesService.stopLogs = async () => {}
  kubernetesService.streamLogs = async () => {}

  try {
    const { refreshLogs, isRefreshing } = useLogStream({
      selectedNamespace,
      selectedWorkloadName,
      selectedWorkloadKind,
      selectedPodName,
      selectedContainerName,
      tailLines
    })

    await refreshLogs()

    assert.equal(
      isRefreshing.value,
      true,
      'isRefreshing should be true immediately after refreshLogs'
    )

    await new Promise((resolve) => setTimeout(resolve, 450))

    assert.equal(isRefreshing.value, false, 'isRefreshing should reset to false after timeout')
  } finally {
    kubernetesService.stopLogs = originalStopLogs
    kubernetesService.streamLogs = originalStreamLogs
  }
})
