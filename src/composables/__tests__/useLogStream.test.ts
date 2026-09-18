import test from 'node:test'
import assert from 'node:assert/strict'
import { ref } from 'vue'
import { useLogStream } from '../useLogStream.ts'

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
