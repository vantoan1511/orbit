import test from 'node:test'
import assert from 'node:assert/strict'
import { resolveSettingsTab } from '../../utils/settingsViewHelpers.ts'
import { VALID_SETTINGS_TABS, ENABLED_SETTINGS_TABS } from '../../types/settings.ts'

test('resolveSettingsTab defaults to "general" when query is undefined or empty', () => {
  assert.equal(resolveSettingsTab(undefined), 'general')
  assert.equal(resolveSettingsTab(''), 'general')
  assert.equal(resolveSettingsTab(null), 'general')
})

test('resolveSettingsTab resolves "about" when query is "about"', () => {
  assert.equal(resolveSettingsTab('about'), 'about')
})

test('resolveSettingsTab resolves "appearance" when query is "appearance"', () => {
  assert.equal(resolveSettingsTab('appearance'), 'appearance')
})

test('resolveSettingsTab resolves "preferences" when query is "preferences"', () => {
  assert.equal(resolveSettingsTab('preferences'), 'preferences')
})

test('resolveSettingsTab resolves "clusters" when query is "clusters"', () => {
  assert.equal(resolveSettingsTab('clusters'), 'clusters')
})

test('resolveSettingsTab resolves "notifications" when query is "notifications"', () => {
  assert.equal(resolveSettingsTab('notifications'), 'notifications')
})

test('resolveSettingsTab resolves "proxy" when query is "proxy"', () => {
  assert.equal(resolveSettingsTab('proxy'), 'proxy')
})

test('resolveSettingsTab falls back to "general" when query is an unknown or invalid tab', () => {
  assert.equal(resolveSettingsTab('invalid-tab'), 'general')
  assert.equal(resolveSettingsTab(123), 'general')
})

test('VALID_SETTINGS_TABS contains all 7 expected tabs', () => {
  assert.deepEqual(VALID_SETTINGS_TABS, [
    'general',
    'clusters',
    'preferences',
    'appearance',
    'notifications',
    'proxy',
    'about'
  ])
})

test('ENABLED_SETTINGS_TABS contains all 7 enabled tabs', () => {
  assert.deepEqual(ENABLED_SETTINGS_TABS, [
    'general',
    'clusters',
    'preferences',
    'appearance',
    'notifications',
    'proxy',
    'about'
  ])
})

test('SettingsView.vue contains zero disabled attributes on Tab elements and mounts all 7 tabs', async () => {
  const fs = await import('node:fs')
  const path = await import('node:path')
  const filePath = path.resolve(process.cwd(), 'src/views/SettingsView.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(!content.includes('disabled'), 'SettingsView.vue should not contain disabled tabs')
  assert.ok(
    !content.includes('opacity-40 cursor-not-allowed'),
    'SettingsView.vue should not contain disabled cursor classes'
  )
  assert.ok(
    !content.toLowerCase().includes('coming soon'),
    'SettingsView.vue should not contain coming soon placeholder text'
  )
  assert.ok(
    content.includes('SettingsNotificationsTab'),
    'SettingsView.vue must import and mount SettingsNotificationsTab'
  )
  assert.ok(
    content.includes('SettingsProxyTab'),
    'SettingsView.vue must import and mount SettingsProxyTab'
  )
})
