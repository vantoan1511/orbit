import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('AppTitleBar component defines data-tauri-drag-region and integrates WindowControls', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/layout/AppTitleBar.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('data-tauri-drag-region'),
    'AppTitleBar must include data-tauri-drag-region attribute for native window dragging'
  )
  assert.ok(
    content.includes('WindowControls'),
    'AppTitleBar must integrate WindowControls component'
  )
  assert.ok(
    content.includes('toggleMaximize'),
    'AppTitleBar must support double click to toggle maximize'
  )
})

test('AppLayout integrates AppTitleBar at root of window layout', () => {
  const layoutPath = path.resolve(process.cwd(), 'src/components/layout/AppLayout.vue')
  const content = fs.readFileSync(layoutPath, 'utf-8')

  assert.ok(content.includes('AppTitleBar'), 'AppLayout must mount AppTitleBar')
})

test('AppTitleBar component integrates centered Search Everywhere bar without inline shortcut hint', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/layout/AppTitleBar.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(content.includes('useSearchStore'), 'AppTitleBar must import useSearchStore')
  assert.ok(content.includes('useShortcutStore'), 'AppTitleBar must import useShortcutStore')
  assert.ok(
    content.includes('searchStore.open()'),
    'AppTitleBar must trigger searchStore.open() on click'
  )
  assert.ok(
    content.includes('absolute left-1/2'),
    'Search bar must be centered using absolute left-1/2'
  )
  assert.ok(
    content.includes('-translate-x-1/2'),
    'Search bar must be translated -translate-x-1/2 for true center'
  )
  assert.ok(
    content.includes('@click.stop'),
    'Click on search bar must stop propagation to prevent window dragging'
  )
  assert.ok(
    content.includes('@dblclick.stop'),
    'Double click on search bar must stop propagation to prevent maximize toggle'
  )
  assert.ok(content.includes('Search everywhere...'), 'Search bar must render placeholder label')
  assert.ok(content.includes('justify-start'), 'Search bar button must use justify-start alignment')
  assert.ok(!content.includes('<kbd'), 'Search bar must not contain inline <kbd> shortcut badge')
  assert.ok(
    !content.includes('Ctrl+Shift+P'),
    'Search bar must not hardcode legacy Ctrl+Shift+P shortcut'
  )
  assert.ok(
    content.includes('v-tooltip.bottom="searchTooltip"'),
    'Search bar must bind tooltip to dynamic searchTooltip computed property'
  )
})

test('AppHeader component is retired and replaced by AppBreadcrumbs in AppFooter', () => {
  const headerPath = path.resolve(process.cwd(), 'src/components/layout/AppHeader.vue')
  assert.ok(!fs.existsSync(headerPath), 'AppHeader.vue must be deleted/retired')
})

test('AppLayout component does not mount retired AppHeader and mounts AppTabBar flush with content', () => {
  const layoutPath = path.resolve(process.cwd(), 'src/components/layout/AppLayout.vue')
  const content = fs.readFileSync(layoutPath, 'utf-8')

  assert.ok(!content.includes('AppHeader'), 'AppLayout must not import or mount AppHeader')
  assert.ok(content.includes('AppTabBar'), 'AppLayout must mount AppTabBar')
})

test('AppTitleBar component removes Orbit logo and brand label', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/layout/AppTitleBar.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(!content.includes('src="/logo.png"'), 'AppTitleBar must remove static Orbit logo image')
  assert.ok(!content.includes('>Orbit</span>'), 'AppTitleBar must remove static Orbit brand text')
})

test('AppTitleBar component integrates active cluster info and health status', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/layout/AppTitleBar.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(content.includes('useCluster'), 'AppTitleBar must import useCluster')
  assert.ok(content.includes('useKubernetesStore'), 'AppTitleBar must import useKubernetesStore')
  assert.ok(content.includes('activeCluster'), 'AppTitleBar must bind activeCluster')
  assert.ok(content.includes('activeCluster.name'), 'AppTitleBar must render activeCluster.name')
  assert.ok(
    content.includes('No active cluster'),
    'AppTitleBar must render fallback when activeCluster is null'
  )
  assert.ok(
    content.includes('clusterTooltip'),
    'AppTitleBar must provide rich hover tooltip with cluster metadata'
  )
  assert.ok(
    content.includes('KubernetesIcon'),
    'AppTitleBar must import KubernetesIcon for responsive metadata'
  )
  assert.ok(
    content.includes('hidden xl:flex'),
    'AppTitleBar must render responsive inline metadata on wide screens'
  )
  assert.ok(
    content.includes('@dblclick.stop'),
    'AppTitleBar cluster info element must stop double click propagation'
  )
  assert.ok(
    content.includes('pointer-events-auto'),
    'AppTitleBar cluster info element must enable pointer events for tooltips'
  )
})
