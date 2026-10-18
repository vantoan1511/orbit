import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('Domain data tables contain unique data-component attributes', () => {
  const tableFiles = [
    { file: 'src/components/pods/PodsDataTable.vue', tag: 'pods-data-table' },
    { file: 'src/components/nodes/NodesTable.vue', tag: 'nodes-table' },
    { file: 'src/components/namespaces/NamespacesDataTable.vue', tag: 'namespaces-data-table' },
    { file: 'src/components/workloads/DeploymentsTable.vue', tag: 'deployments-table' },
    { file: 'src/components/workloads/DaemonSetsTable.vue', tag: 'daemonsets-table' },
    { file: 'src/components/workloads/StatefulSetsTable.vue', tag: 'statefulsets-table' },
    { file: 'src/components/workloads/ReplicaSetsTable.vue', tag: 'replicasets-table' },
    { file: 'src/components/workloads/JobsTable.vue', tag: 'jobs-table' },
    { file: 'src/components/workloads/CronJobsTable.vue', tag: 'cronjobs-table' },
    { file: 'src/components/network/ServicesTable.vue', tag: 'services-table' },
    { file: 'src/components/network/IngressesTable.vue', tag: 'ingresses-table' },
    { file: 'src/components/network/PortForwardsTable.vue', tag: 'port-forwards-table' },
    { file: 'src/components/storage/StorageClassesTable.vue', tag: 'storage-classes-table' },
    { file: 'src/components/storage/StoragePVTable.vue', tag: 'storage-pv-table' },
    { file: 'src/components/storage/StoragePVCTable.vue', tag: 'storage-pvc-table' },
    { file: 'src/components/policies/PoliciesDataTable.vue', tag: 'policies-data-table' }
  ]

  for (const { file, tag } of tableFiles) {
    const filePath = path.resolve(file)
    const content = fs.readFileSync(filePath, 'utf-8')
    assert.ok(
      content.includes(`data-component="${tag}"`),
      `${file} must contain data-component="${tag}"`
    )
  }
})

test('PodsDataTable and NodesTable avoid hardcoded progress bar colors', () => {
  const podsPath = path.resolve('src/components/pods/PodsDataTable.vue')
  const podsContent = fs.readFileSync(podsPath, 'utf-8')
  assert.ok(
    !podsContent.includes('bg-violet-500'),
    'PodsDataTable must not contain hardcoded bg-violet-500'
  )

  const nodesPath = path.resolve('src/components/nodes/NodesTable.vue')
  const nodesContent = fs.readFileSync(nodesPath, 'utf-8')
  assert.ok(
    !nodesContent.includes('bg-blue-500'),
    'NodesTable must not contain hardcoded bg-blue-500'
  )
})
