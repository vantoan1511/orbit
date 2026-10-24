import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('AppSidebarClusters component exists and defines data-component="sidebar-clusters"', () => {
  const filePath = path.resolve(
    process.cwd(),
    'src/components/layout/sidebar/AppSidebarClusters.vue'
  )
  assert.ok(fs.existsSync(filePath), 'AppSidebarClusters.vue must exist')

  const content = fs.readFileSync(filePath, 'utf-8')
  assert.ok(
    content.includes('data-component="sidebar-clusters"'),
    'AppSidebarClusters must define data-component="sidebar-clusters"'
  )
})

test('AppSidebarClusters imports clusterHelpers and ClusterEditModal', () => {
  const filePath = path.resolve(
    process.cwd(),
    'src/components/layout/sidebar/AppSidebarClusters.vue'
  )
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('clusterHelpers') ||
      (content.includes('filterClusters') && content.includes('getClusterDisplayName')),
    'AppSidebarClusters must import helper functions from clusterHelpers'
  )
  assert.ok(
    content.includes('ClusterEditModal'),
    'AppSidebarClusters must integrate ClusterEditModal'
  )
})

test('AppSidebarClusters contains search input, tag quick-filters, and manage clusters action', () => {
  const filePath = path.resolve(
    process.cwd(),
    'src/components/layout/sidebar/AppSidebarClusters.vue'
  )
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(content.includes('searchQuery'), 'AppSidebarClusters must have searchQuery state')
  assert.ok(
    content.includes('selectedTag'),
    'AppSidebarClusters must have selectedTag filter state'
  )
  assert.ok(
    content.includes('InputText'),
    'AppSidebarClusters must use PrimeVue InputText for search'
  )
  assert.ok(
    content.includes('/settings?tab=clusters'),
    'AppSidebarClusters must have link to /settings?tab=clusters'
  )
  assert.ok(
    content.includes('v-tooltip'),
    'AppSidebarClusters must use PrimeVue v-tooltip directive'
  )
  assert.ok(
    !content.includes('isRefreshing'),
    'AppSidebarClusters must not keep unused isRefreshing variable'
  )
})

test('AppSidebarClusters uses PrimeVue IconField and InputIcon for search filter bar', () => {
  const filePath = path.resolve(
    process.cwd(),
    'src/components/layout/sidebar/AppSidebarClusters.vue'
  )
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(content.includes('IconField'), 'AppSidebarClusters must use PrimeVue IconField')
  assert.ok(content.includes('InputIcon'), 'AppSidebarClusters must use PrimeVue InputIcon')
  assert.ok(
    content.includes('pi pi-search'),
    'AppSidebarClusters must use pi pi-search for search icon'
  )
  assert.ok(
    content.includes('pi pi-times'),
    'AppSidebarClusters must use pi pi-times for clear filter button'
  )
  assert.ok(
    !content.includes('class="w-5! h-5! p-0! absolute right-1.5'),
    'AppSidebarClusters must not use manual absolute positioning for clear button'
  )
})
