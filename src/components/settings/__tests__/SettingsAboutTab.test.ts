import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('SettingsAboutTab resolves architecture via nativeService.app.getConfig without NL_ARCH or Neutralino references', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/settings/SettingsAboutTab.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // Zero NL_ or Neutralino references
  assert.ok(!content.includes('NL_ARCH'), 'SettingsAboutTab must not reference NL_ARCH')
  assert.ok(!content.includes('NL_'), 'SettingsAboutTab must not reference any NL_ global')
  assert.ok(
    !content.toLowerCase().includes('neutralino'),
    'SettingsAboutTab must not reference Neutralino'
  )

  // Must import app from nativeService and read config.arch
  assert.ok(content.includes("from '@/services/nativeService'"), 'Must import from nativeService')
  assert.ok(content.includes('app.getConfig()'), 'Must call app.getConfig()')
  assert.ok(content.includes('config.arch'), 'Must read config.arch')
})
