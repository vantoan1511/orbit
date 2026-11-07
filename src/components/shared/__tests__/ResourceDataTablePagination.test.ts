import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('ResourceDataTable hoists passthrough config and configures Paginator without virtual scroll', () => {
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

  // Verifies pagination imports & bindings
  assert.match(content, /import\s+.*Paginator.*from\s+['"]primevue\/paginator['"]/)
  assert.match(content, /from\s+['"]@\/utils\/tablePagination['"]/)
  assert.match(content, /sortTableRowsStable\(/, 'ResourceDataTable must use sortTableRowsStable')
  assert.match(content, /:scrollable=/)
  assert.match(content, /<Paginator/)

  // Verifies virtual scroll is completely removed
  assert.doesNotMatch(
    content,
    /virtualScrollerOptions/,
    'ResourceDataTable must not bind virtualScrollerOptions'
  )
  assert.doesNotMatch(
    content,
    /isVirtualScrollEnabled/,
    'ResourceDataTable must not declare isVirtualScrollEnabled'
  )
  assert.doesNotMatch(
    content,
    /DEFAULT_VIRTUAL_SCROLLER_OPTIONS/,
    'ResourceDataTable must not import DEFAULT_VIRTUAL_SCROLLER_OPTIONS'
  )

  // Verifies table container has data-component attribute and full-height flex layout
  assert.match(content, /data-component="resource-data-table"/)
  assert.match(content, /class="[^"]*flex-1 min-h-0[^"]*"/)
  assert.match(content, /scrollHeight="flex"/)
  assert.match(content, /:scrollable="true"/)
  assert.match(content, /:pt="DATATABLE_PT"/)
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

test('ResourceDataTable and GenericResourceTable wire resourceKind to useVisibleResourceKind', () => {
  const rdtPath = path.resolve('src/components/shared/ResourceDataTable.vue')
  const rdtContent = fs.readFileSync(rdtPath, 'utf-8')

  // ResourceDataTable must import useVisibleResourceKind and declare resourceKind prop
  assert.match(
    rdtContent,
    /import\s+.*useVisibleResourceKind.*from\s+['"]@\/composables\/useVisibleResourceKind['"]/,
    'ResourceDataTable must import useVisibleResourceKind'
  )
  assert.match(
    rdtContent,
    /resourceKind\?:\s*string/,
    'ResourceDataTable must declare optional resourceKind?: string prop'
  )
  assert.match(
    rdtContent,
    /useVisibleResourceKind\(\s*\(\)\s*=>\s*props\.resourceKind\s*\)/,
    'ResourceDataTable must call useVisibleResourceKind with getter for props.resourceKind'
  )

  // GenericResourceTable passes kind to ResourceDataTable
  const grtPath = path.resolve('src/components/shared/GenericResourceTable.vue')
  const grtContent = fs.readFileSync(grtPath, 'utf-8')
  assert.match(
    grtContent,
    /:(resourceKind|resource-kind)="kind"/,
    'GenericResourceTable must bind :resourceKind="kind" to ResourceDataTable'
  )

  // PodsDataTable binds Pod kind to GenericResourceTable
  const podsPath = path.resolve('src/components/pods/PodsDataTable.vue')
  const podsContent = fs.readFileSync(podsPath, 'utf-8')
  assert.match(
    podsContent,
    /:(kind|resourceKind|resource-kind)="KUBERNETES_RESOURCE_KIND\.Pod"/,
    'PodsDataTable must bind Pod kind'
  )
})

test('ResourceDataTable wires useTableScrollGuard, binds data-scrolling and passive capture scroll listener, and omits border-collapse', () => {
  const filePath = path.resolve('src/components/shared/ResourceDataTable.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.match(
    content,
    /useTableScrollGuard/,
    'ResourceDataTable must import and use useTableScrollGuard'
  )
  assert.match(
    content,
    /@scroll\.capture\.passive="onScroll"/,
    'ResourceDataTable must attach passive capture scroll listener'
  )
  assert.match(content, /:data-scrolling=/, 'ResourceDataTable must bind data-scrolling attribute')
  assert.doesNotMatch(
    content,
    /border-collapse/,
    'ResourceDataTable must omit border-collapse from tableClass'
  )
})
