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

    assert.equal(appliedUrl, 'https://github.com/vantoan1511/orbit/releases/download/v0.14.9/Orbit_0.14.9_x64-setup.exe')
    assert.equal(appliedVersion, '0.14.9')
    assert.equal(store.isDownloading, true)
    assert.equal(store.downloadProgress, 0)
  } finally {
    updaterService.applyUpdate = originalApply
  }
})
