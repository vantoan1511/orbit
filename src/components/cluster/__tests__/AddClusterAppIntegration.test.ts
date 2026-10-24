import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('App.vue imports and mounts AddClusterModal', () => {
  const appPath = path.resolve(process.cwd(), 'src/App.vue')
  const content = fs.readFileSync(appPath, 'utf-8')

  assert.ok(content.includes('AddClusterModal'), 'App.vue must import AddClusterModal')
  assert.ok(
    content.includes('isAddClusterModalVisible'),
    'App.vue must bind isAddClusterModalVisible'
  )
  assert.ok(content.includes('<AddClusterModal'), 'App.vue template must render <AddClusterModal')
})
