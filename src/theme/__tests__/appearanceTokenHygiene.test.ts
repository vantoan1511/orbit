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
  'text-amber-500'
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
  'src/components/shared/ResourceYamlTab.vue',
  'src/components/search/SearchEverywhereDialog.vue',
  'src/components/namespaces/NamespaceDetailsDrawer.vue',
  'src/components/nodes/NodeDetailsDrawer.vue',
  'src/components/network/IngressDetailsDrawer.vue',
  'src/components/services/ServiceDetailsDrawer.vue',
  'src/components/pods/PodDetailsDrawer.vue',

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
  'src/components/settings/appearance/ThemeModeSelector.vue',
  'src/components/settings/appearance/UiStyleSelector.vue',
  'src/components/settings/appearance/AccentColorPicker.vue',
  'src/components/settings/appearance/SemanticColorPicker.vue',
  'src/components/settings/appearance/SemanticPreviewCard.vue',
  'src/components/settings/GeneralSettingsForm.vue'
]

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
