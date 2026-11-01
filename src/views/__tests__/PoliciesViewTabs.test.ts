import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('PoliciesView defines exactly the 5 aligned category tabs without rbac', () => {
  const filePath = path.resolve('src/views/PoliciesView.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // Check tabs array
  assert.ok(content.includes("{ id: 'overview' }"), 'overview tab present')
  assert.ok(content.includes("{ id: 'network' }"), 'network tab present')
  assert.ok(content.includes("{ id: 'pod-security' }"), 'pod-security tab present')
  assert.ok(content.includes("{ id: 'resource' }"), 'resource tab present')
  assert.ok(content.includes("{ id: 'admission' }"), 'admission tab present')
  assert.ok(!content.includes("{ id: 'rbac' }"), 'rbac tab must be removed')

  // Check elimination of placeholder strings
  assert.ok(!content.toLowerCase().includes('coming soon'), 'No "coming soon" placeholders allowed')

  // Check tab wiring to PoliciesDataTable
  assert.ok(
    content.includes('category-filter="overview"') ||
      content.includes(':category-filter="\'overview\'"') ||
      content.includes('categoryFilter="overview"')
  )
  assert.ok(
    content.includes('category-filter="network"') ||
      content.includes(':category-filter="\'network\'"') ||
      content.includes('categoryFilter="network"')
  )
  assert.ok(
    content.includes('category-filter="pod-security"') ||
      content.includes(':category-filter="\'pod-security\'"') ||
      content.includes('categoryFilter="pod-security"')
  )
  assert.ok(
    content.includes('category-filter="resource"') ||
      content.includes(':category-filter="\'resource\'"') ||
      content.includes('categoryFilter="resource"')
  )
  assert.ok(
    content.includes('category-filter="admission"') ||
      content.includes(':category-filter="\'admission\'"') ||
      content.includes('categoryFilter="admission"')
  )
})
