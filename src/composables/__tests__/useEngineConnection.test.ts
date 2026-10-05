import assert from 'node:assert/strict'
import test from 'node:test'
import {
  ENGINE_BACKOFF_SCHEDULE,
  MAX_RECONNECT_DURATION_MS,
  createEngineConnectionManager
} from '../useEngineConnection.ts'

test('createEngineConnectionManager starts in connected state', () => {
  const manager = createEngineConnectionManager({
    isNeutralinoEnvironment: () => true,
    probe: async () => true,
    reconnect: () => {},
    restartProcess: async () => {},
    now: () => 1000
  })

  assert.equal(manager.state.value, 'connected')
  assert.equal(manager.isReconnecting.value, false)
})

test('createEngineConnectionManager is no-op outside Neutralino environment', async () => {
  let reconnectCalls = 0
  const manager = createEngineConnectionManager({
    isNeutralinoEnvironment: () => false,
    probe: async () => false,
    reconnect: () => {
      reconnectCalls++
    },
    restartProcess: async () => {},
    now: () => 1000
  })

  assert.equal(manager.state.value, 'connected')
  await manager.handleServerOffline()
  assert.equal(reconnectCalls, 0)
  assert.equal(manager.state.value, 'connected')
})

test('serverOffline triggers reconnect attempt and transitions to reconnecting', async () => {
  let reconnectCalls = 0
  let currentTime = 1000

  const manager = createEngineConnectionManager({
    isNeutralinoEnvironment: () => true,
    probe: async () => false,
    reconnect: () => {
      reconnectCalls++
    },
    restartProcess: async () => {},
    now: () => currentTime
  })

  manager.handleServerOffline()
  assert.equal(manager.state.value, 'reconnecting')
  assert.equal(manager.isReconnecting.value, true)

  // Fast forward past initial backoff (1000ms)
  currentTime += 1050
  await manager.tick()
  assert.equal(reconnectCalls, 1)
})

test('consecutive failures follow backoff sequence [1000, 2000, 4000, 8000, 15000]', async () => {
  let reconnectCalls = 0
  let currentTime = 0

  const manager = createEngineConnectionManager({
    isNeutralinoEnvironment: () => true,
    probe: async () => false,
    reconnect: () => {
      reconnectCalls++
    },
    restartProcess: async () => {},
    now: () => currentTime
  })

  manager.handleServerOffline()
  assert.deepEqual(ENGINE_BACKOFF_SCHEDULE, [1000, 2000, 4000, 8000, 15000])

  // Delays: 1s, 2s, 4s, 8s, 15s
  for (let i = 0; i < ENGINE_BACKOFF_SCHEDULE.length; i++) {
    currentTime += ENGINE_BACKOFF_SCHEDULE[i] + 10
    await manager.tick()
    assert.equal(reconnectCalls, i + 1)
  }
})

test('successful probe or engineConnected resets state to connected and resets backoff', async () => {
  let reconnectCalls = 0
  const isHealthy = false

  const manager = createEngineConnectionManager({
    isNeutralinoEnvironment: () => true,
    probe: async () => isHealthy,
    reconnect: () => {
      reconnectCalls++
    },
    restartProcess: async () => {},
    now: () => 1000
  })

  manager.handleServerOffline()
  assert.equal(manager.state.value, 'reconnecting')

  // When engineConnected event arrives
  manager.handleEngineConnected()
  assert.equal(manager.state.value, 'connected')
  assert.equal(manager.isReconnecting.value, false)
  assert.equal(reconnectCalls, 0)
})

test('exceeding MAX_RECONNECT_DURATION_MS transitions state to failed', async () => {
  let currentTime = 0

  const manager = createEngineConnectionManager({
    isNeutralinoEnvironment: () => true,
    probe: async () => false,
    reconnect: () => {},
    restartProcess: async () => {},
    now: () => currentTime
  })

  manager.handleServerOffline()
  assert.equal(manager.state.value, 'reconnecting')

  // Advance time past 60s
  currentTime += MAX_RECONNECT_DURATION_MS + 1000
  await manager.tick()

  assert.equal(manager.state.value, 'failed')
  assert.equal(manager.isFailed.value, true)
})

test('retryNow triggers immediate probe and reconnect', async () => {
  let reconnectCalls = 0
  const manager = createEngineConnectionManager({
    isNeutralinoEnvironment: () => true,
    probe: async () => false,
    reconnect: () => {
      reconnectCalls++
    },
    restartProcess: async () => {},
    now: () => 1000
  })

  await manager.retryNow()
  assert.equal(reconnectCalls, 1)
  assert.equal(manager.state.value, 'reconnecting')
})

test('restartApp calls restartProcess', async () => {
  let restartCalled = false
  const manager = createEngineConnectionManager({
    isNeutralinoEnvironment: () => true,
    probe: async () => false,
    reconnect: () => {},
    restartProcess: async () => {
      restartCalled = true
    },
    now: () => 1000
  })

  await manager.restartApp()
  assert.equal(restartCalled, true)
})
