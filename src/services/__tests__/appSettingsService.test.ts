import assert from 'node:assert/strict'
import test from 'node:test'
import { appSettingsService } from '../appSettingsService.ts'
import { coreEngine } from '../nativeService.ts'

test('appSettingsService dispatches getAppSettings', async () => {
  let dispatchedEvent = ''

  const originalDispatch = coreEngine.dispatch
  coreEngine.dispatch = async (event: string) => {
    dispatchedEvent = event
  }

  try {
    await appSettingsService.getAppSettings()
    assert.equal(dispatchedEvent, 'getAppSettings')
  } finally {
    coreEngine.dispatch = originalDispatch
  }
})

test('appSettingsService dispatches setLaunchOnStartup with enabled: true', async () => {
  let dispatchedEvent = ''
  let dispatchedData: unknown = null

  const originalDispatch = coreEngine.dispatch
  coreEngine.dispatch = async (event: string, data?: unknown) => {
    dispatchedEvent = event
    dispatchedData = data
  }

  try {
    await appSettingsService.setLaunchOnStartup(true)
    assert.equal(dispatchedEvent, 'setLaunchOnStartup')
    assert.deepEqual(dispatchedData, { enabled: true })
  } finally {
    coreEngine.dispatch = originalDispatch
  }
})

test('appSettingsService dispatches setLaunchOnStartup with enabled: false', async () => {
  let dispatchedEvent = ''
  let dispatchedData: unknown = null

  const originalDispatch = coreEngine.dispatch
  coreEngine.dispatch = async (event: string, data?: unknown) => {
    dispatchedEvent = event
    dispatchedData = data
  }

  try {
    await appSettingsService.setLaunchOnStartup(false)
    assert.equal(dispatchedEvent, 'setLaunchOnStartup')
    assert.deepEqual(dispatchedData, { enabled: false })
  } finally {
    coreEngine.dispatch = originalDispatch
  }
})
