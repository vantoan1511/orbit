import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('SearchEverywhereDialog component exists and uses centralized dialog styling', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/search/SearchEverywhereDialog.vue')
  assert.ok(fs.existsSync(filePath), 'SearchEverywhereDialog.vue must exist')

  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('data-component="search-everywhere-dialog"'),
    'SearchEverywhereDialog must define data-component="search-everywhere-dialog"'
  )

  // Must not hardcode custom mask backdrop blur or tint
  assert.ok(
    !content.includes('bg-black/50'),
    'SearchEverywhereDialog must not hardcode bg-black/50 on mask'
  )
  assert.ok(
    !content.includes('backdrop-blur-[2px]'),
    'SearchEverywhereDialog must not hardcode backdrop-blur-[2px]'
  )

  // Content must use centralized dialog bg token
  assert.ok(
    content.includes('bg-(--ui-dialog-bg)'),
    'SearchEverywhereDialog must use bg-(--ui-dialog-bg) for dialog content surface'
  )
})
