import assert from 'node:assert/strict'
import test from 'node:test'
import {
  getPodStatusBadgeClass,
  getNodeStatusBadgeClass,
  getNamespaceStatusBadgeClass
} from '../severity.ts'
import {
  KUBERNETES_NODE_STATUS,
  KUBERNETES_POD_STATUS,
  KUBERNETES_NAMESPACE_STATUS
} from '../../constants/kubernetes.ts'

test('getPodStatusBadgeClass returns semantic CSS variable classes', () => {
  assert.equal(getPodStatusBadgeClass(KUBERNETES_POD_STATUS.Running), 'bg-(--success)')
  assert.equal(getPodStatusBadgeClass(KUBERNETES_POD_STATUS.Completed), 'bg-(--success)')
  assert.equal(getPodStatusBadgeClass(KUBERNETES_POD_STATUS.Succeeded), 'bg-(--success)')

  assert.equal(getPodStatusBadgeClass(KUBERNETES_POD_STATUS.Pending), 'bg-(--warning)')
  assert.equal(getPodStatusBadgeClass(KUBERNETES_POD_STATUS.ContainerCreating), 'bg-(--warning)')

  assert.equal(getPodStatusBadgeClass(KUBERNETES_POD_STATUS.Failed), 'bg-(--danger)')
  assert.equal(getPodStatusBadgeClass(KUBERNETES_POD_STATUS.CrashLoopBackOff), 'bg-(--danger)')
  assert.equal(getPodStatusBadgeClass(KUBERNETES_POD_STATUS.Error), 'bg-(--danger)')
  assert.equal(getPodStatusBadgeClass(undefined), 'bg-muted-color')
})

test('getNodeStatusBadgeClass and getNamespaceStatusBadgeClass return semantic CSS variable classes', () => {
  assert.equal(getNodeStatusBadgeClass(KUBERNETES_NODE_STATUS.Ready), 'bg-(--success)')
  assert.equal(getNodeStatusBadgeClass(KUBERNETES_NODE_STATUS.NotReady), 'bg-(--danger)')
  assert.equal(getNodeStatusBadgeClass('Unknown'), 'bg-(--warning)')

  assert.equal(getNamespaceStatusBadgeClass(KUBERNETES_NAMESPACE_STATUS.Active), 'bg-(--success)')
  assert.equal(
    getNamespaceStatusBadgeClass(KUBERNETES_NAMESPACE_STATUS.Terminating),
    'bg-(--warning)'
  )
  assert.equal(getNamespaceStatusBadgeClass('Unknown'), 'bg-gray-400')
})
