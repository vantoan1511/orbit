import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('DeploymentsTable gates restart watcher and deployments decorator behind isHydrated', () => {
  const filePath = path.resolve('src/components/workloads/DeploymentsTable.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.match(
    content,
    /import\s+\{[^}]*useDeferredMount[^}]*\}\s+from\s+['"]@\/composables\/useDeferredMount['"]/,
    'DeploymentsTable must import useDeferredMount'
  )
  assert.match(
    content,
    /const\s+\{\s*isHydrated\s*\}\s*=\s*useDeferredMount\(\)/,
    'DeploymentsTable must call useDeferredMount()'
  )
  assert.match(
    content,
    /watch\(\s*\(\)\s*=>\s*\(isHydrated\.value\s*\?\s*restartFingerprint\(k8sStore\.pods\)\s*:\s*null\)/,
    'DeploymentsTable must gate restartFingerprint watcher behind isHydrated'
  )
  assert.match(
    content,
    /const\s+deployments\s*=\s*computed\(\(\)\s*=>\s*\{?\s*(?:return\s+)?\(?isHydrated\.value\s*\?\s*deploymentDecorator\(k8sStore\.deployments,\s*deploymentRestartsMap\.value\)\s*:\s*\[\]\)?/,
    'DeploymentsTable must gate deploymentDecorator computed behind isHydrated'
  )
})

test('ReplicaSetsTable, StatefulSetsTable, and DaemonSetsTable gate withReplicasSortKey behind isHydrated', () => {
  const files = [
    {
      file: 'src/components/workloads/ReplicaSetsTable.vue',
      ident: 'replicaSets',
      storeProp: 'replicaSets'
    },
    {
      file: 'src/components/workloads/StatefulSetsTable.vue',
      ident: 'statefulSets',
      storeProp: 'statefulSets'
    },
    {
      file: 'src/components/workloads/DaemonSetsTable.vue',
      ident: 'daemonSets',
      storeProp: 'daemonSets'
    }
  ]

  for (const { file, ident, storeProp } of files) {
    const filePath = path.resolve(file)
    const content = fs.readFileSync(filePath, 'utf-8')

    assert.match(
      content,
      /import\s+\{[^}]*useDeferredMount[^}]*\}\s+from\s+['"]@\/composables\/useDeferredMount['"]/,
      `${file} must import useDeferredMount`
    )
    assert.match(
      content,
      /const\s+\{\s*isHydrated\s*\}\s*=\s*useDeferredMount\(\)/,
      `${file} must call useDeferredMount()`
    )
    const pattern = new RegExp(
      `const\\s+${ident}\\s*=\\s*computed\\(\\(\\)\\s*=>\\s*\\{?\\s*(?:return\\s+)?\\(?isHydrated\\.value\\s*\\?\\s*withReplicasSortKey\\(k8sStore\\.${storeProp}\\)\\s*:\\s*\\[\\]\\)?`
    )
    assert.match(content, pattern, `${file} must gate ${ident} computed behind isHydrated`)
  }
})

test('PodsDataTable gates podDecorator behind isHydrated', () => {
  const filePath = path.resolve('src/components/pods/PodsDataTable.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.match(
    content,
    /import\s+\{[^}]*useDeferredMount[^}]*\}\s+from\s+['"]@\/composables\/useDeferredMount['"]/,
    'PodsDataTable must import useDeferredMount'
  )
  assert.match(
    content,
    /const\s+\{\s*isHydrated\s*\}\s*=\s*useDeferredMount\(\)/,
    'PodsDataTable must call useDeferredMount()'
  )
  assert.match(
    content,
    /const\s+pods\s*=\s*computed\(\(\)\s*=>\s*\(isHydrated\.value\s*\?\s*podDecorator\(k8sStore\.pods\)\s*:\s*\[\]\)\)/,
    'PodsDataTable must gate podDecorator computed behind isHydrated'
  )
})

test('JobsTable, CronJobsTable, and HpasTable gate data computations behind isHydrated', () => {
  // JobsTable
  const jobsContent = fs.readFileSync(
    path.resolve('src/components/workloads/JobsTable.vue'),
    'utf-8'
  )
  assert.match(
    jobsContent,
    /import\s+\{[^}]*useDeferredMount[^}]*\}\s+from\s+['"]@\/composables\/useDeferredMount['"]/,
    'JobsTable must import useDeferredMount'
  )
  assert.match(
    jobsContent,
    /const\s+jobs\s*=\s*computed\(\(\)\s*=>\s*\(isHydrated\.value\s*\?\s*k8sStore\.jobs\s*:\s*\[\]\)\)/,
    'JobsTable must compute jobs gated by isHydrated'
  )
  assert.match(jobsContent, /:data="jobs"/, 'JobsTable must pass :data="jobs"')

  // CronJobsTable
  const cronJobsContent = fs.readFileSync(
    path.resolve('src/components/workloads/CronJobsTable.vue'),
    'utf-8'
  )
  assert.match(
    cronJobsContent,
    /import\s+\{[^}]*useDeferredMount[^}]*\}\s+from\s+['"]@\/composables\/useDeferredMount['"]/,
    'CronJobsTable must import useDeferredMount'
  )
  assert.match(
    cronJobsContent,
    /if\s*\(!isHydrated\.value\)\s*return\s*\[\]/,
    'filteredCronJobs must return [] when !isHydrated.value'
  )

  // HpasTable
  const hpasContent = fs.readFileSync(
    path.resolve('src/components/workloads/HpasTable.vue'),
    'utf-8'
  )
  assert.match(
    hpasContent,
    /import\s+\{[^}]*useDeferredMount[^}]*\}\s+from\s+['"]@\/composables\/useDeferredMount['"]/,
    'HpasTable must import useDeferredMount'
  )
  assert.match(
    hpasContent,
    /if\s*\(!isHydrated\.value\)\s*return\s*\[\]/,
    'filteredHpas must return [] when !isHydrated.value'
  )
})
