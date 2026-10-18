import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('BaseResourceDrawer and specialized drawers use semantic status classes and data-component', () => {
  const baseDrawerPath = path.resolve('src/components/shared/BaseResourceDrawer.vue')
  const baseDrawerContent = fs.readFileSync(baseDrawerPath, 'utf-8')
  assert.ok(
    baseDrawerContent.includes('data-component="resource-drawer"'),
    'BaseResourceDrawer must have data-component="resource-drawer"'
  )
  assert.ok(
    baseDrawerContent.includes("statusBadgeClass: 'bg-(--success)'"),
    'BaseResourceDrawer default statusBadgeClass must be bg-(--success)'
  )
  assert.ok(
    !baseDrawerContent.includes("statusBadgeClass: 'bg-emerald-500'"),
    'BaseResourceDrawer must not default to bg-emerald-500'
  )

  const workloadDrawerPath = path.resolve('src/components/workloads/WorkloadDetailsDrawer.vue')
  const workloadDrawerContent = fs.readFileSync(workloadDrawerPath, 'utf-8')
  assert.ok(
    workloadDrawerContent.includes('data-component="workload-details-drawer"'),
    'WorkloadDetailsDrawer must have data-component="workload-details-drawer"'
  )
  assert.ok(
    !workloadDrawerContent.includes("'bg-emerald-500'"),
    'WorkloadDetailsDrawer must not have hardcoded bg-emerald-500'
  )

  const policyDrawerPath = path.resolve('src/components/policies/PolicyDetailsDrawer.vue')
  const policyDrawerContent = fs.readFileSync(policyDrawerPath, 'utf-8')
  assert.ok(
    policyDrawerContent.includes('data-component="policy-details-drawer"'),
    'PolicyDetailsDrawer must have data-component="policy-details-drawer"'
  )
  assert.ok(
    !policyDrawerContent.includes('bg-zinc-950'),
    'PolicyDetailsDrawer must not have hardcoded bg-zinc-950'
  )

  const podDrawerPath = path.resolve('src/components/pods/PodDetailsDrawer.vue')
  const podDrawerContent = fs.readFileSync(podDrawerPath, 'utf-8')
  assert.ok(
    podDrawerContent.includes('data-component="pod-details-drawer"'),
    'PodDetailsDrawer must have data-component="pod-details-drawer"'
  )
})
