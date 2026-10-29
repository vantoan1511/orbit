import assert from 'node:assert/strict'
import test from 'node:test'
import { coreEngine } from '../nativeService.ts'
import { kubernetesService } from '../kubernetesService.ts'

test('getResourceRaw forwards requestId to coreEngine.dispatch', async () => {
  let calledEvent = ''
  let calledData: unknown = null

  const originalDispatch = coreEngine.dispatch
  coreEngine.dispatch = async (event: string, data?: unknown) => {
    calledEvent = event
    calledData = data
  }

  try {
    await kubernetesService.getResourceRaw({
      namespace: 'test-ns',
      kind: 'Deployment',
      name: 'test-deploy',
      requestId: 'req-123'
    })

    assert.equal(calledEvent, 'getResourceRaw')
    assert.deepEqual(calledData, {
      namespace: 'test-ns',
      kind: 'Deployment',
      name: 'test-deploy',
      requestId: 'req-123'
    })
  } finally {
    coreEngine.dispatch = originalDispatch
  }
})

test('applyResource forwards requestId and resource data to coreEngine.dispatch', async () => {
  let calledEvent = ''
  let calledData: unknown = null

  const originalDispatch = coreEngine.dispatch
  coreEngine.dispatch = async (event: string, data?: unknown) => {
    calledEvent = event
    calledData = data
  }

  try {
    await kubernetesService.applyResource({
      namespace: 'prod',
      kind: 'Deployment',
      name: 'api',
      data: { spec: { replicas: 3 } },
      requestId: 'edit:Deployment:prod:api:1'
    })

    assert.equal(calledEvent, 'applyResource')
    assert.deepEqual(calledData, {
      namespace: 'prod',
      kind: 'Deployment',
      name: 'api',
      data: { spec: { replicas: 3 } },
      requestId: 'edit:Deployment:prod:api:1'
    })
  } finally {
    coreEngine.dispatch = originalDispatch
  }
})
