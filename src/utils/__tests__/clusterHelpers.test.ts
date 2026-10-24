import assert from 'node:assert/strict'
import test from 'node:test'
import type { ClusterInfo } from '../../types/kubernetes.ts'
import { filterClusters, getAllClusterTags, getClusterDisplayName } from '../clusterHelpers.ts'

const sampleClusters: ClusterInfo[] = [
  {
    id: 'arn:aws:eks:us-west-2:123456789012:cluster/prod-eks-01',
    name: 'arn:aws:eks:us-west-2:123456789012:cluster/prod-eks-01',
    status: 'healthy',
    alias: 'Production US-West',
    tags: ['prod', 'aws', 'us-west'],
    color: '#ef4444',
    description: 'Main production EKS cluster'
  },
  {
    id: 'gke_my-project_europe-west1_staging-01',
    name: 'gke_my-project_europe-west1_staging-01',
    status: 'healthy',
    alias: 'Staging EU',
    tags: ['staging', 'gcp'],
    color: '#f59e0b',
    description: 'Staging environment on GKE'
  },
  {
    id: 'minikube',
    name: 'minikube',
    status: 'offline',
    tags: ['local', 'dev']
  }
]

test('getClusterDisplayName returns alias when defined', () => {
  assert.equal(getClusterDisplayName(sampleClusters[0]), 'Production US-West')
})

test('getClusterDisplayName falls back to name when alias is undefined or empty', () => {
  assert.equal(getClusterDisplayName(sampleClusters[2]), 'minikube')
  assert.equal(
    getClusterDisplayName({ id: '1', name: 'my-ctx', status: 'healthy', alias: '' }),
    'my-ctx'
  )
  assert.equal(getClusterDisplayName(null), '')
  assert.equal(getClusterDisplayName(undefined), '')
})

test('getAllClusterTags returns sorted unique tags across all clusters', () => {
  const tags = getAllClusterTags(sampleClusters)
  assert.deepEqual(tags, ['aws', 'dev', 'gcp', 'local', 'prod', 'staging', 'us-west'])
})

test('filterClusters returns all clusters when search query and tag are empty', () => {
  const filtered = filterClusters(sampleClusters, '')
  assert.equal(filtered.length, 3)
})

test('filterClusters matches search query against alias, name, description, and tags', () => {
  // Match alias
  assert.equal(filterClusters(sampleClusters, 'Production').length, 1)
  assert.equal(filterClusters(sampleClusters, 'Production')[0]?.alias, 'Production US-West')

  // Match raw context name
  assert.equal(filterClusters(sampleClusters, 'minikube').length, 1)

  // Match description
  assert.equal(filterClusters(sampleClusters, 'GKE').length, 1)

  // Match tag via search text
  assert.equal(filterClusters(sampleClusters, 'aws').length, 1)
})

test('filterClusters filters strictly by selectedTag', () => {
  const prodFiltered = filterClusters(sampleClusters, '', 'prod')
  assert.equal(prodFiltered.length, 1)
  assert.equal(prodFiltered[0]?.id, sampleClusters[0]?.id)

  const devFiltered = filterClusters(sampleClusters, '', 'dev')
  assert.equal(devFiltered.length, 1)
  assert.equal(devFiltered[0]?.id, 'minikube')

  // Nonexistent tag
  assert.equal(filterClusters(sampleClusters, '', 'nonexistent').length, 0)
})

test('filterClusters combines search query and selectedTag', () => {
  // Tag 'aws' and search 'prod' -> matches cluster 0
  assert.equal(filterClusters(sampleClusters, 'prod', 'aws').length, 1)

  // Tag 'gcp' and search 'prod' -> empty
  assert.equal(filterClusters(sampleClusters, 'prod', 'gcp').length, 0)
})
