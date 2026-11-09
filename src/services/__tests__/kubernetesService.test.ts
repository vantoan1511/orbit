import assert from 'node:assert/strict'
import test from 'node:test'
import { coreEngine, events } from '../nativeService.ts'
import { kubernetesService, CLUSTER_ADD_TIMEOUT_MS } from '../kubernetesService.ts'
import { OrbitEvents } from '../../types/events.ts'

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

test('addCluster generates requestId and resolves when ClusterAddFinished emits success', async () => {
  let dispatchedEvent = ''
  let dispatchedData: Record<string, unknown> | null = null

  const originalDispatch = coreEngine.dispatch
  coreEngine.dispatch = async (event: string, data?: unknown) => {
    dispatchedEvent = event
    dispatchedData = data as Record<string, unknown>
    setTimeout(() => {
      void events.dispatch(OrbitEvents.ClusterAddFinished, {
        requestId: dispatchedData?.requestId as string,
        success: true,
        addedContexts: ['ctx-1']
      })
    }, 10)
  }

  try {
    const res = await kubernetesService.addCluster({
      kubeconfigContent: 'test-yaml',
      selectedContexts: ['ctx-1']
    })
    assert.equal(dispatchedEvent, 'addCluster')
    assert.ok(dispatchedData?.requestId)
    assert.equal(CLUSTER_ADD_TIMEOUT_MS, 15000)
    assert.equal(res.success, true)
    assert.deepEqual(res.addedContexts, ['ctx-1'])
  } finally {
    coreEngine.dispatch = originalDispatch
  }
})

test('addCluster throws error when ClusterAddFinished emits failure', async () => {
  const originalDispatch = coreEngine.dispatch
  coreEngine.dispatch = async (_event: string, data?: unknown) => {
    const d = data as Record<string, unknown>
    setTimeout(() => {
      void events.dispatch(OrbitEvents.ClusterAddFinished, {
        requestId: d?.requestId as string,
        success: false,
        error: 'Connection refused to 127.0.0.1:6443'
      })
    }, 10)
  }

  try {
    await assert.rejects(
      async () => {
        await kubernetesService.addCluster({
          kubeconfigContent: 'invalid-yaml'
        })
      },
      {
        message: 'Connection refused to 127.0.0.1:6443'
      }
    )
  } finally {
    coreEngine.dispatch = originalDispatch
  }
})
