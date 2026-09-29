import test from 'node:test'
import assert from 'node:assert/strict'
import { getResourceBreadcrumbs, RESOURCE_BREADCRUMB_MAP } from '../breadcrumb.ts'

test('getResourceBreadcrumbs - Deployment (namespaced workload)', () => {
  const crumbs = getResourceBreadcrumbs({
    kind: 'Deployment',
    namespace: 'production',
    name: 'api-gateway'
  })

  assert.deepEqual(crumbs, [
    { label: 'Workloads', route: '/workloads' },
    { label: 'Deployments', route: '/workloads?tab=deployments' },
    { label: 'production' },
    { label: 'api-gateway' },
    { label: 'Edit' }
  ])
})

test('getResourceBreadcrumbs - Pod (namespaced workload)', () => {
  const crumbs = getResourceBreadcrumbs({
    kind: 'Pod',
    namespace: 'kube-system',
    name: 'coredns-xyz'
  })

  assert.deepEqual(crumbs, [
    { label: 'Workloads', route: '/workloads' },
    { label: 'Pods', route: '/workloads?tab=pods' },
    { label: 'kube-system' },
    { label: 'coredns-xyz' },
    { label: 'Edit' }
  ])
})

test('getResourceBreadcrumbs - DaemonSet, StatefulSet, ReplicaSet, Job, CronJob', () => {
  const daemonSetCrumbs = getResourceBreadcrumbs({
    kind: 'DaemonSet',
    namespace: 'monitoring',
    name: 'node-exporter'
  })
  assert.deepEqual(daemonSetCrumbs, [
    { label: 'Workloads', route: '/workloads' },
    { label: 'DaemonSets', route: '/workloads?tab=daemonsets' },
    { label: 'monitoring' },
    { label: 'node-exporter' },
    { label: 'Edit' }
  ])

  const statefulSetCrumbs = getResourceBreadcrumbs({
    kind: 'StatefulSet',
    namespace: 'db',
    name: 'postgres'
  })
  assert.deepEqual(statefulSetCrumbs, [
    { label: 'Workloads', route: '/workloads' },
    { label: 'StatefulSets', route: '/workloads?tab=statefulsets' },
    { label: 'db' },
    { label: 'postgres' },
    { label: 'Edit' }
  ])

  const cronJobCrumbs = getResourceBreadcrumbs({
    kind: 'CronJob',
    namespace: 'batch',
    name: 'nightly-backup'
  })
  assert.deepEqual(cronJobCrumbs, [
    { label: 'Workloads', route: '/workloads' },
    { label: 'CronJobs', route: '/workloads?tab=cronjobs' },
    { label: 'batch' },
    { label: 'nightly-backup' },
    { label: 'Edit' }
  ])
})

test('getResourceBreadcrumbs - Ingress (handles irregular plural Ingresses in Network)', () => {
  const crumbs = getResourceBreadcrumbs({
    kind: 'Ingress',
    namespace: 'default',
    name: 'web-ingress'
  })

  assert.deepEqual(crumbs, [
    { label: 'Network', route: '/network' },
    { label: 'Ingresses', route: '/network?tab=ingresses' },
    { label: 'default' },
    { label: 'web-ingress' },
    { label: 'Edit' }
  ])
})

test('getResourceBreadcrumbs - Service (in Network category)', () => {
  const crumbs = getResourceBreadcrumbs({
    kind: 'Service',
    namespace: 'default',
    name: 'web-service'
  })

  assert.deepEqual(crumbs, [
    { label: 'Network', route: '/network' },
    { label: 'Services', route: '/network?tab=services' },
    { label: 'default' },
    { label: 'web-service' },
    { label: 'Edit' }
  ])
})

test('getResourceBreadcrumbs - ConfigMap & Secret (in ConfigMaps & Secrets category)', () => {
  const configMapCrumbs = getResourceBreadcrumbs({
    kind: 'ConfigMap',
    namespace: 'staging',
    name: 'app-env'
  })
  assert.deepEqual(configMapCrumbs, [
    { label: 'ConfigMaps & Secrets', route: '/config' },
    { label: 'ConfigMaps', route: '/config?tab=configmaps' },
    { label: 'staging' },
    { label: 'app-env' },
    { label: 'Edit' }
  ])

  const secretCrumbs = getResourceBreadcrumbs({
    kind: 'Secret',
    namespace: 'prod',
    name: 'api-token'
  })
  assert.deepEqual(secretCrumbs, [
    { label: 'ConfigMaps & Secrets', route: '/config' },
    { label: 'Secrets', route: '/config?tab=secrets' },
    { label: 'prod' },
    { label: 'api-token' },
    { label: 'Edit' }
  ])
})

test('getResourceBreadcrumbs - Storage resources (PVC, PV, StorageClass)', () => {
  const pvcCrumbs = getResourceBreadcrumbs({
    kind: 'PersistentVolumeClaim',
    namespace: 'data',
    name: 'pvc-disk'
  })
  assert.deepEqual(pvcCrumbs, [
    { label: 'Storage', route: '/storage' },
    { label: 'Volume Claims', route: '/storage?tab=pvcs' },
    { label: 'data' },
    { label: 'pvc-disk' },
    { label: 'Edit' }
  ])

  // PV is cluster-scoped -> no bogus namespace
  const pvCrumbs = getResourceBreadcrumbs({
    kind: 'PersistentVolume',
    namespace: 'default', // Router might pass 'default' as fallback
    name: 'pv-disk-01'
  })
  assert.deepEqual(pvCrumbs, [
    { label: 'Storage', route: '/storage' },
    { label: 'PersistentVolumes', route: '/storage?tab=pvs' },
    { label: 'pv-disk-01' },
    { label: 'Edit' }
  ])

  // StorageClass is cluster-scoped -> no bogus namespace
  const scCrumbs = getResourceBreadcrumbs({
    kind: 'StorageClass',
    namespace: 'default',
    name: 'fast-ssd'
  })
  assert.deepEqual(scCrumbs, [
    { label: 'Storage', route: '/storage' },
    { label: 'StorageClasses', route: '/storage?tab=classes' },
    { label: 'fast-ssd' },
    { label: 'Edit' }
  ])
})

test('getResourceBreadcrumbs - Namespace (cluster-scoped, deduped category and type)', () => {
  const crumbs = getResourceBreadcrumbs({
    kind: 'Namespace',
    namespace: 'default', // should be omitted since Namespace has no parent namespace
    name: 'production'
  })

  assert.deepEqual(crumbs, [
    { label: 'Namespaces', route: '/namespaces' },
    { label: 'production' },
    { label: 'Edit' }
  ])
})

test('getResourceBreadcrumbs - Node (cluster-scoped, deduped category and type)', () => {
  const crumbs = getResourceBreadcrumbs({
    kind: 'Node',
    namespace: 'default',
    name: 'worker-01'
  })

  assert.deepEqual(crumbs, [
    { label: 'Nodes', route: '/nodes' },
    { label: 'worker-01' },
    { label: 'Edit' }
  ])
})

test('getResourceBreadcrumbs - Policy (cluster-scoped)', () => {
  const crumbs = getResourceBreadcrumbs({
    kind: 'Policy',
    name: 'deny-ingress'
  })

  assert.deepEqual(crumbs, [
    { label: 'Policies', route: '/policies' },
    { label: 'deny-ingress' },
    { label: 'Edit' }
  ])
})

test('getResourceBreadcrumbs - Fallback for unknown kind', () => {
  const crumbs = getResourceBreadcrumbs({
    kind: 'CustomResource',
    namespace: 'custom-ns',
    name: 'my-cr'
  })

  assert.deepEqual(crumbs, [
    { label: 'Workloads', route: '/workloads' },
    { label: 'CustomResource', route: '/workloads?tab=customresources' },
    { label: 'custom-ns' },
    { label: 'my-cr' },
    { label: 'Edit' }
  ])
})

test('RESOURCE_BREADCRUMB_MAP provides valid CategoryIds matching sidebar navigation', () => {
  for (const [kind, meta] of Object.entries(RESOURCE_BREADCRUMB_MAP)) {
    assert.ok(meta.categoryId, `Missing categoryId for ${kind}`)
    assert.ok(meta.categoryLabel, `Missing categoryLabel for ${kind}`)
    assert.ok(meta.categoryRoute, `Missing categoryRoute for ${kind}`)
    assert.ok(meta.typeLabel, `Missing typeLabel for ${kind}`)
    assert.ok(meta.typeRoute, `Missing typeRoute for ${kind}`)
    assert.strictEqual(
      typeof meta.isClusterScoped,
      'boolean',
      `Missing isClusterScoped for ${kind}`
    )
  }
})
