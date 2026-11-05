import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'

test('ServicesTable.vue wires external link with nativeService and semantic tokens', () => {
  const filePath = path.resolve('src/components/network/ServicesTable.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // Check imports
  assert.ok(
    content.includes('resolveServiceExternalUrl'),
    'ServicesTable.vue should import resolveServiceExternalUrl'
  )
  assert.ok(
    content.includes("from '@/services/nativeService'") && content.includes('os'),
    'ServicesTable.vue should import os from nativeService'
  )

  // Check click handler and tooltip
  assert.ok(
    content.includes('handleOpenExternal'),
    'ServicesTable.vue should have handleOpenExternal'
  )
  assert.ok(
    content.includes('@click.stop="handleOpenExternal(data)"'),
    'ServicesTable.vue should bind click.stop on ExternalLink'
  )
  assert.ok(content.includes('v-tooltip.top'), 'ServicesTable.vue should use v-tooltip directive')

  // Verify no hardcoded violet colors
  assert.ok(
    !content.includes('text-violet-400'),
    'ServicesTable.vue should not contain hardcoded text-violet-400'
  )
  assert.ok(
    content.includes('text-(--accent)'),
    'ServicesTable.vue should use semantic text-(--accent)'
  )
})

test('ServiceOverviewTab.vue wires external link with nativeService and semantic tokens', () => {
  const filePath = path.resolve('src/components/services/ServiceOverviewTab.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('resolveServiceExternalUrl'),
    'ServiceOverviewTab.vue should import resolveServiceExternalUrl'
  )
  assert.ok(
    content.includes("from '@/services/nativeService'") && content.includes('os'),
    'ServiceOverviewTab.vue should import os from nativeService'
  )
  assert.ok(
    content.includes('handleOpenExternal'),
    'ServiceOverviewTab.vue should have handleOpenExternal'
  )
  assert.ok(
    content.includes('@click.stop="handleOpenExternal(service)"') ||
      content.includes('@click="handleOpenExternal(service)"'),
    'ServiceOverviewTab.vue should bind click on ExternalLink'
  )
  assert.ok(
    content.includes('v-tooltip.top'),
    'ServiceOverviewTab.vue should use v-tooltip directive'
  )
  assert.ok(
    !content.includes('text-violet-400'),
    'ServiceOverviewTab.vue should not contain hardcoded text-violet-400'
  )
  assert.ok(
    content.includes('text-(--accent)'),
    'ServiceOverviewTab.vue should use semantic text-(--accent)'
  )
})
