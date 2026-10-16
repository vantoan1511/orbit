import assert from 'node:assert/strict'
import test from 'node:test'
import { windowService } from '../../services/nativeService.ts'
import { useWindowControls } from '../useWindowControls.ts'

test('useWindowControls delegates actions to windowService', async () => {
  let actionCalled = ''
  let maximizedState = false

  const mockWindow = {
    minimize: async () => {
      actionCalled = 'minimize'
    },
    maximize: async () => {
      actionCalled = 'maximize'
      maximizedState = true
    },
    unmaximize: async () => {
      actionCalled = 'unmaximize'
      maximizedState = false
    },
    toggleMaximize: async () => {
      actionCalled = 'toggleMaximize'
      maximizedState = !maximizedState
    },
    close: async () => {
      actionCalled = 'close'
    },
    isMaximized: async () => {
      return maximizedState
    },
    startDragging: async () => {
      actionCalled = 'startDragging'
    }
  }

  windowService.__setMockWindow?.(mockWindow)

  try {
    const { isMaximized, minimize, toggleMaximize, close, startDragging, checkMaximized } =
      useWindowControls()

    assert.equal(isMaximized.value, false)

    await minimize()
    assert.equal(actionCalled, 'minimize')

    await toggleMaximize()
    assert.equal(actionCalled, 'toggleMaximize')
    assert.equal(isMaximized.value, true)

    await toggleMaximize()
    assert.equal(actionCalled, 'toggleMaximize')
    assert.equal(isMaximized.value, false)

    await close()
    assert.equal(actionCalled, 'close')

    await startDragging()
    assert.equal(actionCalled, 'startDragging')

    maximizedState = true
    await checkMaximized()
    assert.equal(isMaximized.value, true)
  } finally {
    windowService.__setMockWindow?.(null)
  }
})
