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
