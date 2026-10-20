import test from 'node:test'
import assert from 'node:assert/strict'
import { ref } from 'vue'
import { kubernetesService } from '../../services/kubernetesService.ts'
import { filesystem, os } from '../../services/nativeService.ts'
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

const createMockOptions = () => ({
  selectedNamespace: ref('default'),
  selectedWorkloadName: ref('my-app'),
  selectedWorkloadKind: ref('Deployment'),
  selectedPodName: ref('All'),
  selectedContainerName: ref('All'),
  tailLines: ref(100)
})

test('useLogStream initializes with isAtBottom true and isFollowing true', () => {
  const stream = useLogStream(createMockOptions())
  assert.equal(stream.isAtBottom.value, true)
  assert.equal(stream.isFollowing.value, true)
})

test('onScroll updates isAtBottom and isFollowing when scrolled up', () => {
  const stream = useLogStream(createMockOptions())

  // Scrolled up: total 1000, current scroll 200, view 400 => remaining 400 > tolerance 20
  const mockTarget = {
    scrollHeight: 1000,
    scrollTop: 200,
    clientHeight: 400
  } as unknown as HTMLElement

  stream.onScroll({ target: mockTarget } as unknown as Event)

  assert.equal(stream.isAtBottom.value, false)
  assert.equal(stream.isFollowing.value, false)
})

test('onScroll preserves isAtBottom and isFollowing when within tolerance', () => {
  const stream = useLogStream(createMockOptions())

  // Within tolerance: total 1000, scroll 585, client 400 => diff is 15 <= 20
  const mockTarget = {
    scrollHeight: 1000,
    scrollTop: 585,
    clientHeight: 400
  } as unknown as HTMLElement

  stream.onScroll({ target: mockTarget } as unknown as Event)

  assert.equal(stream.isAtBottom.value, true)
  assert.equal(stream.isFollowing.value, true)
})

test('onScroll detects exact 20px tolerance boundary', () => {
  const stream = useLogStream(createMockOptions())

  // Exactly at boundary: 1000 - 580 - 400 = 20
  const boundaryTarget = {
    scrollHeight: 1000,
    scrollTop: 580,
    clientHeight: 400
  } as unknown as HTMLElement

  stream.onScroll({ target: boundaryTarget } as unknown as Event)
  assert.equal(stream.isAtBottom.value, true)

  // 1px beyond boundary: 1000 - 579 - 400 = 21
  const beyondBoundaryTarget = {
    scrollHeight: 1000,
    scrollTop: 579,
    clientHeight: 400
  } as unknown as HTMLElement

  stream.onScroll({ target: beyondBoundaryTarget } as unknown as Event)
  assert.equal(stream.isAtBottom.value, false)
})

test('scrollToBottom restores isAtBottom and isFollowing and scrolls container to bottom', async () => {
  const stream = useLogStream(createMockOptions())

  // First scroll up to turn off isAtBottom and isFollowing
  const scrolledUpTarget = {
    scrollHeight: 1000,
    scrollTop: 100,
    clientHeight: 400
  } as unknown as HTMLElement

  stream.onScroll({ target: scrolledUpTarget } as unknown as Event)
  assert.equal(stream.isAtBottom.value, false)
  assert.equal(stream.isFollowing.value, false)

  // Mock virtual scroller DOM element
  let targetScrollTop = 0
  const mockElement = {
    scrollHeight: 2000,
    clientHeight: 500,
    get scrollTop() {
      return targetScrollTop
    },
    set scrollTop(val: number) {
      targetScrollTop = val
    },
    scrollTo(options?: ScrollToOptions) {
      if (options && typeof options.top === 'number') {
        targetScrollTop = options.top
      }
    }
  }

  // Set mock virtualScrollerRef
  stream.virtualScrollerRef.value = {
    $el: mockElement as unknown as HTMLElement,
    element: mockElement as unknown as HTMLElement,
    scrollTo: (opts?: ScrollToOptions) => mockElement.scrollTo(opts),
    scrollToIndex: () => {},
    scrollInView: () => {},
    getRenderedRange: () => ({ first: 0, last: 0, viewport: { first: 0, last: 0 } })
  }

  stream.scrollToBottom()

  assert.equal(stream.isAtBottom.value, true)
  assert.equal(stream.isFollowing.value, true)
  assert.equal(targetScrollTop, 2000)
})

test('onScroll protects isFollowing and isAtBottom during programmatic scroll', () => {
  const stream = useLogStream(createMockOptions())

  let targetScrollTop = 0
  const mockElement = {
    scrollHeight: 2000,
    clientHeight: 500,
    get scrollTop() {
      return targetScrollTop
    },
    set scrollTop(val: number) {
      targetScrollTop = val
    },
    scrollTo(options?: ScrollToOptions) {
      if (options && typeof options.top === 'number') {
        targetScrollTop = options.top
      }
    }
  }

  stream.virtualScrollerRef.value = {
    $el: mockElement as unknown as HTMLElement,
    element: mockElement as unknown as HTMLElement,
    scrollTo: (opts?: ScrollToOptions) => mockElement.scrollTo(opts),
    scrollToIndex: () => {},
    scrollInView: () => {},
    getRenderedRange: () => ({ first: 0, last: 0, viewport: { first: 0, last: 0 } })
  }

  // Trigger programmatic scrollToBottom
  stream.scrollToBottom()

  // Intermediate scroll event fired while programmatic scrolling is in progress
  const inFlightEvent = {
    target: {
      scrollHeight: 2000,
      scrollTop: 800, // Not at bottom yet!
      clientHeight: 500
    }
  } as unknown as Event

  stream.onScroll(inFlightEvent)

  // During programmatic scrolling, in-flight events must NOT disable follow mode or flip isAtBottom
  assert.equal(stream.isFollowing.value, true)
  assert.equal(stream.isAtBottom.value, true)
})

test('handleLogLinesChunk and handleLogLine create new array references for VirtualScroller reactivity', () => {
  const stream = useLogStream(createMockOptions())

  // Initial reference
  const initialRef = stream.logLines.value
  assert.equal(initialRef.length, 0)

  // Append chunk
  stream.handleLogLinesChunk({
    pod: 'pod-1',
    container: 'container-1',
    lines: ['line 1', 'line 2']
  })

  const chunkRef = stream.logLines.value
  assert.notEqual(chunkRef, initialRef, 'handleLogLinesChunk must assign a new array reference')
  assert.equal(chunkRef.length, 2)
  assert.equal(stream.filteredLogLines.value.length, 2)
  assert.equal(stream.filteredLogLines.value, chunkRef)

  // Append single line
  stream.handleLogLine({
    pod: 'pod-1',
    container: 'container-1',
    line: 'line 3'
  })

  const singleLineRef = stream.logLines.value
  assert.notEqual(singleLineRef, chunkRef, 'handleLogLine must assign a new array reference')
  assert.equal(singleLineRef.length, 3)
  assert.equal(stream.filteredLogLines.value.length, 3)
})

test('handleLogLinesChunk trims buffer when exceeding maxLogLines', () => {
  const stream = useLogStream(createMockOptions())

  // Feed maxLogLines (2000) lines
  const batch1 = Array.from({ length: 2000 }, (_, i) => `init-line-${i}`)
  stream.handleLogLinesChunk({
    pod: 'pod-1',
    container: 'container-1',
    lines: batch1
  })
  assert.equal(stream.logLines.value.length, 2000)

  // Feed 150 more lines (exceeds maxLogLines + 100 threshold)
  const batch2 = Array.from({ length: 150 }, (_, i) => `new-line-${i}`)
  stream.handleLogLinesChunk({
    pod: 'pod-1',
    container: 'container-1',
    lines: batch2
  })

  // Buffer should be capped at maxLogLines (2000)
  assert.equal(stream.logLines.value.length, stream.maxLogLines)
  // Last line should be the latest appended line
  assert.equal(stream.logLines.value[stream.logLines.value.length - 1].text, 'new-line-149')
})

test('scheduleFollowScroll sets programmatic scroll flag and scrolls to bottom', async () => {
  const stream = useLogStream(createMockOptions())

  let targetScrollTop = 0
  const mockElement = {
    scrollHeight: 3000,
    clientHeight: 600,
    get scrollTop() {
      return targetScrollTop
    },
    set scrollTop(val: number) {
      targetScrollTop = val
    },
    scrollTo(options?: ScrollToOptions) {
      if (options && typeof options.top === 'number') {
        targetScrollTop = options.top
      }
    }
  }

  stream.virtualScrollerRef.value = {
    $el: mockElement as unknown as HTMLElement,
    element: mockElement as unknown as HTMLElement,
    scrollTo: (opts?: ScrollToOptions) => mockElement.scrollTo(opts),
    scrollToIndex: () => {},
    scrollInView: () => {},
    getRenderedRange: () => ({ first: 0, last: 0, viewport: { first: 0, last: 0 } })
  }

  // Incoming chunk while isFollowing is true triggers scheduleFollowScroll
  stream.handleLogLinesChunk({
    pod: 'pod-1',
    container: 'container-1',
    lines: ['new line']
  })

  // Await nextTick / event loop
  await new Promise((resolve) => setTimeout(resolve, 10))

  // While programmatic follow scroll is active, an intermediate scroll event should not disable follow
  const inFlightEvent = {
    target: {
      scrollHeight: 3000,
      scrollTop: 1000,
      clientHeight: 600
    }
  } as unknown as Event

  stream.onScroll(inFlightEvent)
  assert.equal(stream.isFollowing.value, true)

  // Wait for RAF to settle
  await new Promise((resolve) => setTimeout(resolve, 50))

  // Container was scrolled to bottom
  assert.equal(targetScrollTop, 3000)
})

test('useLogStream.downloadLogs returns cancelled without prompting dialog when filteredLogLines is empty', async () => {
  const stream = useLogStream(createMockOptions())
  let dialogCalled = false
  const originalShowSave = os.showSaveDialog
  os.showSaveDialog = async () => {
    dialogCalled = true
    return '/tmp/file.txt'
  }

  try {
    const result = await stream.downloadLogs()
    assert.equal(result.status, 'cancelled')
    assert.equal(dialogCalled, false)
  } finally {
    os.showSaveDialog = originalShowSave
  }
})

test('useLogStream.downloadLogs formats filtered log lines with timestamps and pod/container', async () => {
  const stream = useLogStream(createMockOptions())
  stream.logLines.value = [
    {
      pod: 'nginx-pod-1',
      container: 'nginx-main',
      text: 'Worker process started',
      timestamp: '2026-10-14 10:00:00'
    },
    {
      pod: 'nginx-pod-1',
      container: 'nginx-main',
      text: 'Configuration reloaded'
    }
  ]

  let writtenPath = ''
  let writtenContent = ''

  const originalShowSave = os.showSaveDialog
  const originalWriteText = filesystem.writeTextFile

  os.showSaveDialog = async () => '/chosen/path/logs.txt'
  filesystem.writeTextFile = async (path: string, content: string) => {
    writtenPath = path
    writtenContent = content
  }

  try {
    const result = await stream.downloadLogs()
    assert.equal(result.status, 'saved')
    assert.equal(result.filePath, '/chosen/path/logs.txt')
    assert.equal(writtenPath, '/chosen/path/logs.txt')
    const expected =
      '2026-10-14 10:00:00 [nginx-pod-1/nginx-main] Worker process started\n[nginx-pod-1/nginx-main] Configuration reloaded'
    assert.equal(writtenContent, expected)
  } finally {
    os.showSaveDialog = originalShowSave
    filesystem.writeTextFile = originalWriteText
  }
})

test('useLogStream.downloadLogs respects active search query and exports only matching lines', async () => {
  const stream = useLogStream(createMockOptions())
  stream.logLines.value = [
    {
      pod: 'pod-1',
      container: 'c1',
      text: 'Error: database connection timeout',
      timestamp: '2026-10-14 10:00:00'
    },
    {
      pod: 'pod-1',
      container: 'c1',
      text: 'Info: health check passed',
      timestamp: '2026-10-14 10:00:01'
    }
  ]

  stream.searchQuery.value = 'error'

  let writtenContent = ''
  const originalShowSave = os.showSaveDialog
  const originalWriteText = filesystem.writeTextFile

  os.showSaveDialog = async () => '/logs.txt'
  filesystem.writeTextFile = async (_path: string, content: string) => {
    writtenContent = content
  }

  try {
    const result = await stream.downloadLogs()
    assert.equal(result.status, 'saved')
    assert.equal(
      writtenContent,
      '2026-10-14 10:00:00 [pod-1/c1] Error: database connection timeout'
    )
  } finally {
    os.showSaveDialog = originalShowSave
    filesystem.writeTextFile = originalWriteText
  }
})

test('useLogStream.downloadLogs returns cancelled when user cancels save dialog', async () => {
  const stream = useLogStream(createMockOptions())
  stream.logLines.value = [{ pod: 'p1', container: 'c1', text: 'Log 1' }]

  let writeCalled = false
  const originalShowSave = os.showSaveDialog
  const originalWriteText = filesystem.writeTextFile

  os.showSaveDialog = async () => null
  filesystem.writeTextFile = async () => {
    writeCalled = true
  }

  try {
    const result = await stream.downloadLogs()
    assert.equal(result.status, 'cancelled')
    assert.equal(writeCalled, false)
  } finally {
    os.showSaveDialog = originalShowSave
    filesystem.writeTextFile = originalWriteText
  }
})

test('useLogStream.downloadLogs returns error status when filesystem write fails', async () => {
  const stream = useLogStream(createMockOptions())
  stream.logLines.value = [{ pod: 'p1', container: 'c1', text: 'Log 1' }]

  const originalShowSave = os.showSaveDialog
  const originalWriteText = filesystem.writeTextFile

  os.showSaveDialog = async () => '/fail/path.txt'
  filesystem.writeTextFile = async () => {
    throw new Error('Permission denied')
  }

  try {
    const result = await stream.downloadLogs()
    assert.equal(result.status, 'error')
    assert.ok(result.error?.includes('Permission denied'))
  } finally {
    os.showSaveDialog = originalShowSave
    filesystem.writeTextFile = originalWriteText
  }
})

test('logLines stays capped at maxLogLines after burst of single lines', () => {
  const stream = useLogStream(createMockOptions())

  for (let i = 0; i < 3000; i++) {
    stream.handleLogLine({
      pod: 'burst-pod',
      container: 'burst-container',
      line: `burst line ${i}`
    })
  }

  assert.ok(
    stream.logLines.value.length <= stream.maxLogLines + 100,
    `logLines should be capped at maxLogLines + 100, got ${stream.logLines.value.length}`
  )
  assert.equal(stream.logLines.value[stream.logLines.value.length - 1].text, 'burst line 2999')
})
