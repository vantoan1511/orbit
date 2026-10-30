import assert from 'node:assert/strict'
import test from 'node:test'
import {
  extractPVFormValues,
  applyPVFormToResource,
  extractPVCFormValues,
  applyPVCFormToResource,
  extractStorageClassFormValues,
  applyStorageClassFormToResource,
  buildCreatePVManifest,
  validateCreatePV,
  buildCreatePVCManifest,
  validateCreatePVC,
  buildCreateStorageClassManifest,
  validateCreateStorageClass
} from '../storageForms.ts'

test('PV form extractor and merger preserves volume source and omits namespace', () => {
  const initialPV = {
    apiVersion: 'v1',
    kind: 'PersistentVolume',
    metadata: {
      name: 'pv-local-1',
      labels: { env: 'prod' },
      annotations: { note: 'test' },
      finalizers: ['kubernetes.io/pv-protection']
    },
    spec: {
      capacity: { storage: '10Gi' },
      accessModes: ['ReadWriteOnce'],
      persistentVolumeReclaimPolicy: 'Retain',
      storageClassName: 'manual',
      volumeMode: 'Filesystem',
      mountOptions: ['ro'],
      hostPath: { path: '/data/db' },
      claimRef: { namespace: 'default', name: 'my-claim' }
    }
  }

  const formValues = extractPVFormValues(initialPV as Record<string, unknown>)
  assert.equal(formValues.name, 'pv-local-1')
  assert.equal(formValues.capacity, '10Gi')
  assert.deepEqual(formValues.accessModes, ['ReadWriteOnce'])
  assert.equal(formValues.reclaimPolicy, 'Retain')
  assert.equal(formValues.storageClassName, 'manual')
  assert.equal(formValues.volumeMode, 'Filesystem')
  assert.deepEqual(formValues.mountOptions, ['ro'])
  assert.ok(formValues.sourceSummary.includes('/data/db'))
  assert.equal(formValues.claimRefSummary, 'default/my-claim')

  // Modify mutable fields
  formValues.capacity = '50Gi'
  formValues.reclaimPolicy = 'Delete'
  formValues.mountOptions = ['rw', 'sync']
  formValues.labels = [{ key: 'env', value: 'staging' }]

  const updatedPV = applyPVFormToResource(initialPV as Record<string, unknown>, formValues)
  assert.equal(updatedPV.metadata?.name, 'pv-local-1')
  assert.equal(updatedPV.metadata?.namespace, undefined) // Cluster-scoped!
  assert.deepEqual(updatedPV.metadata?.labels, { env: 'staging' })
  assert.deepEqual(updatedPV.metadata?.finalizers, ['kubernetes.io/pv-protection'])
  assert.equal(updatedPV.spec?.capacity?.storage, '50Gi')
  assert.equal(updatedPV.spec?.persistentVolumeReclaimPolicy, 'Delete')
  assert.deepEqual(updatedPV.spec?.mountOptions, ['rw', 'sync'])
  // Existing volume source and claimRef must be preserved
  assert.deepEqual((updatedPV.spec as Record<string, unknown>).hostPath, { path: '/data/db' })
  assert.deepEqual((updatedPV.spec as Record<string, unknown>).claimRef, {
    namespace: 'default',
    name: 'my-claim'
  })
})

test('PVC form extractor and merger preserves immutable fields', () => {
  const initialPVC = {
    apiVersion: 'v1',
    kind: 'PersistentVolumeClaim',
    metadata: {
      name: 'pvc-data',
      namespace: 'production',
      labels: { app: 'web' },
      annotations: {}
    },
    spec: {
      accessModes: ['ReadWriteMany'],
      resources: {
        requests: { storage: '20Gi' }
      },
      storageClassName: 'fast-ssd',
      volumeName: 'pv-001',
      volumeMode: 'Filesystem'
    }
  }

  const formValues = extractPVCFormValues(initialPVC as Record<string, unknown>)
  assert.equal(formValues.name, 'pvc-data')
  assert.equal(formValues.namespace, 'production')
  assert.equal(formValues.storageRequest, '20Gi')
  assert.deepEqual(formValues.accessModes, ['ReadWriteMany'])
  assert.equal(formValues.storageClassName, 'fast-ssd')
  assert.equal(formValues.volumeName, 'pv-001')

  // Update storage expansion and labels
  formValues.storageRequest = '100Gi'
  formValues.labels = [{ key: 'tier', value: 'gold' }]

  const updatedPVC = applyPVCFormToResource(initialPVC as Record<string, unknown>, formValues)
  assert.equal(updatedPVC.metadata?.name, 'pvc-data')
  assert.equal(updatedPVC.metadata?.namespace, 'production')
  assert.deepEqual(updatedPVC.metadata?.labels, { tier: 'gold' })
  assert.equal(updatedPVC.spec?.resources?.requests?.storage, '100Gi')
  // Immutable fields must remain unchanged
  assert.deepEqual(updatedPVC.spec?.accessModes, ['ReadWriteMany'])
  assert.equal(updatedPVC.spec?.storageClassName, 'fast-ssd')
  assert.equal(updatedPVC.spec?.volumeName, 'pv-001')
  assert.equal(updatedPVC.spec?.volumeMode, 'Filesystem')
})

test('StorageClass form extractor and merger handles top-level fields and default class annotation', () => {
  const initialSC = {
    apiVersion: 'storage.k8s.io/v1',
    kind: 'StorageClass',
    metadata: {
      name: 'standard',
      annotations: {
        'storageclass.kubernetes.io/is-default-class': 'true'
      }
    },
    provisioner: 'kubernetes.io/no-provisioner',
    reclaimPolicy: 'Delete',
    volumeBindingMode: 'WaitForFirstConsumer',
    allowVolumeExpansion: true,
    parameters: { type: 'gp3' },
    mountOptions: ['debug']
  }

  const formValues = extractStorageClassFormValues(initialSC as Record<string, unknown>)
  assert.equal(formValues.name, 'standard')
  assert.equal(formValues.isDefaultClass, true)
  assert.equal(formValues.provisioner, 'kubernetes.io/no-provisioner')
  assert.equal(formValues.allowVolumeExpansion, true)
  assert.deepEqual(formValues.mountOptions, ['debug'])
  assert.deepEqual(formValues.parameters, [{ key: 'type', value: 'gp3' }])

  // Toggle default class to false and expansion to false
  formValues.isDefaultClass = false
  formValues.allowVolumeExpansion = false
  formValues.mountOptions = []

  const updatedSC = applyStorageClassFormToResource(
    initialSC as Record<string, unknown>,
    formValues
  )
  assert.equal(updatedSC.metadata?.name, 'standard')
  assert.equal(updatedSC.metadata?.namespace, undefined) // Cluster-scoped!
  assert.equal(
    updatedSC.metadata?.annotations?.['storageclass.kubernetes.io/is-default-class'],
    undefined
  )
  assert.equal(updatedSC.allowVolumeExpansion, false)
  assert.deepEqual(updatedSC.mountOptions, [])
  // Top-level immutable fields preserved
  assert.equal(updatedSC.provisioner, 'kubernetes.io/no-provisioner')
  assert.equal(updatedSC.reclaimPolicy, 'Delete')
  assert.equal(updatedSC.volumeBindingMode, 'WaitForFirstConsumer')
  assert.deepEqual(updatedSC.parameters, { type: 'gp3' })
})

test('Create builders and validators for PV, PVC, and StorageClass', () => {
  // 1. PV
  const pvValidationInvalid = validateCreatePV({
    name: 'Invalid_PV_Name',
    capacity: 'not-a-size',
    accessModes: [],
    reclaimPolicy: 'Retain',
    volumeMode: 'Filesystem',
    sourceType: 'hostPath',
    hostPath: ''
  })
  assert.equal(pvValidationInvalid.valid, false)
  assert.ok(pvValidationInvalid.errors.name)
  assert.ok(pvValidationInvalid.errors.capacity)
  assert.ok(pvValidationInvalid.errors.accessModes)
  assert.ok(pvValidationInvalid.errors.hostPath)

  const pvValidationValid = validateCreatePV({
    name: 'pv-valid',
    capacity: '20Gi',
    accessModes: ['ReadWriteOnce'],
    reclaimPolicy: 'Retain',
    volumeMode: 'Filesystem',
    sourceType: 'hostPath',
    hostPath: '/mnt/storage'
  })
  assert.equal(pvValidationValid.valid, true)

  const pvManifest = buildCreatePVManifest({
    name: 'pv-valid',
    capacity: '20Gi',
    accessModes: ['ReadWriteOnce'],
    reclaimPolicy: 'Retain',
    volumeMode: 'Filesystem',
    storageClassName: 'standard',
    sourceType: 'hostPath',
    hostPath: '/mnt/storage',
    labels: [{ key: 'type', value: 'local' }]
  })
  const pvMeta = pvManifest.metadata as Record<string, unknown>
  const pvSpec = pvManifest.spec as Record<string, unknown>
  assert.equal(pvMeta.name, 'pv-valid')
  assert.equal(pvMeta.namespace, undefined)
  assert.equal((pvSpec.capacity as Record<string, unknown>).storage, '20Gi')
  assert.equal((pvSpec.hostPath as Record<string, unknown>).path, '/mnt/storage')

  // 2. PVC
  const pvcValidationValid = validateCreatePVC({
    name: 'pvc-valid',
    namespace: 'dev',
    storageRequest: '10Gi',
    accessModes: ['ReadWriteOnce'],
    volumeMode: 'Filesystem'
  })
  assert.equal(pvcValidationValid.valid, true)

  const pvcManifest = buildCreatePVCManifest({
    name: 'pvc-valid',
    namespace: 'dev',
    storageRequest: '10Gi',
    accessModes: ['ReadWriteOnce'],
    volumeMode: 'Filesystem',
    storageClassName: 'fast'
  })
  const pvcMeta = pvcManifest.metadata as Record<string, unknown>
  const pvcSpec = pvcManifest.spec as Record<string, unknown>
  const pvcRequests = (pvcSpec.resources as Record<string, unknown>).requests as Record<
    string,
    unknown
  >
  assert.equal(pvcMeta.name, 'pvc-valid')
  assert.equal(pvcMeta.namespace, 'dev')
  assert.equal(pvcRequests.storage, '10Gi')

  // 3. StorageClass
  const scValidationValid = validateCreateStorageClass({
    name: 'sc-valid',
    provisioner: 'rancher.io/local-path',
    reclaimPolicy: 'Delete',
    volumeBindingMode: 'WaitForFirstConsumer',
    allowVolumeExpansion: true,
    isDefaultClass: true
  })
  assert.equal(scValidationValid.valid, true)

  const scManifest = buildCreateStorageClassManifest({
    name: 'sc-valid',
    provisioner: 'rancher.io/local-path',
    reclaimPolicy: 'Delete',
    volumeBindingMode: 'WaitForFirstConsumer',
    allowVolumeExpansion: true,
    isDefaultClass: true,
    parameters: [{ key: 'type', value: 'local' }]
  })
  const scMeta = scManifest.metadata as Record<string, unknown>
  const scAnnotations = scMeta.annotations as Record<string, unknown>
  assert.equal(scMeta.name, 'sc-valid')
  assert.equal(scAnnotations['storageclass.kubernetes.io/is-default-class'], 'true')
  assert.equal(scManifest.provisioner, 'rancher.io/local-path')
  assert.deepEqual(scManifest.parameters, { type: 'local' })
})
