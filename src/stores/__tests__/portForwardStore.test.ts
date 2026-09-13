import assert from 'node:assert/strict'
import test, { beforeEach } from 'node:test'
import { createPinia, setActivePinia } from 'pinia'
import { OrbitEvents } from '../../types/events.ts'
import type { ActivePortForward } from '../../types/kubernetes.ts'
import { useKubernetesStore } from '../kubernetesStore.ts'

class MockWindow {
  private listeners = new Map<string, Array<(e: { type: string; detail: unknown }) => void>>()

  addEventListener(event: string, fn: (e: { type: string; detail: unknown }) => void) {
    const list = this.listeners.get(event) || []
    list.push(fn)
    this.listeners.set(event, list)
  }

  removeEventListener(event: string, fn: (e: { type: string; detail: unknown }) => void) {
    const list = this.listeners.get(event) || []
    this.listeners.set(
      event,
      list.filter((f) => f !== fn)
    )
  }

  dispatchEvent(evt: { type: string; detail: unknown }) {
    const list = this.listeners.get(evt.type) || []
    for (const fn of list) {
      fn(evt)
    }
    return true
  }
}

const mockWindow = new MockWindow()
// eslint-disable-next-line @typescript-eslint/no-explicit-any
;(globalThis as any).window = mockWindow

beforeEach(() => {
  setActivePinia(createPinia())
})

test('port forward store handles PortForwardStarted, PortForwardStopped, and PortForwardsUpdated', () => {
  const store = useKubernetesStore()

  const pf1: ActivePortForward = {
    id: 'default/deployment/web:8080:80',
    clusterId: 'cluster-1',
    namespace: 'default',
    kind: 'deployment',
    name: 'web',
    localPort: 8080,
    remotePort: 80
  }

  const pf2: ActivePortForward = {
    id: 'default/service/api:3000:3000',
    clusterId: 'cluster-1',
    namespace: 'default',
    kind: 'service',
    name: 'api',
    localPort: 3000,
    remotePort: 3000
  }

  // Initial state should be empty
  assert.equal(store.activePortForwards.length, 0)

  // 1. PortForwardStarted adds forward
  mockWindow.dispatchEvent({ type: OrbitEvents.PortForwardStarted, detail: pf1 })
  assert.equal(store.activePortForwards.length, 1)
  assert.equal(store.activePortForwards[0]?.id, pf1.id)

  // Duplicate start should not add duplicate item
  mockWindow.dispatchEvent({ type: OrbitEvents.PortForwardStarted, detail: pf1 })
  assert.equal(store.activePortForwards.length, 1)

  // 2. PortForwardsUpdated bulk updates active forwards
  mockWindow.dispatchEvent({
    type: OrbitEvents.PortForwardsUpdated,
    detail: { portForwards: [pf1, pf2] }
  })
  assert.equal(store.activePortForwards.length, 2)
  assert.equal(store.activePortForwards[1]?.id, pf2.id)

  // 3. PortForwardStopped removes specific forward by id
  mockWindow.dispatchEvent({ type: OrbitEvents.PortForwardStopped, detail: { id: pf1.id } })
  assert.equal(store.activePortForwards.length, 1)
  assert.equal(store.activePortForwards[0]?.id, pf2.id)

  // 4. PortForwardsUpdated with empty list clears all forwards
  mockWindow.dispatchEvent({
    type: OrbitEvents.PortForwardsUpdated,
    detail: { portForwards: [] }
  })
  assert.equal(store.activePortForwards.length, 0)

  // 5. Stopping a non-existent or phantom forward does not crash or corrupt store
  mockWindow.dispatchEvent({
    type: OrbitEvents.PortForwardStopped,
    detail: { id: 'nonexistent/id' }
  })
  assert.equal(store.activePortForwards.length, 0)

  // 6. Switching cluster resets transient active forwards
  mockWindow.dispatchEvent({ type: OrbitEvents.PortForwardStarted, detail: pf1 })
  assert.equal(store.activePortForwards.length, 1)
  store.setActiveClusterId('cluster-2')
  assert.equal(store.activePortForwards.length, 0)
})
