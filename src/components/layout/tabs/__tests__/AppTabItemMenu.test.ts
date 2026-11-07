import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('AppTabItem ContextMenu delegates root surface to PrimeVue preset and delegates hover/focus states', () => {
  const filePath = path.resolve('src/components/layout/tabs/AppTabItem.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // ContextMenu root surface delegates styling to PrimeVue preset; must not have ad-hoc surface classes
  assert.ok(
    !content.includes('bg-(--ui-overlay-bg)'),
    'AppTabItem ContextMenu must not have ad-hoc bg-(--ui-overlay-bg) in template'
  )
  assert.ok(
    !content.includes('bg-(--bg-card)'),
    'AppTabItem ContextMenu must not use bg-(--bg-card)'
  )

  // Inner action element must not have conflicting hover:bg-(--bg-hover)
  assert.ok(
    !content.includes('hover:bg-(--bg-hover)'),
    'AppTabItem ContextMenu must not have conflicting hover:bg-(--bg-hover) on inner action link'
  )
  assert.ok(
    content.includes('text-inherit bg-transparent'),
    'AppTabItem ContextMenu inner action link must use text-inherit bg-transparent'
  )
})

test('AppTabBar handleCloseAllTabs does not navigate to Overview route and EditView goBack does not fallback to Overview when closing tracked tab', () => {
  const tabBarPath = path.resolve('src/components/layout/tabs/AppTabBar.vue')
  const tabBarContent = fs.readFileSync(tabBarPath, 'utf-8')

  assert.ok(
    tabBarContent.includes('const handleCloseAllTabs = () => {'),
    'AppTabBar must define handleCloseAllTabs'
  )
  assert.ok(
    !tabBarContent.includes("void router.push('/')"),
    'AppTabBar handleCloseAllTabs must not push root / route when closing all tabs'
  )

  const editViewPath = path.resolve('src/views/EditView.vue')
  const editViewContent = fs.readFileSync(editViewPath, 'utf-8')
  assert.ok(
    editViewContent.includes(
      'if (nextRoute) {\n      void router.replace(nextRoute)\n    }\n    return'
    ),
    'EditView goBack must return after closing tracked tab so it does not fallback to root route'
  )
})
