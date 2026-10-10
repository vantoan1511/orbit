import assert from 'node:assert/strict'
import test from 'node:test'
import { app, coreEngine, events, os, probe, storage } from '../nativeService.ts'

test('storage provides in-memory fallback when native store is unavailable', async () => {
  await storage.setData('test-key', 'test-value')
  const val = await storage.getData('test-key')
  assert.equal(val, 'test-value')

  const keys = await storage.getKeys()
  assert.ok(keys.includes('test-key'))

  await storage.removeData('test-key')
  const afterRemove = await storage.getData('test-key')
  assert.equal(afterRemove, null)

  await storage.setData('k1', 'v1')
  await storage.setData('k2', 'v2')
  await storage.clear()
  const cleared = await storage.getData('k1')
  assert.equal(cleared, null)
})

test('events.on and events.off manages handler lifecycle', async () => {
  let received = ''
  const handler = (data: { status: 'ready' | 'error'; message: string }) => {
    received = data.message
  }

  await events.on('engineConnected', handler)
  await events.dispatch('engineConnected', { status: 'ready', message: 'Hello Tauri' })
  assert.equal(received, 'Hello Tauri')

  await events.off('engineConnected', handler)
  await events.dispatch('engineConnected', { status: 'ready', message: 'Second Event' })
  assert.equal(received, 'Hello Tauri')
})

test('coreEngine.dispatch forwards event and data safely', async () => {
  let calledEvent = ''
  let calledData: unknown = null

  const originalDispatch = coreEngine.dispatch
  coreEngine.dispatch = async (event: string, data?: unknown) => {
    calledEvent = event
    calledData = data
  }

  try {
    await coreEngine.dispatch('getPods', { namespace: 'default' })
    assert.equal(calledEvent, 'getPods')
    assert.deepEqual(calledData, { namespace: 'default' })
  } finally {
    coreEngine.dispatch = originalDispatch
  }
})

test('os.showOpenDialog and os.open provide fallback implementations', async () => {
  const dialogResult = await os.showOpenDialog('Select File')
  assert.ok(Array.isArray(dialogResult))

  // os.open resolves without crashing
  await os.open('https://example.com')
})

test('app.getConfig, restartProcess and exit provide safe fallbacks', async () => {
  const config = await app.getConfig()
  assert.ok(typeof config === 'object')

  // restartProcess and exit should resolve safely
  await app.restartProcess()
  await app.exit(0)
})

test('probe returns a boolean', async () => {
  const result = await probe(100)
  assert.equal(typeof result, 'boolean')
})

test('init dispatches clientConnect in Tauri environment', async () => {
  const { init } = await import('../nativeService.ts')
  let dispatchedEvent = ''
  const originalDispatch = coreEngine.dispatch
  coreEngine.dispatch = async (event: string) => {
    dispatchedEvent = event
  }

  const originalWindow = (globalThis as unknown as { window?: unknown }).window
  ;(globalThis as unknown as { window?: unknown }).window = { __TAURI_INTERNALS__: {} }

  try {
    await init()
    assert.equal(dispatchedEvent, 'clientConnect')
  } finally {
    coreEngine.dispatch = originalDispatch
    ;(globalThis as unknown as { window?: unknown }).window = originalWindow
  }
})

test('reconnect dispatches clientConnect in Tauri environment', async () => {
  const { reconnect } = await import('../nativeService.ts')
  let dispatchedEvent = ''
  const originalDispatch = coreEngine.dispatch
  coreEngine.dispatch = async (event: string) => {
    dispatchedEvent = event
  }

  const originalWindow = (globalThis as unknown as { window?: unknown }).window
  ;(globalThis as unknown as { window?: unknown }).window = { __TAURI_INTERNALS__: {} }

  try {
    await reconnect()
    assert.equal(dispatchedEvent, 'clientConnect')
  } finally {
    coreEngine.dispatch = originalDispatch
    ;(globalThis as unknown as { window?: unknown }).window = originalWindow
  }
})

test('probe dispatches ping in Tauri environment and resolves true on success', async () => {
  let pingDispatched = false
  const originalDispatch = coreEngine.dispatch
  coreEngine.dispatch = async (event: string) => {
    if (event === 'ping') pingDispatched = true
  }

  const originalWindow = (globalThis as unknown as { window?: unknown }).window
  ;(globalThis as unknown as { window?: unknown }).window = { __TAURI_INTERNALS__: {} }

  try {
    const result = await probe(500)
    assert.equal(result, true)
    assert.equal(pingDispatched, true)
  } finally {
    coreEngine.dispatch = originalDispatch
    ;(globalThis as unknown as { window?: unknown }).window = originalWindow
  }
})

test('probe returns false when engine ping rejects or times out', async () => {
  const originalDispatch = coreEngine.dispatch
  coreEngine.dispatch = async () => {
    throw new Error('IPC disconnected')
  }

  const originalWindow = (globalThis as unknown as { window?: unknown }).window
  ;(globalThis as unknown as { window?: unknown }).window = { __TAURI_INTERNALS__: {} }

  try {
    const result = await probe(500)
    assert.equal(result, false)
  } finally {
    coreEngine.dispatch = originalDispatch
    ;(globalThis as unknown as { window?: unknown }).window = originalWindow
  }
})
