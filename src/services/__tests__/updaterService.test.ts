import assert from 'node:assert/strict'
import test from 'node:test'
import { coreEngine } from '../nativeService.ts'
import { updaterService } from '../updaterService.ts'

test('updaterService dispatches checkForUpdates with empty payload', async () => {
  let dispatchedEvent = ''
  let dispatchedData: unknown = null

  const originalDispatch = coreEngine.dispatch
  coreEngine.dispatch = async (event: string, data?: unknown) => {
    dispatchedEvent = event
    dispatchedData = data
  }

  try {
    updaterService.checkForUpdates()
    assert.equal(dispatchedEvent, 'checkForUpdates')
    assert.deepEqual(dispatchedData, {})
  } finally {
    coreEngine.dispatch = originalDispatch
  }
})

test('updaterService dispatches applyUpdate with url and version', async () => {
  let dispatchedEvent = ''
  let dispatchedData: unknown = null

  const originalDispatch = coreEngine.dispatch
  coreEngine.dispatch = async (event: string, data?: unknown) => {
    dispatchedEvent = event
    dispatchedData = data
  }

  try {
    updaterService.applyUpdate('https://github.com/vantoan1511/orbit/releases/download/v0.14.9/Orbit_0.14.9_x64-setup.exe', '0.14.9')
    assert.equal(dispatchedEvent, 'applyUpdate')
    assert.deepEqual(dispatchedData, {
      url: 'https://github.com/vantoan1511/orbit/releases/download/v0.14.9/Orbit_0.14.9_x64-setup.exe',
      version: '0.14.9'
    })
  } finally {
    coreEngine.dispatch = originalDispatch
  }
})
