import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('DeploymentsTable configures sortable replicas column with replicasSortKey and v-tooltip', () => {
  const filePath = path.resolve('src/components/workloads/DeploymentsTable.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // 1. Imports computeReplicasSortKey
  assert.ok(
    content.includes('computeReplicasSortKey'),
    'DeploymentsTable must import computeReplicasSortKey from @/utils/workloadMetrics'
  )

  // 2. Computes replicasSortKey in deployments computed
  assert.ok(
    /replicasSortKey:\s*computeReplicasSortKey\(\s*d\.replicas\s*\)/.test(content),
    'deployments computed list must attach replicasSortKey'
  )

  // 3. Configures sortable Column with field="replicas" and sortField="replicasSortKey"
  assert.ok(
    content.includes('sortField="replicasSortKey"'),
    'Replicas Column must configure sortField="replicasSortKey"'
  )
  assert.ok(
    /<Column\b[^>]*v-if="visibleCols\['replicas'\]"[^>]*field="replicas"[^>]*sortField="replicasSortKey"[^>]*sortable/s.test(
      content
    ) ||
      /<Column\b[^>]*field="replicas"[^>]*sortField="replicasSortKey"[^>]*header="Replicas"[^>]*sortable/s.test(
        content
      ),
    'Replicas Column must be sortable with field="replicas" and sortField="replicasSortKey"'
  )

  // 4. Preserves inline replica scaling click handler
  assert.ok(
    content.includes('@click.stop="openReplicasEdit($event, data)"'),
    'Must preserve openReplicasEdit click handler'
  )

  // 5. Tooltip directive hygiene: zero static title="..." in template
  const templateMatch = content.match(/<template\b[^>]*>([\s\S]*?)<\/template>/)
  assert.ok(templateMatch, 'Must contain template block')
  const templateContent = templateMatch[1]
  const staticTitleMatches = [...templateContent.matchAll(/(?<!:)\btitle\s*=\s*(['"][^'"]*?['"])/g)]
  assert.equal(
    staticTitleMatches.length,
    0,
    `DeploymentsTable template must not contain static title="..." attributes; found: ${staticTitleMatches.map((m) => m[0]).join(', ')}`
  )

  // 6. Uses v-tooltip.top for edit triggers
  assert.ok(
    content.includes('v-tooltip.top="\'Click to edit replicas\'"'),
    'Replicas edit trigger must use v-tooltip.top'
  )
  assert.ok(
    content.includes('v-tooltip.top="\'Click to edit container images\'"'),
    'Images edit trigger must use v-tooltip.top'
  )
})

test('StatefulSetsTable configures sortable replicas column with withReplicasSortKey', () => {
  const filePath = path.resolve('src/components/workloads/StatefulSetsTable.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // 1. Imports withReplicasSortKey
  assert.ok(
    content.includes('withReplicasSortKey'),
    'StatefulSetsTable must import withReplicasSortKey from @/utils/workloadMetrics'
  )

  // 2. Computes statefulSets with replicasSortKey
  assert.ok(
    /const\s+statefulSets\s*=\s*computed\(\(\)\s*=>\s*withReplicasSortKey\(\s*k8sStore\.statefulSets\s*\)\)/.test(
      content
    ),
    'statefulSets computed list must call withReplicasSortKey(k8sStore.statefulSets)'
  )

  // 3. Passes computed statefulSets to GenericResourceTable
  assert.ok(
    content.includes(':data="statefulSets"'),
    'GenericResourceTable must bind :data="statefulSets"'
  )

  // 4. Configures sortable Column with field="replicas" and sortField="replicasSortKey"
  assert.ok(
    content.includes('sortField="replicasSortKey"'),
    'Replicas Column must configure sortField="replicasSortKey"'
  )
  assert.ok(
    /<Column\b[^>]*field="replicas"[^>]*sortField="replicasSortKey"[^>]*header="Replicas"[^>]*sortable/s.test(
      content
    ) ||
      /<Column\b[^>]*v-if="visibleCols\['replicas'\]"[^>]*field="replicas"[^>]*sortField="replicasSortKey"[^>]*header="Replicas"[^>]*sortable/s.test(
        content
      ),
    'Replicas Column in StatefulSetsTable must be sortable with field="replicas" and sortField="replicasSortKey"'
  )
})

test('ReplicaSetsTable configures sortable replicas column with withReplicasSortKey', () => {
  const filePath = path.resolve('src/components/workloads/ReplicaSetsTable.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // 1. Imports withReplicasSortKey
  assert.ok(
    content.includes('withReplicasSortKey'),
    'ReplicaSetsTable must import withReplicasSortKey from @/utils/workloadMetrics'
  )

  // 2. Computes replicaSets with replicasSortKey
  assert.ok(
    /const\s+replicaSets\s*=\s*computed\(\(\)\s*=>\s*withReplicasSortKey\(\s*k8sStore\.replicaSets\s*\)\)/.test(
      content
    ),
    'replicaSets computed list must call withReplicasSortKey(k8sStore.replicaSets)'
  )

  // 3. Passes computed replicaSets to GenericResourceTable
  assert.ok(
    content.includes(':data="replicaSets"'),
    'GenericResourceTable must bind :data="replicaSets"'
  )

  // 4. Configures sortable Column with field="replicas" and sortField="replicasSortKey"
  assert.ok(
    content.includes('sortField="replicasSortKey"'),
    'Replicas Column must configure sortField="replicasSortKey"'
  )
  assert.ok(
    /<Column\b[^>]*field="replicas"[^>]*sortField="replicasSortKey"[^>]*header="Replicas"[^>]*sortable/s.test(
      content
    ) ||
      /<Column\b[^>]*v-if="visibleCols\['replicas'\]"[^>]*field="replicas"[^>]*sortField="replicasSortKey"[^>]*header="Replicas"[^>]*sortable/s.test(
        content
      ),
    'Replicas Column in ReplicaSetsTable must be sortable with field="replicas" and sortField="replicasSortKey"'
  )
})
