import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('Settings components contain data-component attributes', () => {
  const settingsFiles = [
    { file: 'src/views/SettingsView.vue', tag: 'settings-view' },
    { file: 'src/components/settings/SettingsAppearanceTab.vue', tag: 'settings-appearance-tab' },
    { file: 'src/components/settings/appearance/ThemeModeSelector.vue', tag: 'theme-mode-selector' },
    { file: 'src/components/settings/appearance/UiStyleSelector.vue', tag: 'ui-style-selector' },
    { file: 'src/components/settings/appearance/AccentColorPicker.vue', tag: 'accent-color-picker' },
    { file: 'src/components/settings/appearance/SemanticColorPicker.vue', tag: 'semantic-color-picker' },
    { file: 'src/components/settings/appearance/SemanticPreviewCard.vue', tag: 'semantic-preview-card' },
  ]

  for (const { file, tag } of settingsFiles) {
    const filePath = path.resolve(file)
    const content = fs.readFileSync(filePath, 'utf-8')
    assert.ok(
      content.includes(`data-component="${tag}"`),
      `${file} must contain data-component="${tag}"`
    )
  }
})

test('GeneralSettingsForm avoids hardcoded rose-500 tint', () => {
  const formPath = path.resolve('src/components/settings/GeneralSettingsForm.vue')
  const content = fs.readFileSync(formPath, 'utf-8')
  assert.ok(
    !content.includes('bg-rose-500/5'),
    'GeneralSettingsForm must not contain hardcoded bg-rose-500/5'
  )
})
