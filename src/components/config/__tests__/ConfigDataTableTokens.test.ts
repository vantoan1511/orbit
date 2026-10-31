import assert from 'node:assert/strict'
import test from 'node:test'
import fs from 'node:fs'
import path from 'node:path'

const TABLE_PATH = path.resolve('src/components/config/ConfigDataTable.vue')

test('ConfigDataTable.vue uses design tokens for mounted pods instead of hardcoded emerald color', () => {
  const content = fs.readFileSync(TABLE_PATH, 'utf-8')

  // Must NOT contain hardcoded emerald-400
  assert.ok(
    !content.includes('text-emerald-400'),
    'ConfigDataTable.vue must not contain hardcoded text-emerald-400'
  )

  // Must use semantic text-(--success) token for positive mounted pods
  assert.ok(
    content.includes("data.mountedPods > 0 ? 'text-(--success)' : 'text-muted-color'"),
    'ConfigDataTable.vue must use text-(--success) for positive mountedPods'
  )
})
