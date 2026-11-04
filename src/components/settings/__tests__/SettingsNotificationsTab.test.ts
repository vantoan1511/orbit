import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('SettingsNotificationsTab exists and defines data-component="settings-notifications-tab"', () => {
  const filePath = path.resolve(
    process.cwd(),
    'src/components/settings/SettingsNotificationsTab.vue'
  )
  assert.ok(fs.existsSync(filePath), 'SettingsNotificationsTab.vue must exist')

  const content = fs.readFileSync(filePath, 'utf-8')
  assert.ok(
    content.includes('data-component="settings-notifications-tab"'),
    'SettingsNotificationsTab must define data-component="settings-notifications-tab"'
  )
})

test('SettingsNotificationsTab uses PrimeVue ToggleSwitch controls for notification preferences', () => {
  const filePath = path.resolve(
    process.cwd(),
    'src/components/settings/SettingsNotificationsTab.vue'
  )
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('ToggleSwitch'),
    'SettingsNotificationsTab must use PrimeVue ToggleSwitch'
  )
  assert.ok(
    content.includes('enableToasts'),
    'SettingsNotificationsTab must configure enableToasts'
  )
  assert.ok(
    content.includes('notifyOnSuccess'),
    'SettingsNotificationsTab must configure notifyOnSuccess'
  )
  assert.ok(
    content.includes('notifyOnError'),
    'SettingsNotificationsTab must configure notifyOnError'
  )
  assert.ok(
    content.includes('notifyOnUpdates'),
    'SettingsNotificationsTab must configure notifyOnUpdates'
  )
})

test('SettingsNotificationsTab defines reset action with v-tooltip and RotateCcw icon', () => {
  const filePath = path.resolve(
    process.cwd(),
    'src/components/settings/SettingsNotificationsTab.vue'
  )
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('RotateCcw'),
    'SettingsNotificationsTab must use RotateCcw icon for reset'
  )
  assert.ok(
    content.includes('v-tooltip'),
    'SettingsNotificationsTab must use PrimeVue v-tooltip directive'
  )
  assert.ok(content.includes('Button'), 'SettingsNotificationsTab must use PrimeVue Button')
})

test('SettingsNotificationsTab adheres to token hygiene with zero ad-hoc .p-* overrides and dynamic radius', () => {
  const filePath = path.resolve(
    process.cwd(),
    'src/components/settings/SettingsNotificationsTab.vue'
  )
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    !content.includes('.p-'),
    'SettingsNotificationsTab must not contain ad-hoc .p-* overrides'
  )
  assert.ok(
    !content.includes('!bg-') && !content.includes('!p-') && !content.includes('!border-'),
    'SettingsNotificationsTab must not use Tailwind !important modifiers'
  )
})
