import assert from 'node:assert/strict'
import test from 'node:test'
import fs from 'node:fs'
import path from 'node:path'

const DRAWER_PATH = path.resolve('src/components/config/ConfigDetailsDrawer.vue')

test('ConfigDetailsDrawer.vue renders mountType badges and handles interactive pod navigation', () => {
  const content = fs.readFileSync(DRAWER_PATH, 'utf-8')

  // Must import router and tableFilterStore
  assert.ok(content.includes('useRouter'), 'ConfigDetailsDrawer.vue must import useRouter')
  assert.ok(
    content.includes('@/stores/tableFilterStore'),
    'ConfigDetailsDrawer.vue must import useTableFilterStore'
  )

  // Must define handleNavigateToPod
  assert.ok(
    content.includes('handleNavigateToPod'),
    'ConfigDetailsDrawer.vue must define handleNavigateToPod'
  )

  // Must set filters on tableFilterStore for pod search & namespace
  assert.ok(
    content.includes("tableFilterStore.setFilter('pod', 'searchQuery'"),
    "ConfigDetailsDrawer.vue must set 'pod' 'searchQuery' filter"
  )
  assert.ok(
    content.includes("tableFilterStore.setFilter('pod', 'selectedNamespace'"),
    "ConfigDetailsDrawer.vue must set 'pod' 'selectedNamespace' filter"
  )

  // Must render mountType badges
  assert.ok(
    content.includes('pod.mountType'),
    'ConfigDetailsDrawer.vue must reference pod.mountType for badges'
  )

  // Must NOT have hardcoded violet colors
  assert.ok(
    !content.includes('text-violet-400'),
    'ConfigDetailsDrawer.vue must not contain hardcoded text-violet-400'
  )

  // Must NOT have native title attributes on HTML elements
  assert.ok(
    !/<(button|div|span|input|a)\b[^>]*\btitle=/i.test(content),
    'ConfigDetailsDrawer.vue must not contain native HTML title attributes on elements'
  )
})
