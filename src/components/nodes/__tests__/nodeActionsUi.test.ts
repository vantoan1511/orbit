import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'

test('NodesTable.vue enables actions column, disables selection, and renders cordoned status tag with v-tooltip', () => {
  const filePath = path.resolve('src/components/nodes/NodesTable.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // Check actions enabled and selection disabled
  assert.ok(
    content.includes(':hideActionsColumn="false"'),
    'NodesTable.vue should set :hideActionsColumn="false"'
  )
  assert.ok(
    content.includes(':selectable="false"'),
    'NodesTable.vue should set :selectable="false"'
  )

  // Check status slot with Cordoned warning tag
  assert.ok(
    content.includes('<template #status="{ data }">') ||
      content.includes('<template #status="{ data }">'),
    'NodesTable.vue should define a #status slot'
  )
  assert.ok(
    content.includes('data.isCordoned'),
    'NodesTable.vue should check data.isCordoned in status slot'
  )
  assert.ok(content.includes('value="Cordoned"'), 'NodesTable.vue should render Cordoned tag')

  // Verify PrimeVue tooltip directive usage and absence of native :title on label tags
  assert.ok(
    content.includes('v-tooltip.top'),
    'NodesTable.vue should use v-tooltip.top directive for labels'
  )
  assert.ok(
    !content.includes(':title="label"'),
    'NodesTable.vue should not use native :title="label"'
  )
})

test('NodeDetailsDrawer.vue binds liveNode reactivity and provides Cordon/Uncordon, Drain, and Edit actions', () => {
  const filePath = path.resolve('src/components/nodes/NodeDetailsDrawer.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // Live node reactivity from store
  assert.ok(content.includes('liveNode'), 'NodeDetailsDrawer.vue should define liveNode')
  assert.ok(
    content.includes('k8sStore.nodes.find'),
    'NodeDetailsDrawer.vue should resolve liveNode from k8sStore.nodes'
  )

  // Drawer tags slot
  assert.ok(content.includes('<template #tags>'), 'NodeDetailsDrawer.vue should define #tags slot')
  assert.ok(
    content.includes('isCordoned'),
    'NodeDetailsDrawer.vue should display cordoned tag when node is cordoned'
  )

  // Drawer actions slot
  assert.ok(
    content.includes('<template #actions>'),
    'NodeDetailsDrawer.vue should define #actions slot'
  )
  assert.ok(
    content.includes('cordonNode') || content.includes('handleCordon'),
    'NodeDetailsDrawer.vue should support cordon action'
  )
  assert.ok(
    content.includes('uncordonNode') || content.includes('handleUncordon'),
    'NodeDetailsDrawer.vue should support uncordon action'
  )
  assert.ok(
    content.includes('drainNode') || content.includes('handleDrain'),
    'NodeDetailsDrawer.vue should support drain action'
  )
  assert.ok(
    content.includes('handleEdit') || content.includes('edit-workload'),
    'NodeDetailsDrawer.vue should support edit action'
  )
})
