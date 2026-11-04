import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('GeneralSettingsForm preserves data-component="general-settings-form"', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/settings/GeneralSettingsForm.vue')
  assert.ok(fs.existsSync(filePath), 'GeneralSettingsForm.vue must exist')

  const content = fs.readFileSync(filePath, 'utf-8')
  assert.ok(
    content.includes('data-component="general-settings-form"'),
    'GeneralSettingsForm must define data-component="general-settings-form"'
  )
})

test('GeneralSettingsForm removes "Coming soon" tag and opacity-60 from Danger Zone', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/settings/GeneralSettingsForm.vue')
  const content = fs.readFileSync(filePath, 'utf-8')
  const dangerZoneIndex = content.indexOf('Danger Zone')
  assert.ok(dangerZoneIndex !== -1, 'Danger Zone header must exist')

  const dangerZoneSection = content.slice(dangerZoneIndex)
  assert.ok(
    !dangerZoneSection.includes('Coming soon'),
    'Danger Zone section must not contain "Coming soon"'
  )
  assert.ok(
    !dangerZoneSection.includes('opacity-60'),
    'Danger Zone section must not deemphasize with opacity-60'
  )
})

test('GeneralSettingsForm enables all Danger Zone actions with confirmation hooks', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/settings/GeneralSettingsForm.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('useConfirm'),
    'GeneralSettingsForm must use PrimeVue useConfirm for dangerous actions'
  )
  assert.ok(
    content.includes('handleResetCache'),
    'GeneralSettingsForm must implement handleResetCache'
  )
  assert.ok(
    content.includes('handleDeleteCachedContexts'),
    'GeneralSettingsForm must implement handleDeleteCachedContexts'
  )
  assert.ok(
    content.includes('handleResetAllSettings'),
    'GeneralSettingsForm must implement handleResetAllSettings'
  )

  // Ensure buttons are not hardcoded disabled
  assert.ok(
    !content.includes('<Button severity="danger" variant="outlined" disabled'),
    'Danger Zone buttons must not be disabled'
  )
})
