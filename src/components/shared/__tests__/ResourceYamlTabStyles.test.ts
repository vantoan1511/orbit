import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('ResourceYamlTab adapts to theme tokens and drops hardcoded zinc colors', () => {
  const yamlPath = path.resolve('src/components/shared/ResourceYamlTab.vue')
  const content = fs.readFileSync(yamlPath, 'utf-8')

  assert.ok(
    content.includes('data-component="resource-yaml-tab"'),
    'ResourceYamlTab must have data-component="resource-yaml-tab"'
  )
  assert.ok(
    !content.includes('bg-zinc-950'),
    'ResourceYamlTab must not contain hardcoded bg-zinc-950'
  )
  assert.ok(
    !content.includes('border-zinc-800'),
    'ResourceYamlTab must not contain hardcoded border-zinc-800'
  )
  assert.ok(content.includes('bg-(--bg-card)'), 'ResourceYamlTab must use bg-(--bg-card)')
})

test('SearchEverywhereDialog uses dynamic radius and has data-component', () => {
  const searchDialogPath = path.resolve('src/components/search/SearchEverywhereDialog.vue')
  const content = fs.readFileSync(searchDialogPath, 'utf-8')

  assert.ok(
    content.includes('data-component="search-everywhere-dialog"'),
    'SearchEverywhereDialog must have data-component="search-everywhere-dialog"'
  )
  assert.ok(
    content.includes('rounded-[var(--ui-radius-card)]'),
    'SearchEverywhereDialog modal content must use var(--ui-radius-card)'
  )
})
