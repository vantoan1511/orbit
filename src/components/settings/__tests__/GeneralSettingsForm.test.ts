import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('GeneralSettingsForm does not disable controls via settingsStore.isSaving', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/settings/GeneralSettingsForm.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    !content.includes('settingsStore.isSaving'),
    'GeneralSettingsForm must not bind controls or buttons to settingsStore.isSaving'
  )
})

test('GeneralSettingsForm per-field reset buttons do not use v-if and use icon template with disabled binding', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/settings/GeneralSettingsForm.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    !content.includes('v-if="isConfigModified(config)"'),
    'Per-field reset buttons must not use v-if="isConfigModified(config)"'
  )

  assert.ok(
    content.includes(':disabled="!config.enable || !isConfigModified(config)"'),
    'Per-field reset buttons must bind :disabled="!config.enable || !isConfigModified(config)"'
  )

  // Verify canonical icon-only button syntax per orbit-primevue.md
  assert.ok(
    content.includes('<template #icon>'),
    'Icon-only reset buttons must use <template #icon>'
  )
})

test('GeneralSettingsForm saves silently without Settings Saved toast but retains Settings Reset toast', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/settings/GeneralSettingsForm.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    !content.includes("'Settings Saved'"),
    'GeneralSettingsForm should not show "Settings Saved" toast on setting changes'
  )

  assert.ok(
    content.includes("'Settings Reset'"),
    'GeneralSettingsForm should show "Settings Reset" toast when a setting is reset'
  )
})

test('ENABLE_EXPERIENCE_FEATURES is registered in CONFIG_KEYS and treated as a general config key', async () => {
  const { CONFIG_KEYS, isGeneralConfigKey } = await import('../../../types/settings.ts')
  assert.equal(
    (CONFIG_KEYS as Record<string, string>).ENABLE_EXPERIENCE_FEATURES,
    'enableExperienceFeatures'
  )
  assert.equal(isGeneralConfigKey('enableExperienceFeatures'), true)
})
