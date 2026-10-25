import assert from 'node:assert/strict'
import test, { beforeEach } from 'node:test'
import { createPinia, setActivePinia } from 'pinia'
import { updaterService } from '../../services/updaterService.ts'
import { useUpdaterStore } from '../updater.ts'

beforeEach(() => {
  setActivePinia(createPinia())
})

test('updaterStore applyUpdate does nothing if manifest has no url or version', () => {
  let appliedUrl = ''
  let appliedVersion = ''

  const originalApply = updaterService.applyUpdate
  updaterService.applyUpdate = (url: string, version: string) => {
    appliedUrl = url
    appliedVersion = version
  }

  try {
    const store = useUpdaterStore()
    store.manifest = null
    store.applyUpdate()

    assert.equal(appliedUrl, '')
    assert.equal(appliedVersion, '')
    assert.equal(store.isDownloading, false)
  } finally {
    updaterService.applyUpdate = originalApply
  }
})

test('updaterStore applyUpdate calls updaterService with both url and version', () => {
  let appliedUrl = ''
  let appliedVersion = ''

  const originalApply = updaterService.applyUpdate
  updaterService.applyUpdate = (url: string, version: string) => {
    appliedUrl = url
    appliedVersion = version
  }

  try {
    const store = useUpdaterStore()
    store.manifest = {
      version: '0.14.9',
      url: 'https://github.com/vantoan1511/orbit/releases/download/v0.14.9/Orbit_0.14.9_x64-setup.exe'
    }

    store.applyUpdate()

    assert.equal(
      appliedUrl,
      'https://github.com/vantoan1511/orbit/releases/download/v0.14.9/Orbit_0.14.9_x64-setup.exe'
    )
    assert.equal(appliedVersion, '0.14.9')
    assert.equal(store.isDownloading, true)
    assert.equal(store.downloadProgress, 0)
  } finally {
    updaterService.applyUpdate = originalApply
  }
})

test('updaterStore resets download state and records error on updateInstallFailed', () => {
  let failureHandler:
    ((data: { code: 'elevationDenied' | 'launchFailed'; message: string }) => void) | null = null

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const originalOnFailed = (updaterService as any).onUpdateInstallFailed
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  ;(updaterService as any).onUpdateInstallFailed = (handler: any) => {
    failureHandler = handler
    return () => {}
  }

  try {
    const store = useUpdaterStore()
    store.initListeners()
    store.isDownloading = true
    store.downloadProgress = 50
    store.updateReady = 'app'

    assert(failureHandler !== null, 'failureHandler should be registered')
    failureHandler({ code: 'elevationDenied', message: 'Permission denied' })

    assert.equal(store.isDownloading, false)
    assert.equal(store.downloadProgress, 0)
    assert.equal(store.updateReady, null)
    assert.deepEqual(store.installError, { code: 'elevationDenied', message: 'Permission denied' })
  } finally {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ;(updaterService as any).onUpdateInstallFailed = originalOnFailed
  }
})

test('updaterStore applyUpdate clears previous installError', () => {
  const store = useUpdaterStore()
  store.installError = { code: 'elevationDenied', message: 'old error' }
  store.manifest = {
    version: '0.15.0',
    url: 'https://example.com/installer.exe'
  }
  store.applyUpdate()
  assert.equal(store.installError, null)
})

test('updaterStore initListeners registers listeners only once', () => {
  let checkFinishedCount = 0
  let downloadProgressCount = 0
  let readyCount = 0
  let failedCount = 0

  const origCheck = updaterService.onUpdateCheckFinished
  const origProgress = updaterService.onUpdateDownloadProgress
  const origReady = updaterService.onUpdateReady
  const origFailed = updaterService.onUpdateInstallFailed

  updaterService.onUpdateCheckFinished = () => {
    checkFinishedCount++
    return () => {}
  }
  updaterService.onUpdateDownloadProgress = () => {
    downloadProgressCount++
    return () => {}
  }
  updaterService.onUpdateReady = () => {
    readyCount++
    return () => {}
  }
  updaterService.onUpdateInstallFailed = () => {
    failedCount++
    return () => {}
  }

  try {
    const store = useUpdaterStore()
    store.initListeners()
    store.initListeners()

    assert.equal(checkFinishedCount, 1)
    assert.equal(downloadProgressCount, 1)
    assert.equal(readyCount, 1)
    assert.equal(failedCount, 1)
  } finally {
    updaterService.onUpdateCheckFinished = origCheck
    updaterService.onUpdateDownloadProgress = origProgress
    updaterService.onUpdateReady = origReady
    updaterService.onUpdateInstallFailed = origFailed
  }
})
