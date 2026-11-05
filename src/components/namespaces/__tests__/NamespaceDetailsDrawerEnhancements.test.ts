import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'

test('NamespaceDetailsDrawer.vue includes YAML tab, live metrics, and real quota/limits parsing', () => {
  const filePath = path.resolve('src/components/namespaces/NamespaceDetailsDrawer.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // YAML tab support
  assert.ok(
    content.includes('useResourceRawManifest'),
    'NamespaceDetailsDrawer.vue should use useResourceRawManifest composable'
  )
  assert.ok(
    content.includes('ResourceYamlTab'),
    'NamespaceDetailsDrawer.vue should import ResourceYamlTab'
  )
  assert.ok(
    !content.includes(':show-yaml-tab="false"'),
    'NamespaceDetailsDrawer.vue should remove :show-yaml-tab="false"'
  )
  assert.ok(
    content.includes('<template #yaml>'),
    'NamespaceDetailsDrawer.vue should render <template #yaml> slot'
  )

  // Real Quotas, Limits, and Aggregated Metrics
  assert.ok(
    content.includes('extractNamespaceResourceQuota'),
    'NamespaceDetailsDrawer.vue should import extractNamespaceResourceQuota'
  )
  assert.ok(
    content.includes('extractNamespaceLimitRanges'),
    'NamespaceDetailsDrawer.vue should import extractNamespaceLimitRanges'
  )
  assert.ok(
    content.includes('aggregateNamespacePodMetrics'),
    'NamespaceDetailsDrawer.vue should import aggregateNamespacePodMetrics'
  )
  assert.ok(
    content.includes('useKubernetesStore'),
    'NamespaceDetailsDrawer.vue should access useKubernetesStore'
  )

  // No placeholder text
  assert.ok(
    !content.includes('Resource usage metrics are currently unavailable'),
    'NamespaceDetailsDrawer.vue should not contain the placeholder message'
  )
})
