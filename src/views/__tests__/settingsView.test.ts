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

test('resolveSettingsTab falls back to "general" when query is an unknown or disabled tab', () => {
  assert.equal(resolveSettingsTab('notifications'), 'general')
  assert.equal(resolveSettingsTab('proxy'), 'general')
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

test('ENABLED_SETTINGS_TABS contains implemented general, clusters, preferences, appearance, and about tabs', () => {
  assert.deepEqual(ENABLED_SETTINGS_TABS, [
    'general',
    'clusters',
    'preferences',
    'appearance',
    'about'
  ])
})
