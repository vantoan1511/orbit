import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('Sidebar components contain data-component attributes and semantic accent tokens', () => {
  const activityBarPath = path.resolve('src/components/layout/sidebar/AppSidebarActivityBar.vue')
  const activityBarContent = fs.readFileSync(activityBarPath, 'utf-8')
  assert.ok(
    activityBarContent.includes('data-component="activity-bar"'),
    'AppSidebarActivityBar must have data-component="activity-bar"'
  )

  const panelPath = path.resolve('src/components/layout/sidebar/AppSidebarPanel.vue')
  const panelContent = fs.readFileSync(panelPath, 'utf-8')
  assert.ok(
    panelContent.includes('data-component="sidebar-panel"'),
    'AppSidebarPanel must have data-component="sidebar-panel"'
  )
  assert.ok(
    panelContent.includes('hover:bg-(--accent)') || panelContent.includes('hover:bg-primary'),
    'AppSidebarPanel resize handle must use accent'
  )

  const clustersPath = path.resolve('src/components/layout/sidebar/AppSidebarClusters.vue')
  const clustersContent = fs.readFileSync(clustersPath, 'utf-8')
  assert.ok(
    clustersContent.includes('data-component="sidebar-clusters"'),
    'AppSidebarClusters must have data-component="sidebar-clusters"'
  )

  const navMenuPath = path.resolve('src/components/layout/sidebar/AppSidebarNavMenu.vue')
  const navMenuContent = fs.readFileSync(navMenuPath, 'utf-8')
  assert.ok(
    navMenuContent.includes('data-component="sidebar-nav-menu"'),
    'AppSidebarNavMenu must have data-component="sidebar-nav-menu"'
  )
})
