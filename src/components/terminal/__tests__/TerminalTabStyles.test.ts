import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('TerminalTab resolves selection from CSS property and tags data-component', () => {
  const tabPath = path.resolve('src/components/terminal/TerminalTab.vue')
  const content = fs.readFileSync(tabPath, 'utf-8')

  assert.ok(
    content.includes('data-component="terminal-tab"'),
    'TerminalTab must have data-component="terminal-tab"'
  )
  assert.ok(
    content.includes('--selection'),
    'TerminalTab getTerminalTheme must read --selection CSS custom property'
  )
  assert.ok(
    content.includes('useAppearance'),
    'TerminalTab must import useAppearance to sync with accent color changes'
  )

  const panelPath = path.resolve('src/components/terminal/TerminalPanel.vue')
  const panelContent = fs.readFileSync(panelPath, 'utf-8')

  assert.ok(
    panelContent.includes('data-component="terminal-panel"'),
    'TerminalPanel must have data-component="terminal-panel"'
  )
})
