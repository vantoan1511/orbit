import assert from 'node:assert/strict'
import test from 'node:test'
import { getTabMetadataForRoute } from '../tabIcons.ts'

test('getTabMetadataForRoute resolves Core routes', () => {
  const dashboard = getTabMetadataForRoute('/')
  assert.equal(dashboard.title, 'Overview')
  assert.equal(dashboard.iconName, 'LayoutDashboard')
  assert.equal(dashboard.category, 'core')

  const nodes = getTabMetadataForRoute('/nodes')
  assert.equal(nodes.title, 'Nodes')
  assert.equal(nodes.iconName, 'Server')
  assert.equal(nodes.category, 'core')

  const namespaces = getTabMetadataForRoute('/namespaces')
  assert.equal(namespaces.title, 'Namespaces')
  assert.equal(namespaces.iconName, 'FolderOpen')
  assert.equal(namespaces.category, 'core')

  const events = getTabMetadataForRoute('/events')
  assert.equal(events.title, 'Events')
  assert.equal(events.iconName, 'Activity')
  assert.equal(events.category, 'core')
})

test('getTabMetadataForRoute resolves Workloads sub-tabs', () => {
  const deployments = getTabMetadataForRoute('/workloads', { tab: 'deployments' })
  assert.equal(deployments.title, 'Deployments')
  assert.equal(deployments.iconName, 'Layers')
  assert.equal(deployments.category, 'workloads')

  const pods = getTabMetadataForRoute('/workloads', { tab: 'pods' })
  assert.equal(pods.title, 'Pods')
  assert.equal(pods.iconName, 'Box')
  assert.equal(pods.category, 'workloads')

  const fallbackWorkloads = getTabMetadataForRoute('/workloads')
  assert.equal(fallbackWorkloads.title, 'Deployments')
})

test('getTabMetadataForRoute resolves Network, Config, Storage, Security, Logs', () => {
  const services = getTabMetadataForRoute('/network', { tab: 'services' })
  assert.equal(services.title, 'Services')
  assert.equal(services.iconName, 'Network')
  assert.equal(services.category, 'network')

  const configmaps = getTabMetadataForRoute('/config', { tab: 'configmaps' })
  assert.equal(configmaps.title, 'ConfigMaps')
  assert.equal(configmaps.iconName, 'FileJson')
  assert.equal(configmaps.category, 'config')

  const policies = getTabMetadataForRoute('/policies')
  assert.equal(policies.title, 'Policies')
  assert.equal(policies.iconName, 'ShieldCheck')
  assert.equal(policies.category, 'security')

  const logs = getTabMetadataForRoute('/logs')
  assert.equal(logs.title, 'Logs')
  assert.equal(logs.iconName, 'FileText')
  assert.equal(logs.category, 'logs')

  const settings = getTabMetadataForRoute('/settings')
  assert.equal(settings.title, 'Settings')
  assert.equal(settings.iconName, 'Settings')
})

test('getTabMetadataForRoute provides graceful fallback for unknown routes', () => {
  const unknown = getTabMetadataForRoute('/some-unknown-path')
  assert.equal(unknown.title, 'Some Unknown Path')
  assert.equal(unknown.iconName, 'LayoutDashboard')
  assert.ok(unknown.icon)
})
