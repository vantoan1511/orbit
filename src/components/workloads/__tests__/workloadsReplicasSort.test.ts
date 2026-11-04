import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('DeploymentsTable configures sortable replicas column with replicasSortKey and aria-label', () => {
  const filePath = path.resolve('src/components/workloads/DeploymentsTable.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // 1. Imports createDeploymentRowDecorator and does NOT import computeReplicasSortKey
  assert.ok(
    content.includes('createDeploymentRowDecorator'),
    'DeploymentsTable must import createDeploymentRowDecorator from @/utils/workloadMetrics'
  )
  assert.ok(
    !content.includes('computeReplicasSortKey'),
    'DeploymentsTable must NOT import unused computeReplicasSortKey'
  )

  // 2. Uses shallowRef and watch observing restartFingerprint for deploymentRestartsMap
  assert.ok(
    /const\s+deploymentRestartsMap\s*=\s*shallowRef\(new\s+Map<string,\s*number>\(\)\)/.test(
      content
    ),
    'DeploymentsTable must declare deploymentRestartsMap using shallowRef'
  )
  assert.ok(
    /watch\(\s*\(\)\s*=>\s*restartFingerprint\(k8sStore\.pods\)/.test(content),
    'DeploymentsTable must watch restartFingerprint(k8sStore.pods)'
  )

  // 3. Computes deployments using deploymentDecorator
  assert.ok(
    /deploymentDecorator\(\s*k8sStore\.deployments,\s*deploymentRestartsMap\.value\s*\)/.test(
      content
    ),
    'DeploymentsTable must compute deployments with deploymentDecorator'
  )

  // 4. Configures sortable Column with field="replicas" and sortField="replicasSortKey"
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

  // 5. Preserves inline replica scaling click handler
  assert.ok(
    content.includes('@click.stop="openReplicasEdit($event, data)"'),
    'Must preserve openReplicasEdit click handler'
  )

  // 6. Tooltip directive hygiene: zero static title="..." or :title="..." in template
  const templateMatch = content.match(/<template\b[^>]*>([\s\S]*?)<\/template>/)
  assert.ok(templateMatch, 'Must contain template block')
  const templateContent = templateMatch[1]
  const titleMatches = [...templateContent.matchAll(/\b:?title\s*=\s*(['"][^'"]*?['"])/g)]
  assert.equal(
    titleMatches.length,
    0,
    `DeploymentsTable template must not contain title / :title attributes; found: ${titleMatches.map((m) => m[0]).join(', ')}`
  )

  // 7. Uses aria-label for edit triggers
  assert.ok(
    content.includes('aria-label="Click to edit replicas"'),
    'Replicas edit trigger must use aria-label="Click to edit replicas"'
  )
  assert.ok(
    content.includes('aria-label="Click to edit container images"'),
    'Images edit trigger must use aria-label="Click to edit container images"'
  )

  // 8. Renders precomputed orderedConditions and imageTags
  assert.ok(
    content.includes('data.orderedConditions'),
    'DeploymentsTable must render precomputed data.orderedConditions'
  )
  assert.ok(
    content.includes('data.imageTags'),
    'DeploymentsTable must render precomputed data.imageTags'
  )
  assert.ok(
    content.includes('v-tooltip.top="img.full"'),
    'DeploymentsTable must use v-tooltip.top="img.full" for image tags'
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

test('ReplicaSetsTable configures sortable replicas column with withReplicasSortKey and precomputed imageTags', () => {
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

  // 4. Renders precomputed data.imageTags and avoids native title / :title
  assert.ok(
    content.includes('data.imageTags'),
    'ReplicaSetsTable must render precomputed data.imageTags'
  )
  assert.ok(
    content.includes('v-tooltip.top="img.full"'),
    'ReplicaSetsTable must use v-tooltip.top="img.full"'
  )

  const templateMatch = content.match(/<template\b[^>]*>([\s\S]*?)<\/template>/)
  assert.ok(templateMatch, 'Must contain template block')
  const templateContent = templateMatch[1]
  const titleMatches = [...templateContent.matchAll(/\b:?title\s*=\s*(['"][^'"]*?['"])/g)]
  assert.equal(
    titleMatches.length,
    0,
    `ReplicaSetsTable template must not contain title / :title attributes; found: ${titleMatches.map((m) => m[0]).join(', ')}`
  )
})

test('PodsDataTable uses createPodRowDecorator and avoids native :title attributes', () => {
  const filePath = path.resolve('src/components/pods/PodsDataTable.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('createPodRowDecorator'),
    'PodsDataTable must import and use createPodRowDecorator'
  )
  assert.ok(
    content.includes('data.nodeShort'),
    'PodsDataTable must render precomputed data.nodeShort'
  )
  assert.ok(
    content.includes('data.cpuWidthStyle'),
    'PodsDataTable must bind precomputed data.cpuWidthStyle'
  )
  assert.ok(
    content.includes('data.memoryWidthStyle'),
    'PodsDataTable must bind precomputed data.memoryWidthStyle'
  )
  assert.ok(
    content.includes('v-tooltip.top="data.node"'),
    'PodsDataTable must use v-tooltip.top="data.node" for node column'
  )

  const templateMatch = content.match(/<template\b[^>]*>([\s\S]*?)<\/template>/)
  assert.ok(templateMatch, 'Must contain template block')
  const templateContent = templateMatch[1]
  const titleMatches = [...templateContent.matchAll(/\b:?title\s*=\s*(['"][^'"]*?['"])/g)]
  assert.equal(
    titleMatches.length,
    0,
    `PodsDataTable template must not contain title / :title attributes; found: ${titleMatches.map((m) => m[0]).join(', ')}`
  )
})
