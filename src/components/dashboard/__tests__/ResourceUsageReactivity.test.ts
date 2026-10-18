import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('ResourceUsage component establishes reactive dependency on accentColor', () => {
  const filePath = path.resolve('src/components/dashboard/ResourceUsage.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes("import { useAppearance } from '@/composables/useAppearance'"),
    'ResourceUsage must import useAppearance composable'
  )
  assert.ok(
    content.includes('accentColor'),
    'ResourceUsage must extract accentColor from useAppearance'
  )
  assert.ok(
    content.includes('accentColor.value'),
    'ResourceUsage cpuChartData must react to accentColor.value changes'
  )
})
