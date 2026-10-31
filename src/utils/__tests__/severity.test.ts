import assert from 'node:assert/strict'
import test from 'node:test'
import {
  getPodStatusBadgeClass,
  getNodeStatusBadgeClass,
  getNamespaceStatusBadgeClass,
  getWorkloadStatusBadgeClass,
  getPolicyStatusBadgeClass,
  getVolumeStatusBadgeClass
} from '../severity.ts'
import {
  KUBERNETES_NODE_STATUS,
  KUBERNETES_POD_STATUS,
  KUBERNETES_NAMESPACE_STATUS,
  KUBERNETES_WORKLOAD_STATUS,
  KUBERNETES_JOB_STATUS,
  KUBERNETES_VOLUME_STATUS
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
  assert.equal(getNamespaceStatusBadgeClass('Unknown'), 'bg-muted-color')
})

test('getWorkloadStatusBadgeClass and getPolicyStatusBadgeClass return semantic CSS variable classes', () => {
  assert.equal(getWorkloadStatusBadgeClass(KUBERNETES_WORKLOAD_STATUS.Running), 'bg-(--success)')
  assert.equal(getWorkloadStatusBadgeClass(KUBERNETES_JOB_STATUS.Succeeded), 'bg-(--success)')
  assert.equal(getWorkloadStatusBadgeClass(KUBERNETES_JOB_STATUS.Active), 'bg-(--success)')
  assert.equal(
    getWorkloadStatusBadgeClass(KUBERNETES_WORKLOAD_STATUS.Progressing),
    'bg-(--warning)'
  )
  assert.equal(getWorkloadStatusBadgeClass(KUBERNETES_POD_STATUS.Pending), 'bg-(--warning)')
  assert.equal(getWorkloadStatusBadgeClass(KUBERNETES_WORKLOAD_STATUS.Failed), 'bg-(--danger)')
  assert.equal(getWorkloadStatusBadgeClass(KUBERNETES_POD_STATUS.CrashLoopBackOff), 'bg-(--danger)')
  assert.equal(getWorkloadStatusBadgeClass('Unknown'), 'bg-muted-color')

  assert.equal(getPolicyStatusBadgeClass('Enforced'), 'bg-(--success)')
  assert.equal(getPolicyStatusBadgeClass('Audit'), 'bg-(--info)')
  assert.equal(getPolicyStatusBadgeClass('Disabled'), 'bg-muted-color')
  assert.equal(getPolicyStatusBadgeClass(undefined), 'bg-muted-color')
})

test('getVolumeStatusBadgeClass returns semantic CSS variable classes', () => {
  assert.equal(getVolumeStatusBadgeClass(KUBERNETES_VOLUME_STATUS.Bound), 'bg-(--success)')
  assert.equal(getVolumeStatusBadgeClass(KUBERNETES_VOLUME_STATUS.Available), 'bg-(--success)')
  assert.equal(getVolumeStatusBadgeClass(KUBERNETES_VOLUME_STATUS.Pending), 'bg-(--warning)')
  assert.equal(getVolumeStatusBadgeClass(KUBERNETES_VOLUME_STATUS.Released), 'bg-(--warning)')
  assert.equal(getVolumeStatusBadgeClass(KUBERNETES_VOLUME_STATUS.Failed), 'bg-(--danger)')
  assert.equal(getVolumeStatusBadgeClass(KUBERNETES_VOLUME_STATUS.Lost), 'bg-(--danger)')
  assert.equal(getVolumeStatusBadgeClass('Unknown'), 'bg-muted-color')
  assert.equal(getVolumeStatusBadgeClass(undefined), 'bg-muted-color')
})
