import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'

test('NamespacesDataTable.vue removes fake pod sparkline and renders clean numeric count', () => {
  const filePath = path.resolve('src/components/namespaces/NamespacesDataTable.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('data-component="namespaces-data-table"'),
    'NamespacesDataTable.vue should preserve data-component attribute'
  )
  assert.ok(
    !content.includes('podSparkline'),
    'NamespacesDataTable.vue should not contain podSparkline'
  )
  assert.ok(
    !content.includes('getSparklineData'),
    'NamespacesDataTable.vue should not contain getSparklineData'
  )
  assert.ok(
    !content.includes("import Chart from 'primevue/chart'"),
    'NamespacesDataTable.vue should not import Chart for sparklines'
  )
  assert.ok(
    content.includes('{{ data.pods }}'),
    'NamespacesDataTable.vue should display clean numeric count for data.pods'
  )
})
