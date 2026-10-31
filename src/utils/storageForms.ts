/**
 * Pure Storage Form Models, Manifest Builders, and Validators
 *
 * Implements form extraction, mutability enforcement, and manifest builders for:
 * - PersistentVolume (cluster-scoped, mutable capacity/reclaim/mountOptions, immutable volume source)
 * - PersistentVolumeClaim (namespaced, mutable storage expansion/metadata, immutable spec)
 * - StorageClass (cluster-scoped, top-level fields — NO spec wrapper!)
 */

import type { KubernetesResource } from '../types/kubernetes.ts'
import { isValidStorageQuantity, parseStorageBytes } from './storageQuantity.ts'
import { isValidK8sName } from './validators.ts'

export interface KeyValueItem {
  key: string
  value: string
}

function objectToKeyValueItems(obj?: Record<string, unknown> | null): KeyValueItem[] {
  if (!obj || typeof obj !== 'object') return []
  return Object.entries(obj).map(([key, value]) => ({
    key,
    value: typeof value === 'string' ? value : String(value ?? '')
  }))
}

function keyValueItemsToObject(items?: KeyValueItem[] | null): Record<string, string> {
  const result: Record<string, string> = {}
  for (const item of items || []) {
    const k = item?.key?.trim()
    if (k) {
      result[k] = item.value ?? ''
    }
  }
  return result
}

// ==========================================
// PersistentVolume (PV)
// ==========================================

export interface PVFormValues {
  name: string
  labels: KeyValueItem[]
  annotations: KeyValueItem[]
  finalizers: string[]
  capacity: string
  accessModes: string[]
  reclaimPolicy: string
  storageClassName: string
  volumeMode: string
  mountOptions: string[]
  sourceSummary: string
  claimRefSummary: string
}

export function extractPVFormValues(
  pv?: Record<string, unknown> | KubernetesResource | null
): PVFormValues {
  const meta = (pv?.metadata as Record<string, unknown>) || {}
  const spec = (pv?.spec as Record<string, unknown>) || {}
  const cap = (spec.capacity as Record<string, string>) || {}

  // Determine volume source summary
  let sourceSummary = 'Configured volume source'
  if (spec.hostPath) {
    const hp = spec.hostPath as { path?: string; type?: string }
    sourceSummary = `hostPath: ${hp.path || '-'} (${hp.type || 'Any'})`
  } else if (spec.csi) {
    const csi = spec.csi as { driver?: string; volumeHandle?: string }
    sourceSummary = `CSI: ${csi.driver || '-'} / ${csi.volumeHandle || '-'}`
  } else if (spec.nfs) {
    const nfs = spec.nfs as { server?: string; path?: string }
    sourceSummary = `NFS: ${nfs.server || '-'}:${nfs.path || '-'}`
  } else if (spec.local) {
    const local = spec.local as { path?: string }
    sourceSummary = `local: ${local.path || '-'}`
  }

  // Determine claimRef summary
  let claimRefSummary = '-'
  const claimRef = spec.claimRef as { namespace?: string; name?: string } | undefined
  if (claimRef?.name) {
    claimRefSummary = claimRef.namespace ? `${claimRef.namespace}/${claimRef.name}` : claimRef.name
  }

  const accessModes = Array.isArray(spec.accessModes)
    ? (spec.accessModes as string[])
    : ['ReadWriteOnce']

  return {
    name: (meta.name as string) || '',
    labels: objectToKeyValueItems(meta.labels as Record<string, unknown>),
    annotations: objectToKeyValueItems(meta.annotations as Record<string, unknown>),
    finalizers: Array.isArray(meta.finalizers) ? (meta.finalizers as string[]) : [],
    capacity: cap.storage || '10Gi',
    accessModes,
    reclaimPolicy: (spec.persistentVolumeReclaimPolicy as string) || 'Retain',
    storageClassName: (spec.storageClassName as string) || '',
    volumeMode: (spec.volumeMode as string) || 'Filesystem',
    mountOptions: Array.isArray(spec.mountOptions) ? (spec.mountOptions as string[]) : [],
    sourceSummary,
    claimRefSummary
  }
}

export function applyPVFormToResource(
  initialResource: Record<string, unknown> | KubernetesResource,
  form: PVFormValues
): Record<string, unknown> {
  const meta = { ...((initialResource?.metadata as Record<string, unknown>) || {}) }
  const spec = { ...((initialResource?.spec as Record<string, unknown>) || {}) }
  const cap = { ...((spec.capacity as Record<string, string>) || {}) }

  meta.name = form.name.trim()
  delete meta.namespace // PersistentVolume is strictly cluster-scoped

  const labels = keyValueItemsToObject(form.labels)
  if (Object.keys(labels).length > 0) {
    meta.labels = labels
  } else {
    delete meta.labels
  }

  const annotations = keyValueItemsToObject(form.annotations)
  if (Object.keys(annotations).length > 0) {
    meta.annotations = annotations
  } else {
    delete meta.annotations
  }

  if (form.finalizers && form.finalizers.length > 0) {
    meta.finalizers = [...form.finalizers]
  } else {
    delete meta.finalizers
  }

  // Mutable spec fields
  cap.storage = form.capacity.trim()
  spec.capacity = cap
  spec.accessModes = [...form.accessModes]
  spec.persistentVolumeReclaimPolicy = form.reclaimPolicy

  if (form.storageClassName.trim()) {
    spec.storageClassName = form.storageClassName.trim()
  } else {
    delete spec.storageClassName
  }

  if (form.mountOptions && form.mountOptions.length > 0) {
    spec.mountOptions = [...form.mountOptions]
  } else {
    delete spec.mountOptions
  }

  return {
    ...initialResource,
    metadata: meta,
    spec
  }
}

// ==========================================
// PersistentVolumeClaim (PVC)
// ==========================================

export interface PVCFormValues {
  name: string
  namespace: string
  labels: KeyValueItem[]
  annotations: KeyValueItem[]
  storageRequest: string
  accessModes: string[]
  storageClassName: string
  volumeName: string
  volumeMode: string
}

export function extractPVCFormValues(
  pvc?: Record<string, unknown> | KubernetesResource | null
): PVCFormValues {
  const meta = (pvc?.metadata as Record<string, unknown>) || {}
  const spec = (pvc?.spec as Record<string, unknown>) || {}
  const resources = (spec.resources as Record<string, unknown>) || {}
  const requests = (resources.requests as Record<string, string>) || {}

  const accessModes = Array.isArray(spec.accessModes)
    ? (spec.accessModes as string[])
    : ['ReadWriteOnce']

  return {
    name: (meta.name as string) || '',
    namespace: (meta.namespace as string) || 'default',
    labels: objectToKeyValueItems(meta.labels as Record<string, unknown>),
    annotations: objectToKeyValueItems(meta.annotations as Record<string, unknown>),
    storageRequest: requests.storage || '10Gi',
    accessModes,
    storageClassName: (spec.storageClassName as string) || '',
    volumeName: (spec.volumeName as string) || '',
    volumeMode: (spec.volumeMode as string) || 'Filesystem'
  }
}

export function applyPVCFormToResource(
  initialResource: Record<string, unknown> | KubernetesResource,
  form: PVCFormValues
): Record<string, unknown> {
  const meta = { ...((initialResource?.metadata as Record<string, unknown>) || {}) }
  const spec = { ...((initialResource?.spec as Record<string, unknown>) || {}) }
  const resources = { ...((spec.resources as Record<string, unknown>) || {}) }
  const requests = { ...((resources.requests as Record<string, string>) || {}) }

  meta.name = form.name.trim()
  if (form.namespace.trim()) {
    meta.namespace = form.namespace.trim()
  }

  const labels = keyValueItemsToObject(form.labels)
  if (Object.keys(labels).length > 0) {
    meta.labels = labels
  } else {
    delete meta.labels
  }

  const annotations = keyValueItemsToObject(form.annotations)
  if (Object.keys(annotations).length > 0) {
    meta.annotations = annotations
  } else {
    delete meta.annotations
  }

  // PVC volume expansion: update requests.storage while preserving immutable fields
  requests.storage = form.storageRequest.trim()
  resources.requests = requests
  spec.resources = resources

  return {
    ...initialResource,
    metadata: meta,
    spec
  }
}

// ==========================================
// StorageClass (SC) — Top-level fields!
// ==========================================

export interface StorageClassFormValues {
  name: string
  labels: KeyValueItem[]
  annotations: KeyValueItem[]
  isDefaultClass: boolean
  provisioner: string
  reclaimPolicy: string
  volumeBindingMode: string
  allowVolumeExpansion: boolean
  mountOptions: string[]
  parameters: KeyValueItem[]
}

export function extractStorageClassFormValues(
  sc?: Record<string, unknown> | KubernetesResource | null
): StorageClassFormValues {
  const meta = (sc?.metadata as Record<string, unknown>) || {}
  const scRecord = sc as Record<string, unknown> | undefined
  const annotations = (meta.annotations as Record<string, string>) || {}
  const isDefaultClass =
    annotations['storageclass.kubernetes.io/is-default-class'] === 'true' ||
    annotations['storageclass.beta.kubernetes.io/is-default-class'] === 'true'

  const filteredAnnotations = objectToKeyValueItems(
    meta.annotations as Record<string, unknown>
  ).filter(
    (item) =>
      item.key !== 'storageclass.kubernetes.io/is-default-class' &&
      item.key !== 'storageclass.beta.kubernetes.io/is-default-class'
  )

  return {
    name: (meta.name as string) || '',
    labels: objectToKeyValueItems(meta.labels as Record<string, unknown>),
    annotations: filteredAnnotations,
    isDefaultClass,
    provisioner: (scRecord?.provisioner as string) || '',
    reclaimPolicy: (scRecord?.reclaimPolicy as string) || 'Delete',
    volumeBindingMode: (scRecord?.volumeBindingMode as string) || 'Immediate',
    allowVolumeExpansion: Boolean(scRecord?.allowVolumeExpansion),
    mountOptions: Array.isArray(scRecord?.mountOptions) ? (scRecord.mountOptions as string[]) : [],
    parameters: objectToKeyValueItems(scRecord?.parameters as Record<string, unknown>)
  }
}

export function applyStorageClassFormToResource(
  initialResource: Record<string, unknown> | KubernetesResource,
  form: StorageClassFormValues
): Record<string, unknown> {
  const meta = { ...((initialResource?.metadata as Record<string, unknown>) || {}) }
  delete meta.namespace // Cluster-scoped

  meta.name = form.name.trim()

  const labels = keyValueItemsToObject(form.labels)
  if (Object.keys(labels).length > 0) {
    meta.labels = labels
  } else {
    delete meta.labels
  }

  const annotations = keyValueItemsToObject(form.annotations)
  if (form.isDefaultClass) {
    annotations['storageclass.kubernetes.io/is-default-class'] = 'true'
  } else {
    delete annotations['storageclass.kubernetes.io/is-default-class']
    delete annotations['storageclass.beta.kubernetes.io/is-default-class']
  }

  if (Object.keys(annotations).length > 0) {
    meta.annotations = annotations
  } else {
    delete meta.annotations
  }

  const updated: Record<string, unknown> = {
    ...initialResource,
    metadata: meta,
    allowVolumeExpansion: form.allowVolumeExpansion
  }

  const cleanedMountOptions = Array.isArray(form.mountOptions)
    ? form.mountOptions.map((m) => m.trim()).filter(Boolean)
    : []
  if (cleanedMountOptions.length > 0) {
    updated.mountOptions = cleanedMountOptions
  } else {
    delete updated.mountOptions
  }

  return updated
}

// ==========================================
// Creation Builders and Validators
// ==========================================

export interface CreatePVParams {
  name: string
  capacity: string
  accessModes: string[]
  reclaimPolicy: string
  volumeMode: string
  storageClassName?: string
  sourceType: 'hostPath' | 'nfs' | 'csi'
  hostPath?: string
  nfsServer?: string
  nfsPath?: string
  csiDriver?: string
  csiVolumeHandle?: string
  labels?: KeyValueItem[]
}

export function validateCreatePV(params: CreatePVParams): {
  valid: boolean
  errors: Record<string, string>
} {
  const errors: Record<string, string> = {}

  if (!params.name || !isValidK8sName(params.name)) {
    errors.name = 'Name must be a valid DNS-1123 subdomain (lowercase alphanumeric and -).'
  }

  if (!params.capacity || !isValidStorageQuantity(params.capacity)) {
    errors.capacity = 'Capacity must be a valid quantity (e.g. 10Gi, 500Mi).'
  }

  if (!params.accessModes || params.accessModes.length === 0) {
    errors.accessModes = 'At least one access mode must be selected.'
  }

  if (params.sourceType === 'hostPath') {
    if (!params.hostPath || !params.hostPath.trim()) {
      errors.hostPath = 'Host path is required for hostPath volume.'
    }
  } else if (params.sourceType === 'nfs') {
    if (!params.nfsServer || !params.nfsServer.trim()) {
      errors.nfsServer = 'NFS server is required.'
    }
    if (!params.nfsPath || !params.nfsPath.trim()) {
      errors.nfsPath = 'NFS export path is required.'
    }
  } else if (params.sourceType === 'csi') {
    if (!params.csiDriver || !params.csiDriver.trim()) {
      errors.csiDriver = 'CSI driver is required.'
    }
    if (!params.csiVolumeHandle || !params.csiVolumeHandle.trim()) {
      errors.csiVolumeHandle = 'CSI volume handle is required.'
    }
  }

  return {
    valid: Object.keys(errors).length === 0,
    errors
  }
}

export function buildCreatePVManifest(params: CreatePVParams): Record<string, unknown> {
  const meta: Record<string, unknown> = {
    name: params.name.trim()
  }

  const labels = keyValueItemsToObject(params.labels)
  if (Object.keys(labels).length > 0) {
    meta.labels = labels
  }

  const spec: Record<string, unknown> = {
    capacity: {
      storage: params.capacity.trim()
    },
    accessModes: params.accessModes,
    persistentVolumeReclaimPolicy: params.reclaimPolicy,
    volumeMode: params.volumeMode || 'Filesystem'
  }

  if (params.storageClassName?.trim()) {
    spec.storageClassName = params.storageClassName.trim()
  }

  if (params.sourceType === 'hostPath') {
    spec.hostPath = {
      path: params.hostPath?.trim() || ''
    }
  } else if (params.sourceType === 'nfs') {
    spec.nfs = {
      server: params.nfsServer?.trim() || '',
      path: params.nfsPath?.trim() || ''
    }
  } else if (params.sourceType === 'csi') {
    spec.csi = {
      driver: params.csiDriver?.trim() || '',
      volumeHandle: params.csiVolumeHandle?.trim() || ''
    }
  }

  return {
    apiVersion: 'v1',
    kind: 'PersistentVolume',
    metadata: meta,
    spec
  }
}

export interface CreatePVCParams {
  name: string
  namespace: string
  storageRequest: string
  accessModes: string[]
  volumeMode: string
  storageClassName?: string
  labels?: KeyValueItem[]
}

export function validateCreatePVC(params: CreatePVCParams): {
  valid: boolean
  errors: Record<string, string>
} {
  const errors: Record<string, string> = {}

  if (!params.name || !isValidK8sName(params.name)) {
    errors.name = 'Name must be a valid DNS-1123 subdomain.'
  }

  if (!params.namespace || !params.namespace.trim()) {
    errors.namespace = 'Namespace is required.'
  }

  if (!params.storageRequest || !isValidStorageQuantity(params.storageRequest)) {
    errors.storageRequest = 'Storage request must be a valid quantity (e.g. 10Gi, 500Mi).'
  }

  if (!params.accessModes || params.accessModes.length === 0) {
    errors.accessModes = 'At least one access mode must be selected.'
  }

  return {
    valid: Object.keys(errors).length === 0,
    errors
  }
}

export function buildCreatePVCManifest(params: CreatePVCParams): Record<string, unknown> {
  const meta: Record<string, unknown> = {
    name: params.name.trim(),
    namespace: params.namespace.trim()
  }

  const labels = keyValueItemsToObject(params.labels)
  if (Object.keys(labels).length > 0) {
    meta.labels = labels
  }

  const spec: Record<string, unknown> = {
    accessModes: params.accessModes,
    resources: {
      requests: {
        storage: params.storageRequest.trim()
      }
    },
    volumeMode: params.volumeMode || 'Filesystem'
  }

  if (params.storageClassName?.trim()) {
    spec.storageClassName = params.storageClassName.trim()
  }

  return {
    apiVersion: 'v1',
    kind: 'PersistentVolumeClaim',
    metadata: meta,
    spec
  }
}

export interface CreateStorageClassParams {
  name: string
  provisioner: string
  reclaimPolicy: string
  volumeBindingMode: string
  allowVolumeExpansion: boolean
  isDefaultClass: boolean
  parameters?: KeyValueItem[]
  mountOptions?: string[]
  labels?: KeyValueItem[]
}

export function validateCreateStorageClass(params: CreateStorageClassParams): {
  valid: boolean
  errors: Record<string, string>
} {
  const errors: Record<string, string> = {}

  if (!params.name || !isValidK8sName(params.name)) {
    errors.name = 'Name must be a valid DNS-1123 subdomain.'
  }

  if (!params.provisioner || !params.provisioner.trim()) {
    errors.provisioner = 'Provisioner is required (e.g. rancher.io/local-path).'
  }

  return {
    valid: Object.keys(errors).length === 0,
    errors
  }
}

export function buildCreateStorageClassManifest(
  params: CreateStorageClassParams
): Record<string, unknown> {
  const meta: Record<string, unknown> = {
    name: params.name.trim()
  }

  const annotations: Record<string, string> = {}
  if (params.isDefaultClass) {
    annotations['storageclass.kubernetes.io/is-default-class'] = 'true'
  }
  if (Object.keys(annotations).length > 0) {
    meta.annotations = annotations
  }

  const labels = keyValueItemsToObject(params.labels)
  if (Object.keys(labels).length > 0) {
    meta.labels = labels
  }

  const manifest: Record<string, unknown> = {
    apiVersion: 'storage.k8s.io/v1',
    kind: 'StorageClass',
    metadata: meta,
    provisioner: params.provisioner.trim(),
    reclaimPolicy: params.reclaimPolicy || 'Delete',
    volumeBindingMode: params.volumeBindingMode || 'Immediate',
    allowVolumeExpansion: params.allowVolumeExpansion
  }

  const parameters = keyValueItemsToObject(params.parameters)
  if (Object.keys(parameters).length > 0) {
    manifest.parameters = parameters
  }

  const cleanedMountOptions = params.mountOptions
    ? params.mountOptions.map((m) => m.trim()).filter(Boolean)
    : []
  if (cleanedMountOptions.length > 0) {
    manifest.mountOptions = cleanedMountOptions
  }

  return manifest
}

/**
 * Validates that a PVC storage expansion does not reduce the requested capacity.
 * In Kubernetes, PVC storage capacity can only be expanded, never shrunk.
 */
export function validatePVCStorageExpansion(
  initialQuantity: string | null | undefined,
  newQuantity: string | null | undefined
): boolean {
  const initialBytes = parseStorageBytes(initialQuantity)
  const newBytes = parseStorageBytes(newQuantity)
  if (newBytes <= 0) return false
  return newBytes >= initialBytes
}
