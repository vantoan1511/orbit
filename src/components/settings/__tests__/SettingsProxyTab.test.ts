import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('SettingsProxyTab exists and defines data-component="settings-proxy-tab"', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/settings/SettingsProxyTab.vue')
  assert.ok(fs.existsSync(filePath), 'SettingsProxyTab.vue must exist')

  const content = fs.readFileSync(filePath, 'utf-8')
  assert.ok(
    content.includes('data-component="settings-proxy-tab"'),
    'SettingsProxyTab must define data-component="settings-proxy-tab"'
  )
})

test('SettingsProxyTab includes toggle switch for proxy status and inputs for HTTP, HTTPS, and Bypass', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/settings/SettingsProxyTab.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(content.includes('ToggleSwitch'), 'SettingsProxyTab must use PrimeVue ToggleSwitch')
  assert.ok(
    content.includes('InputText'),
    'SettingsProxyTab must use PrimeVue InputText for proxy URLs'
  )
  assert.ok(content.includes('httpProxy'), 'SettingsProxyTab must configure httpProxy')
  assert.ok(content.includes('httpsProxy'), 'SettingsProxyTab must configure httpsProxy')
  assert.ok(content.includes('noProxy'), 'SettingsProxyTab must configure noProxy')
})

test('SettingsProxyTab includes reset button with v-tooltip and RotateCcw icon', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/settings/SettingsProxyTab.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(content.includes('RotateCcw'), 'SettingsProxyTab must use RotateCcw icon for reset')
  assert.ok(content.includes('v-tooltip'), 'SettingsProxyTab must use PrimeVue v-tooltip directive')
  assert.ok(content.includes('Button'), 'SettingsProxyTab must use PrimeVue Button')
})

test('SettingsProxyTab adheres to token hygiene with zero ad-hoc .p-* overrides and dynamic radius', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/settings/SettingsProxyTab.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(!content.includes('.p-'), 'SettingsProxyTab must not contain ad-hoc .p-* overrides')
  assert.ok(
    !content.includes('!bg-') && !content.includes('!p-') && !content.includes('!border-'),
    'SettingsProxyTab must not use Tailwind !important modifiers'
  )
})
