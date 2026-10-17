import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('AppBreadcrumbs component defines PrimeVue Breadcrumb with route hierarchy and footer tokens', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/layout/AppBreadcrumbs.vue')
  assert.ok(fs.existsSync(filePath), 'AppBreadcrumbs.vue must exist')

  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes("import Breadcrumb from 'primevue/breadcrumb'"),
    'AppBreadcrumbs must import Breadcrumb from primevue/breadcrumb'
  )
  assert.ok(
    content.includes("from 'vue-router'"),
    'AppBreadcrumbs must import useRoute from vue-router'
  )
  assert.ok(
    content.includes('getResourceBreadcrumbs'),
    'AppBreadcrumbs must import getResourceBreadcrumbs from utils/breadcrumb'
  )
  assert.ok(
    content.includes('data-testid="footer-breadcrumbs"'),
    'AppBreadcrumbs must define a container with data-testid="footer-breadcrumbs"'
  )
  assert.ok(
    content.includes('text-[11px]'),
    'AppBreadcrumbs must use compact text-[11px] typography matching the footer token'
  )
  assert.ok(
    content.includes('truncate'),
    'AppBreadcrumbs must truncate long resource labels to prevent layout overflow'
  )
  assert.ok(content.includes('pi pi-home'), 'AppBreadcrumbs must provide home route icon')
})
