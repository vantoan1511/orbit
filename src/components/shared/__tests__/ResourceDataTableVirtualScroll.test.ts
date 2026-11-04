import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('ResourceDataTable hoists passthrough config and configures virtualScrollerOptions with Paginator', () => {
  const filePath = path.resolve('src/components/shared/ResourceDataTable.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // Verifies hoisted passthrough constant
  assert.match(
    content,
    /const (PAGINATOR_PT|DATATABLE_PT)\b/,
    'ResourceDataTable must declare hoisted passthrough config'
  )
  assert.doesNotMatch(
    content,
    /:pt="\{\s*pcPaginator:/,
    'ResourceDataTable must not declare inline pcPaginator :pt object'
  )

  // Verifies virtual pagination and scrolling imports & bindings
  assert.match(content, /import\s+.*Paginator.*from\s+['"]primevue\/paginator['"]/)
  assert.match(content, /from\s+['"]@\/utils\/tableVirtualPagination['"]/)
  assert.match(content, /:scrollable=/)
  assert.match(content, /:virtualScrollerOptions=/)
  assert.match(content, /<Paginator/)

  // Verifies table container has data-component attribute
  assert.match(content, /data-component="resource-data-table"/)
})

test('GenericResourceTable uses Set for selectedRowKeys lookup and omits redundant v-tooltip on Actions button', () => {
  const filePath = path.resolve('src/components/shared/GenericResourceTable.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // Verifies Set membership for row selection lookups
  assert.match(
    content,
    /(new Set\([^)]*\)\.has\(|const \w+ = new Set\([^)]*\)[\s\S]*?\b\w+\.has\()/,
    'GenericResourceTable must use Set.has for selectedRowKeys matching'
  )
  assert.doesNotMatch(
    content,
    /storedRowKeys\.includes\(/,
    'GenericResourceTable must not use storedRowKeys.includes'
  )

  // Verifies Actions button omits redundant v-tooltip
  const actionsColMatch = content.match(/<!-- Actions Column -->[\s\S]*?<\/Column>/)
  assert.ok(actionsColMatch, 'Actions Column must exist')
  const actionsCol = actionsColMatch[0]
  assert.doesNotMatch(actionsCol, /v-tooltip/, 'Actions button must omit redundant v-tooltip')
  assert.ok(
    actionsCol.includes('aria-label="Actions"'),
    'Actions button must preserve aria-label="Actions"'
  )
})
