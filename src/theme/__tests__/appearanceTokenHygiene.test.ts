import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

const FORBIDDEN_CLASSES = [
  'bg-emerald-500',
  'bg-rose-500',
  'bg-amber-500',
  'bg-blue-500',
  'bg-violet-500',
  'bg-zinc-950',
  'text-emerald-500',
  'text-rose-500',
  'text-amber-500',
  'bg-gray-400',
  'bg-gray-500',
  'text-gray-400',
  'text-slate-50',
  'text-slate-950'
]

const ADAPTED_COMPONENTS = [
  // App Shell
  'src/components/layout/AppTitleBar.vue',
  'src/components/layout/WindowControls.vue',
  'src/components/layout/sidebar/AppSidebarActivityBar.vue',
  'src/components/layout/sidebar/AppSidebarPanel.vue',
  'src/components/layout/sidebar/AppSidebarClusters.vue',
  'src/components/layout/sidebar/AppSidebarNavMenu.vue',
  'src/components/layout/sidebar/AppSidebarLogsMenu.vue',
  'src/components/layout/tabs/AppTabBar.vue',
  'src/components/layout/tabs/AppTabItem.vue',
  'src/components/layout/AppFooter.vue',
  'src/views/OfflineClusterView.vue',
  'src/App.vue',
  'src/components/layout/AppLayout.vue',

  // Drawers & Search
  'src/components/shared/BaseResourceDrawer.vue',
  'src/components/workloads/WorkloadDetailsDrawer.vue',
  'src/components/policies/PolicyDetailsDrawer.vue',
  'src/components/events/EventDetailsDrawer.vue',
  'src/components/config/ConfigDetailsDrawer.vue',
  'src/components/layout/NotificationDrawer.vue',
  'src/components/shared/ResourceYamlTab.vue',
  'src/components/search/SearchEverywhereDialog.vue',
  'src/components/namespaces/NamespaceDetailsDrawer.vue',
  'src/components/nodes/NodeDetailsDrawer.vue',
  'src/components/network/IngressDetailsDrawer.vue',
  'src/components/services/ServiceDetailsDrawer.vue',
  'src/components/pods/PodDetailsDrawer.vue',

  // Dashboard & Pod/Service details
  'src/components/dashboard/AlertsEvents.vue',
  'src/components/dashboard/HealthOverview.vue',
  'src/components/dashboard/WorkloadSummaries.vue',
  'src/components/pods/PodContainersTab.vue',
  'src/components/services/ServiceEndpointsTab.vue',
  'src/components/workloads/WorkloadEventsTab.vue',
  'src/components/storage/StorageClassesSidebar.vue',

  // Workload Edit Forms
  'src/components/workloads/DeploymentEditForm.vue',
  'src/components/workloads/DaemonSetEditForm.vue',
  'src/components/workloads/StatefulSetEditForm.vue',
  'src/components/workloads/ReplicaSetEditForm.vue',
  'src/components/workloads/JobEditForm.vue',
  'src/components/workloads/PodEditForm.vue',
  'src/components/workloads/CreateCronJobDialog.vue',

  // Shared Editors
  'src/components/shared/ContainerEnvEditor.vue',
  'src/components/shared/ContainerPortsEditor.vue',
  'src/components/shared/KeyValueEditor.vue',
  'src/components/shared/StringListEditor.vue',

  // Shared Badges & Metric Cards
  'src/components/shared/StatusBadge.vue',
  'src/components/shared/ReplicasProgressBar.vue',
  'src/components/shared/HighlightedText.vue',
  'src/components/dashboard/StatusSummary.vue',
  'src/components/dashboard/ResourceUsage.vue',
  'src/components/pods/PodMetricsCards.vue',
  'src/components/nodes/NodeMetricsCards.vue',
  'src/components/namespaces/NamespaceMetricsCards.vue',
  'src/components/workloads/WorkloadMetricsCards.vue',
  'src/components/services/ServiceMetricsCards.vue',
  'src/components/storage/StorageMetricsCards.vue',
  'src/components/policies/PoliciesMetricsCards.vue',
  'src/components/config/ConfigMetricsCards.vue',
  'src/components/events/EventMetricsCards.vue',

  // Terminal & Logs
  'src/components/terminal/TerminalPanel.vue',
  'src/components/terminal/TerminalTab.vue',
  'src/views/LogsView.vue',

  // DataTables
  'src/components/pods/PodsDataTable.vue',
  'src/components/nodes/NodesTable.vue',
  'src/components/namespaces/NamespacesDataTable.vue',
  'src/components/workloads/DeploymentsTable.vue',
  'src/components/workloads/DaemonSetsTable.vue',
  'src/components/workloads/StatefulSetsTable.vue',
  'src/components/workloads/ReplicaSetsTable.vue',
  'src/components/workloads/JobsTable.vue',
  'src/components/workloads/CronJobsTable.vue',
  'src/components/network/ServicesTable.vue',
  'src/components/network/IngressesTable.vue',
  'src/components/network/PortForwardsTable.vue',
  'src/components/storage/StorageClassesTable.vue',
  'src/components/storage/StoragePVTable.vue',
  'src/components/storage/StoragePVCTable.vue',
  'src/components/policies/PoliciesDataTable.vue',

  // Settings
  'src/views/SettingsView.vue',
  'src/components/settings/SettingsAppearanceTab.vue',
  'src/components/settings/SettingsAboutTab.vue',
  'src/components/settings/preferences/ShortcutConflictModal.vue',
  'src/components/settings/appearance/ThemeModeSelector.vue',
  'src/components/settings/appearance/UiStyleSelector.vue',
  'src/components/settings/appearance/AccentColorPicker.vue',
  'src/components/settings/appearance/SemanticColorPicker.vue',
  'src/components/settings/appearance/SemanticPreviewCard.vue',
  'src/components/settings/GeneralSettingsForm.vue'
]

function collectVueFiles(dir: string): string[] {
  const results: string[] = []
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name)
    if (entry.isDirectory()) {
      results.push(...collectVueFiles(fullPath))
    } else if (entry.isFile() && entry.name.endsWith('.vue')) {
      results.push(fullPath)
    }
  }
  return results
}

test('verifies zero hardcoded semantic colors exist across adapted components and views', () => {
  const violations: Array<{ file: string; forbidden: string; line: number }> = []

  for (const relativePath of ADAPTED_COMPONENTS) {
    const file = path.resolve(relativePath)
    const content = fs.readFileSync(file, 'utf-8')
    const lines = content.split('\n')
    lines.forEach((lineText, idx) => {
      for (const forbidden of FORBIDDEN_CLASSES) {
        const regex = new RegExp(`\\b${forbidden}\\b`)
        if (regex.test(lineText)) {
          violations.push({
            file: relativePath,
            forbidden,
            line: idx + 1
          })
        }
      }
    })
  }

  assert.equal(
    violations.length,
    0,
    `Found hardcoded raw color violations in adapted components:\n` +
      violations.map((v) => `  ${v.file}:${v.line} uses ${v.forbidden}`).join('\n')
  )
})

test('verifies all domain tables have data-component attribute', () => {
  const tableFiles = [
    'src/components/pods/PodsDataTable.vue',
    'src/components/nodes/NodesTable.vue',
    'src/components/namespaces/NamespacesDataTable.vue',
    'src/components/workloads/DeploymentsTable.vue',
    'src/components/workloads/DaemonSetsTable.vue',
    'src/components/workloads/StatefulSetsTable.vue',
    'src/components/workloads/ReplicaSetsTable.vue',
    'src/components/workloads/JobsTable.vue',
    'src/components/workloads/CronJobsTable.vue',
    'src/components/network/ServicesTable.vue',
    'src/components/network/IngressesTable.vue',
    'src/components/network/PortForwardsTable.vue',
    'src/components/storage/StorageClassesTable.vue',
    'src/components/storage/StoragePVTable.vue',
    'src/components/storage/StoragePVCTable.vue',
    'src/components/policies/PoliciesDataTable.vue'
  ]

  for (const file of tableFiles) {
    const content = fs.readFileSync(path.resolve(file), 'utf-8')
    assert.match(
      content,
      /data-component="[a-z0-9-]+"/,
      `${file} must have a data-component attribute`
    )
  }
})

test('verifies all resource drawers have data-component attribute', () => {
  const drawerFiles = [
    'src/components/shared/BaseResourceDrawer.vue',
    'src/components/workloads/WorkloadDetailsDrawer.vue',
    'src/components/policies/PolicyDetailsDrawer.vue',
    'src/components/namespaces/NamespaceDetailsDrawer.vue',
    'src/components/nodes/NodeDetailsDrawer.vue',
    'src/components/network/IngressDetailsDrawer.vue',
    'src/components/services/ServiceDetailsDrawer.vue',
    'src/components/pods/PodDetailsDrawer.vue'
  ]

  for (const file of drawerFiles) {
    const content = fs.readFileSync(path.resolve(file), 'utf-8')
    assert.match(
      content,
      /data-component="[a-z0-9-]+"/,
      `${file} must have a data-component attribute`
    )
  }
})

test('base.css defines all core design tokens across all 5 UI styles (flat, glass, aero, liquidglass, ios)', () => {
  const baseCssPath = path.resolve(process.cwd(), 'src/assets/base.css')
  const content = fs.readFileSync(baseCssPath, 'utf-8')

  const requiredTokens = [
    '--ui-radius-control',
    '--ui-radius-card',
    '--ui-control-bg',
    '--ui-card-bg',
    '--ui-overlay-bg',
    '--ui-dialog-bg',
    '--ui-dialog-blur',
    '--ui-mask-bg',
    '--ui-mask-blur',
    '--border',
    '--border-subtle',
    '--ui-glass-blur',
    '--ui-sheen-highlight'
  ]

  for (const token of requiredTokens) {
    assert.ok(content.includes(`${token}:`), `base.css must define ${token}`)
  }

  for (const style of ['flat', 'glass', 'aero', 'liquidglass', 'ios']) {
    assert.ok(
      content.includes(`[data-ui-style='${style}']`) ||
        content.includes(`[data-ui-style="${style}"]`),
      `base.css must define [data-ui-style="${style}"] preset rules`
    )
  }
})

test('main.ts defines canonical global pass-through for dialog with ui-dialog-bg and ui-mask-bg', () => {
  const mainTsPath = path.resolve(process.cwd(), 'src/main.ts')
  const content = fs.readFileSync(mainTsPath, 'utf-8')

  assert.ok(content.includes('pt:'), 'main.ts must configure PrimeVue global pt')
  assert.ok(content.includes('dialog:'), 'main.ts pt must configure dialog')
  assert.ok(
    content.includes('bg-(--ui-dialog-bg)'),
    'main.ts dialog pt must configure bg-(--ui-dialog-bg)'
  )
  assert.ok(
    content.includes('bg-(--ui-mask-bg)'),
    'main.ts dialog pt must configure bg-(--ui-mask-bg)'
  )
  assert.ok(content.includes('ui-modal-mask'), 'main.ts dialog pt must configure ui-modal-mask')
  assert.ok(
    content.includes('rounded-[var(--ui-radius-card)]'),
    'main.ts dialog pt must use rounded-[var(--ui-radius-card)]'
  )
})

test('Stylesheets and Vue components contain zero ad-hoc .p-* CSS overrides or !important declarations', () => {
  const cssFiles = [
    path.resolve(process.cwd(), 'src/assets/base.css'),
    path.resolve(process.cwd(), 'src/assets/main.css')
  ]
  const vueFiles = collectVueFiles(path.resolve(process.cwd(), 'src'))
  const violations: string[] = []

  const pClassOverrideRegex =
    /\.p-(?:button|dialog|select|datatable|card|drawer|popover|inputtext|textarea|tag|inputnumber|tooltip)\b/

  for (const filePath of [...cssFiles, ...vueFiles]) {
    const content = fs.readFileSync(filePath, 'utf-8')
    const relPath = path.relative(process.cwd(), filePath).replace(/\\/g, '/')
    const lines = content.split(/\r?\n/)

    lines.forEach((line, idx) => {
      if (pClassOverrideRegex.test(line)) {
        violations.push(`${relPath}:${idx + 1}: ad-hoc PrimeVue class selector -> ${line.trim()}`)
      }
      if (line.includes('!important')) {
        violations.push(`${relPath}:${idx + 1}: !important declaration -> ${line.trim()}`)
      }
    })
  }

  assert.deepEqual(
    violations,
    [],
    `Found ad-hoc PrimeVue CSS overrides or !important declarations:\n${violations.join('\n')}`
  )
})

test('Vue views and components use design-token-aligned corner radius utilities and zero ! important class modifiers', () => {
  const vueFiles = collectVueFiles(path.resolve(process.cwd(), 'src'))
  const violations: string[] = []

  const hardcodedRadiusRegex = /\brounded-(?:sm|md|lg|xl|2xl|3xl)\b/
  const importantModifierRegex = /(?:^|\s|'|")!(?:[a-z]+-)|(?:[a-z0-9\])%-]+)!(?=\s|'|"$)/

  for (const filePath of vueFiles) {
    const content = fs.readFileSync(filePath, 'utf-8')
    const relPath = path.relative(process.cwd(), filePath).replace(/\\/g, '/')
    const lines = content.split(/\r?\n/)

    lines.forEach((line, idx) => {
      const radiusMatch = line.match(hardcodedRadiusRegex)
      if (radiusMatch) {
        violations.push(
          `${relPath}:${idx + 1}: hardcoded radius "${radiusMatch[0]}" -> ${line.trim()}`
        )
      }
      if (
        (line.includes('class') || line.includes("'") || line.includes('"')) &&
        !line.includes('Copied!') &&
        !line.includes('INVALID_SERVICE_NAME!') &&
        !line.includes('item.route!') &&
        importantModifierRegex.test(line)
      ) {
        violations.push(`${relPath}:${idx + 1}: important modifier (!) -> ${line.trim()}`)
      }
    })
  }

  assert.deepEqual(
    violations,
    [],
    `Found hardcoded corner-radius classes or ! modifiers in Vue files:\n${violations.slice(0, 30).join('\n')}${violations.length > 30 ? `\n...and ${violations.length - 30} more` : ''}`
  )
})
