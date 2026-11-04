import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('GenericResourceTable action button uses template #icon and defines aria-label="Actions"', () => {
  const filePath = path.resolve('src/components/shared/GenericResourceTable.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // Locate the Actions Column
  const actionsColMatch = content.match(/<!-- Actions Column -->[\s\S]*?<\/Column>/)
  assert.ok(actionsColMatch, 'Actions Column must exist in GenericResourceTable.vue')
  const actionsCol = actionsColMatch[0]

  // Assert Button includes aria-label="Actions"
  assert.ok(
    actionsCol.includes('aria-label="Actions"'),
    'Action button must include aria-label="Actions"'
  )

  // Assert <MoreVertical> icon is nested inside <template #icon>
  assert.match(
    actionsCol,
    /<template\s+#icon>\s*<MoreVertical[\s\S]*?\/>\s*<\/template>/,
    'Action button icon must be nested inside <template #icon>'
  )
})
