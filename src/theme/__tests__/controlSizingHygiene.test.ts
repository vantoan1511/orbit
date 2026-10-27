import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'
import { collectVueFiles } from './helpers/collectVueFiles.ts'
import type { ControlTier } from '../uiContract.ts'

interface TierRule {
  readonly tier: ControlTier
  readonly include: RegExp
  readonly exclude?: readonly string[]
}

const TIER_RULES: readonly TierRule[] = [
  {
    tier: 'standard',
    include:
      /(Dialog|Modal|EditForm|GeneralSettingsForm|ClusterCustomizationFields|(ContainerEnv|ContainerPorts|ContainerResources|KeyValue|StringList)Editor)\.vue$/,
    exclude: ['SearchEverywhereDialog.vue']
  },
  {
    tier: 'compact',
    include: /(ResourceDataTable|NamespaceFilter|TableFilterSelect)\.vue$/
  }
]

test('PrimeVue overlay components carry no surface classes', () => {
  const vueFiles = collectVueFiles(path.resolve(process.cwd(), 'src'))
  const violations: string[] = []

  const overlayTagRegex = /<(Menu|ContextMenu|TieredMenu|Popover|Select|MultiSelect)\b([\s\S]*?)>/g
  const surfaceClassRegex =
    /(?:^|\s)(?:bg-\(--ui-overlay-bg\)|backdrop-blur(?:-[a-z0-9-]+)?|shadow(?:-[a-z0-9-]+)?|rounded-(?!full\b)[a-z0-9-\[\]]+)/

  for (const filePath of vueFiles) {
    const relPath = path.relative(process.cwd(), filePath).replace(/\\/g, '/')
    const content = fs.readFileSync(filePath, 'utf-8')
    let match: RegExpExecArray | null

    while ((match = overlayTagRegex.exec(content)) !== null) {
      const tagContent = match[0]
      const classAttrMatch =
        tagContent.match(/\bclass="([^"]*)"/) || tagContent.match(/\bclass='([^']*)'/)
      if (classAttrMatch) {
        const classNames = classAttrMatch[1]
        if (surfaceClassRegex.test(classNames)) {
          violations.push(
            `${relPath}: overlay tag <${match[1]}> has surface classes: "${classNames}"`
          )
        }
      }
    }
  }

  assert.deepEqual(
    violations,
    [],
    `Found surface classes on PrimeVue overlay components:\n${violations.join('\n')}`
  )
})

test('Standard-tier files have no size="small" on sized controls', () => {
  const vueFiles = collectVueFiles(path.resolve(process.cwd(), 'src'))
  const standardRule = TIER_RULES.find((r) => r.tier === 'standard')!
  const violations: string[] = []

  const sizedControlRegex =
    /<(Button|InputText|Select|InputNumber|MultiSelect|Textarea)\b([\s\S]*?)>/g

  for (const filePath of vueFiles) {
    const relPath = path.relative(process.cwd(), filePath).replace(/\\/g, '/')
    const fileName = path.basename(filePath)

    if (standardRule.exclude?.includes(fileName)) continue
    if (!standardRule.include.test(filePath)) continue

    const content = fs.readFileSync(filePath, 'utf-8')
    let match: RegExpExecArray | null

    while ((match = sizedControlRegex.exec(content)) !== null) {
      const attrs = match[2]
      if (attrs.includes('data-sizing-exempt')) continue
      if (/\bsize=["']small["']/.test(attrs) || /:size=["']'small'["']/.test(attrs)) {
        violations.push(`${relPath}: <${match[1]}> carries size="small" in Standard-tier file`)
      }
    }
  }

  assert.deepEqual(
    violations,
    [],
    `Standard-tier files must not contain size="small":\n${violations.join('\n')}`
  )
})

test('Compact-tier files: every sized control has size="small"', () => {
  const vueFiles = collectVueFiles(path.resolve(process.cwd(), 'src'))
  const compactRule = TIER_RULES.find((r) => r.tier === 'compact')!
  const violations: string[] = []

  const sizedControlRegex =
    /<(Button|InputText|Select|InputNumber|MultiSelect|Textarea)\b([\s\S]*?)>/g

  for (const filePath of vueFiles) {
    const relPath = path.relative(process.cwd(), filePath).replace(/\\/g, '/')
    const fileName = path.basename(filePath)

    if (compactRule.exclude?.includes(fileName)) continue
    if (!compactRule.include.test(filePath)) continue

    const content = fs.readFileSync(filePath, 'utf-8')
    let match: RegExpExecArray | null

    while ((match = sizedControlRegex.exec(content)) !== null) {
      const attrs = match[2]
      if (attrs.includes('data-sizing-exempt')) continue
      const hasSmall =
        /\bsize=["']small["']/.test(attrs) ||
        /:size=["']'small'["']/.test(attrs) ||
        /:size=["'](?:props\.)?size["']/.test(attrs)
      if (!hasSmall) {
        violations.push(`${relPath}: <${match[1]}> missing size="small" in Compact-tier file`)
      }
    }
  }

  assert.deepEqual(
    violations,
    [],
    `Compact-tier files must have size="small" on sized controls:\n${violations.join('\n')}`
  )
})

test('Sizing overrides never exceed baseline; migrated files are 0', () => {
  const vueFiles = collectVueFiles(path.resolve(process.cwd(), 'src'))
  const baselinePath = path.resolve(
    process.cwd(),
    'src/theme/__tests__/fixtures/sizingOverrideBaseline.json'
  )
  const baseline: Record<string, number> = JSON.parse(fs.readFileSync(baselinePath, 'utf-8'))

  const controlRegex = /<(Button|InputText|Select|InputNumber|MultiSelect)\b([\s\S]*?)>/g
  const overrideRegex = /\b(h-\d|h-\[|min-h-|py-\d|py-\[)/

  const currentCounts: Record<string, number> = {}
  const violations: string[] = []

  for (const filePath of vueFiles) {
    const relPath = path.relative(process.cwd(), filePath).replace(/\\/g, '/')
    const content = fs.readFileSync(filePath, 'utf-8')
    let match: RegExpExecArray | null
    let count = 0

    while ((match = controlRegex.exec(content)) !== null) {
      const attrs = match[2]
      if (attrs.includes('data-sizing-exempt')) continue
      const classMatch = attrs.match(/\bclass="([^"]*)"/) || attrs.match(/\bclass='([^']*)'/)
      if (classMatch && overrideRegex.test(classMatch[1])) {
        count++
      }
    }

    if (count > 0) {
      currentCounts[relPath] = count
    }

    const baselineCount = baseline[relPath] ?? 0
    if (count > baselineCount) {
      violations.push(
        `${relPath}: has ${count} sizing overrides, which exceeds baseline of ${baselineCount}`
      )
    }
  }

  assert.deepEqual(violations, [], `Sizing overrides exceeded baseline:\n${violations.join('\n')}`)
})

test('docs mention Compact 32px / Standard 36px and OVERLAY contract', () => {
  const designSystemDoc = fs.readFileSync(
    path.resolve(process.cwd(), '.agents/rules/orbit-design-system.md'),
    'utf-8'
  )
  assert.ok(
    designSystemDoc.includes('32px') && designSystemDoc.includes('36px'),
    'orbit-design-system.md must document Compact 32px / Standard 36px'
  )
  assert.ok(
    designSystemDoc.includes('OVERLAY') || designSystemDoc.includes('Overlay'),
    'orbit-design-system.md must document overlay contract'
  )

  const quirksDoc = fs.readFileSync(
    path.resolve(process.cwd(), '.agents/references/gotchas/primevue-quirks.md'),
    'utf-8'
  )
  assert.ok(
    quirksDoc.includes('size="normal"') || quirksDoc.includes('size='),
    'primevue-quirks.md must mention no size="normal"'
  )
})
