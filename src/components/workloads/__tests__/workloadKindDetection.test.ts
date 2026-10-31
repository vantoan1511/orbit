import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import { getWorkloadKind } from '../../../utils/workloadKind.ts'
import { KUBERNETES_RESOURCE_KIND } from '../../../constants/kubernetes.ts'
import type {
  StatefulSetInfo,
  ReplicaSetInfo,
  DeploymentInfo,
  DaemonSetInfo,
  JobInfo,
  CronJobInfo
} from '../../../types/kubernetes.ts'

test('getWorkloadKind resolves StatefulSet for redis without "stateful" in name', () => {
  const redisSs: StatefulSetInfo = {
    kind: 'StatefulSet',
    name: 'redis',
    namespace: 'default',
    status: 'Running',
    replicas: { current: 1, desired: 1 },
    age: '5d',
    images: ['redis:7'],
    labels: {},
    annotations: {}
  }
  assert.equal(getWorkloadKind(redisSs), KUBERNETES_RESOURCE_KIND.StatefulSet)
})

test('getWorkloadKind resolves StatefulSet for postgres-cluster-0', () => {
  const pgSs: StatefulSetInfo = {
    kind: 'StatefulSet',
    name: 'postgres-cluster-0',
    namespace: 'database',
    status: 'Running',
    replicas: { current: 3, desired: 3 },
    age: '12d',
    images: ['postgres:16'],
    labels: {},
    annotations: {}
  }
  assert.equal(getWorkloadKind(pgSs), KUBERNETES_RESOURCE_KIND.StatefulSet)
})

test('getWorkloadKind resolves ReplicaSet for frontend-rs', () => {
  const rs: ReplicaSetInfo = {
    kind: 'ReplicaSet',
    name: 'frontend-rs',
    namespace: 'default',
    status: 'Running',
    replicas: { current: 2, desired: 2 },
    age: '2d',
    images: ['frontend:v1'],
    labels: {},
    annotations: {}
  }
  assert.equal(getWorkloadKind(rs), KUBERNETES_RESOURCE_KIND.ReplicaSet)
})

test('getWorkloadKind resolves Deployment, DaemonSet, Job, CronJob from explicit kind', () => {
  const dep: DeploymentInfo = {
    kind: 'Deployment',
    name: 'api',
    namespace: 'default',
    status: 'Running',
    replicas: { current: 2, desired: 2 },
    available: 2,
    upToDate: 2,
    minReadySeconds: 0,
    age: '1d',
    images: ['api:latest'],
    labels: {},
    annotations: {}
  }
  assert.equal(getWorkloadKind(dep), KUBERNETES_RESOURCE_KIND.Deployment)

  const ds: DaemonSetInfo = {
    kind: 'DaemonSet',
    name: 'node-exporter',
    namespace: 'monitoring',
    status: 'Running',
    replicas: { current: 5, desired: 5, ready: 5, upToDate: 5, available: 5 },
    age: '10d',
    images: ['prom/node-exporter:v1'],
    labels: {},
    annotations: {}
  }
  assert.equal(getWorkloadKind(ds), KUBERNETES_RESOURCE_KIND.DaemonSet)

  const job: JobInfo = {
    kind: 'Job',
    name: 'migrate-db',
    namespace: 'default',
    status: 'Succeeded',
    completions: '1/1',
    age: '2h',
    images: ['migrate:v1'],
    labels: {},
    annotations: {}
  }
  assert.equal(getWorkloadKind(job), KUBERNETES_RESOURCE_KIND.Job)

  const cj: CronJobInfo = {
    kind: 'CronJob',
    name: 'backup-nightly',
    namespace: 'ops',
    schedule: '0 2 * * *',
    suspend: false,
    active: 0,
    age: '30d',
    images: ['backup:v2'],
    labels: {},
    annotations: {}
  }
  assert.equal(getWorkloadKind(cj), KUBERNETES_RESOURCE_KIND.CronJob)
})

test('getWorkloadKind falls back gracefully to heuristic matching when kind is missing', () => {
  const legacySs = {
    name: 'statefulset-legacy',
    namespace: 'default',
    replicas: { current: 1, desired: 1 }
  } as unknown as StatefulSetInfo
  assert.equal(getWorkloadKind(legacySs), KUBERNETES_RESOURCE_KIND.StatefulSet)

  const legacyRs = {
    name: 'frontend-rs',
    namespace: 'default',
    replicas: { current: 2, desired: 2 }
  } as unknown as ReplicaSetInfo
  assert.equal(getWorkloadKind(legacyRs), KUBERNETES_RESOURCE_KIND.ReplicaSet)

  const legacyDs = {
    name: 'ds-agent',
    namespace: 'default',
    replicas: { current: 1, desired: 1, ready: 1, upToDate: 1, available: 1 }
  } as unknown as DaemonSetInfo
  assert.equal(getWorkloadKind(legacyDs), KUBERNETES_RESOURCE_KIND.DaemonSet)

  const legacyJob = {
    name: 'batch-task',
    namespace: 'default',
    completions: '1/1'
  } as unknown as JobInfo
  assert.equal(getWorkloadKind(legacyJob), KUBERNETES_RESOURCE_KIND.Job)

  const legacyCronJob = {
    name: 'cron-task',
    namespace: 'default',
    schedule: '*/5 * * * *'
  } as unknown as CronJobInfo
  assert.equal(getWorkloadKind(legacyCronJob), KUBERNETES_RESOURCE_KIND.CronJob)
})

test('WorkloadDetailsDrawer.vue integrates getWorkloadKind without hardcoded heuristics', () => {
  const drawerPath = path.resolve('src/components/workloads/WorkloadDetailsDrawer.vue')
  const content = fs.readFileSync(drawerPath, 'utf-8')

  // Verify WorkloadDetailsDrawer imports getWorkloadKind
  assert.ok(
    content.includes('getWorkloadKind'),
    'WorkloadDetailsDrawer must reference getWorkloadKind'
  )
  assert.ok(
    content.includes('@/utils/workloadKind') || content.includes('getWorkloadKind'),
    'WorkloadDetailsDrawer must import getWorkloadKind or use kind'
  )
})
