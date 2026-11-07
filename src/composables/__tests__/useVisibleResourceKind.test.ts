import assert from 'node:assert/strict'
import test from 'node:test'
import { effectScope, ref } from 'vue'
import { useVisibleResourceKind } from '../useVisibleResourceKind.ts'

test('useVisibleResourceKind: registers kind immediately on scope start', () => {
  const registeredKinds: string[] = []
  let unregisterCallCount = 0

  const fakeRegister = (kind: string) => {
    registeredKinds.push(kind)
    return () => {
      unregisterCallCount++
    }
  }

  const scope = effectScope()
  scope.run(() => {
    useVisibleResourceKind('Pod', fakeRegister)
  })

  assert.deepEqual(registeredKinds, ['Pod'])
  assert.equal(unregisterCallCount, 0)

  scope.stop()
  assert.equal(unregisterCallCount, 1)
})

test('useVisibleResourceKind: unregisters previous kind and registers new kind when reactive ref changes', () => {
  const activeKinds: string[] = []
  const unregisterCalls: string[] = []

  const fakeRegister = (kind: string) => {
    activeKinds.push(kind)
    return () => {
      unregisterCalls.push(kind)
    }
  }

  const kindRef = ref<string | undefined>('Deployment')
  const scope = effectScope()

  scope.run(() => {
    useVisibleResourceKind(kindRef, fakeRegister)
  })

  assert.deepEqual(activeKinds, ['Deployment'])
  assert.deepEqual(unregisterCalls, [])

  // Change kind to 'StatefulSet'
  kindRef.value = 'StatefulSet'

  assert.deepEqual(unregisterCalls, ['Deployment'])
  assert.deepEqual(activeKinds, ['Deployment', 'StatefulSet'])

  // Change kind to undefined
  kindRef.value = undefined
  assert.deepEqual(unregisterCalls, ['Deployment', 'StatefulSet'])

  // Stop scope
  scope.stop()
  // No additional unregister call since kind was undefined
  assert.deepEqual(unregisterCalls, ['Deployment', 'StatefulSet'])
})

test('useVisibleResourceKind: handles initial undefined kind without registering until set', () => {
  const activeKinds: string[] = []
  const unregisterCalls: string[] = []

  const fakeRegister = (kind: string) => {
    activeKinds.push(kind)
    return () => {
      unregisterCalls.push(kind)
    }
  }

  const kindRef = ref<string | undefined>(undefined)
  const scope = effectScope()

  scope.run(() => {
    useVisibleResourceKind(kindRef, fakeRegister)
  })

  assert.deepEqual(activeKinds, [])
  assert.deepEqual(unregisterCalls, [])

  kindRef.value = 'ConfigMap'
  assert.deepEqual(activeKinds, ['ConfigMap'])
  assert.deepEqual(unregisterCalls, [])

  scope.stop()
  assert.deepEqual(unregisterCalls, ['ConfigMap'])
})

test('useVisibleResourceKind: exports pause/resume lifecycle hooks and unregisters cleanly when deactivated', () => {
  const activeKinds: string[] = []
  const unregisterCalls: string[] = []

  const fakeRegister = (kind: string) => {
    activeKinds.push(kind)
    return () => {
      unregisterCalls.push(kind)
    }
  }

  const kindRef = ref<string | undefined>('Pod')
  const scope = effectScope()

  let hooks!: { pause: () => void; resume: () => void }
  scope.run(() => {
    hooks = useVisibleResourceKind(kindRef, fakeRegister)
  })

  assert.deepEqual(activeKinds, ['Pod'])
  assert.deepEqual(unregisterCalls, [])

  // Deactivate / Pause
  hooks.pause()
  assert.deepEqual(unregisterCalls, ['Pod'])

  // Calling pause again is idempotent
  hooks.pause()
  assert.deepEqual(unregisterCalls, ['Pod'])

  // While paused, changing kind does not register
  kindRef.value = 'Deployment'
  assert.deepEqual(activeKinds, ['Pod'])

  // Activate / Resume
  hooks.resume()
  assert.deepEqual(activeKinds, ['Pod', 'Deployment'])
  assert.deepEqual(unregisterCalls, ['Pod'])

  // Calling resume again is idempotent (no duplicate registration)
  hooks.resume()
  assert.deepEqual(activeKinds, ['Pod', 'Deployment'])

  scope.stop()
  assert.deepEqual(unregisterCalls, ['Pod', 'Deployment'])
})
