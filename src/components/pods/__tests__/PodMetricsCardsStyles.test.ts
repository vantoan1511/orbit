import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('PodMetricsCards uses semantic tokens and avoids hardcoded palette colors', () => {
  const podCardsPath = path.resolve('src/components/pods/PodMetricsCards.vue')
  const content = fs.readFileSync(podCardsPath, 'utf-8')

  assert.ok(
    content.includes('data-component="pod-metrics-cards"'),
    'PodMetricsCards must have data-component="pod-metrics-cards"'
  )
  assert.ok(content.includes('text-(--success)'), 'PodMetricsCards must use text-(--success)')
  assert.ok(
    !content.includes('text-emerald-500'),
    'PodMetricsCards must not contain text-emerald-500'
  )
  assert.ok(!content.includes('bg-rose-500/10'), 'PodMetricsCards must not contain bg-rose-500/10')
})

test('All metric card components have designated data-component attributes', () => {
  const metricsFiles = [
    { file: 'src/components/pods/PodMetricsCards.vue', tag: 'pod-metrics-cards' },
    { file: 'src/components/nodes/NodeMetricsCards.vue', tag: 'node-metrics-cards' },
    { file: 'src/components/namespaces/NamespaceMetricsCards.vue', tag: 'namespace-metrics-cards' },
    { file: 'src/components/workloads/WorkloadMetricsCards.vue', tag: 'workload-metrics-cards' },
    { file: 'src/components/services/ServiceMetricsCards.vue', tag: 'service-metrics-cards' },
    { file: 'src/components/storage/StorageMetricsCards.vue', tag: 'storage-metrics-cards' },
    { file: 'src/components/policies/PoliciesMetricsCards.vue', tag: 'policies-metrics-cards' },
    { file: 'src/components/config/ConfigMetricsCards.vue', tag: 'config-metrics-cards' },
    { file: 'src/components/events/EventMetricsCards.vue', tag: 'event-metrics-cards' }
  ]

  for (const { file, tag } of metricsFiles) {
    const filePath = path.resolve(file)
    const content = fs.readFileSync(filePath, 'utf-8')
    assert.ok(
      content.includes(`data-component="${tag}"`),
      `${file} must contain data-component="${tag}"`
    )
  }
})
