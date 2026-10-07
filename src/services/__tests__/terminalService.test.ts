import assert from 'node:assert/strict'
import test from 'node:test'
import { coreEngine } from '../nativeService.ts'
import { terminalService } from '../terminalService.ts'

test('terminalService dispatches openLocalTerminal with correct payload', async () => {
  let dispatchedEvent = ''
  let dispatchedData: unknown = null

  const originalDispatch = coreEngine.dispatch
  coreEngine.dispatch = async (event: string, data?: unknown) => {
    dispatchedEvent = event
    dispatchedData = data
  }

  try {
    await terminalService.openLocalTerminal('session-abc', 'bash')
    assert.equal(dispatchedEvent, 'openLocalTerminal')
    assert.deepEqual(dispatchedData, {
      sessionId: 'session-abc',
      shell: 'bash'
    })
  } finally {
    coreEngine.dispatch = originalDispatch
  }
})

test('terminalService dispatches openPodTerminal with correct payload', async () => {
  let dispatchedEvent = ''
  let dispatchedData: unknown = null

  const originalDispatch = coreEngine.dispatch
  coreEngine.dispatch = async (event: string, data?: unknown) => {
    dispatchedEvent = event
    dispatchedData = data
  }

  try {
    await terminalService.openPodTerminal('session-123', 'default', 'nginx-pod', 'nginx')
    assert.equal(dispatchedEvent, 'openPodTerminal')
    assert.deepEqual(dispatchedData, {
      sessionId: 'session-123',
      namespace: 'default',
      pod: 'nginx-pod',
      container: 'nginx'
    })
  } finally {
    coreEngine.dispatch = originalDispatch
  }
})

test('terminalService dispatches sendTerminalData, resizeTerminal, and closeTerminal', async () => {
  const events: Array<{ event: string; data?: unknown }> = []

  const originalDispatch = coreEngine.dispatch
  coreEngine.dispatch = async (event: string, data?: unknown) => {
    events.push({ event, data })
  }

  try {
    await terminalService.sendData('session-123', 'echo test\n')
    await terminalService.resize('session-123', 80, 24)
    await terminalService.close('session-123')

    assert.equal(events.length, 3)
    assert.deepEqual(events[0], {
      event: 'sendTerminalData',
      data: { sessionId: 'session-123', data: 'echo test\n' }
    })
    assert.deepEqual(events[1], {
      event: 'resizeTerminal',
      data: { sessionId: 'session-123', cols: 80, rows: 24 }
    })
    assert.deepEqual(events[2], {
      event: 'closeTerminal',
      data: { sessionId: 'session-123' }
    })
  } finally {
    coreEngine.dispatch = originalDispatch
  }
})
