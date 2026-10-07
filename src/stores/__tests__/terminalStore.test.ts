import assert from 'node:assert/strict'
import test, { beforeEach } from 'node:test'
import { createPinia, setActivePinia } from 'pinia'
import { useTerminalStore } from '../terminalStore.ts'

beforeEach(() => {
  setActivePinia(createPinia())
})

test('terminalStore initializes with closed panel and empty sessions', () => {
  const store = useTerminalStore()
  assert.equal(store.isOpen, false)
  assert.deepEqual(store.sessions, [])
  assert.equal(store.activeSessionId, null)
  assert.equal(store.activeSession, undefined)
})

test('terminalStore can toggle, open, and close panel', () => {
  const store = useTerminalStore()
  store.openPanel()
  assert.equal(store.isOpen, true)

  store.closePanel()
  assert.equal(store.isOpen, false)

  store.togglePanel()
  assert.equal(store.isOpen, true)

  store.togglePanel()
  assert.equal(store.isOpen, false)
})

test('terminalStore adds sessions, opens panel, and sets active session', () => {
  const store = useTerminalStore()
  store.addSession({
    id: 'session-1',
    title: 'Pod: test-pod',
    type: 'pod',
    namespace: 'default',
    pod: 'test-pod'
  })

  assert.equal(store.isOpen, true)
  assert.equal(store.sessions.length, 1)
  assert.equal(store.activeSessionId, 'session-1')
  assert.equal(store.activeSession?.title, 'Pod: test-pod')

  store.addSession({
    id: 'session-2',
    title: 'Local Shell',
    type: 'local'
  })

  assert.equal(store.sessions.length, 2)
  assert.equal(store.activeSessionId, 'session-2')
  assert.equal(store.activeSession?.title, 'Local Shell')
})

test('terminalStore removes session and switches active session correctly', () => {
  const store = useTerminalStore()
  store.addSession({ id: 's1', title: 'Session 1', type: 'local' })
  store.addSession({ id: 's2', title: 'Session 2', type: 'local' })
  store.addSession({ id: 's3', title: 'Session 3', type: 'local' })

  assert.equal(store.activeSessionId, 's3')

  // Remove active session
  store.removeSession('s3')
  assert.equal(store.sessions.length, 2)
  assert.equal(store.activeSessionId, 's2')

  // Remove non-active session
  store.removeSession('s1')
  assert.equal(store.sessions.length, 1)
  assert.equal(store.activeSessionId, 's2')

  // Remove last session
  store.removeSession('s2')
  assert.equal(store.sessions.length, 0)
  assert.equal(store.activeSessionId, null)
  assert.equal(store.isOpen, false)
})
