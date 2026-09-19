import test from 'node:test'
import assert from 'node:assert/strict'
import { KUBERNETES_RESOURCE_KIND } from '../../constants/kubernetes.ts'
import { getAvailablePorts } from '../portForward.ts'

test('getAvailablePorts returns ports for Service with portsList', () => {
  const service = {
    name: 'web-service',
    portsList: [{ port: 80 }, { port: 443 }]
  }
  const ports = getAvailablePorts(service, KUBERNETES_RESOURCE_KIND.Service)
  assert.deepEqual(ports, [80, 443])
})

test('getAvailablePorts returns empty array for Service without portsList', () => {
  const service = { name: 'web-service' }
  const ports = getAvailablePorts(service, KUBERNETES_RESOURCE_KIND.Service)
  assert.deepEqual(ports, [])
})

test('getAvailablePorts returns ports for Deployment with ports array', () => {
  const deployment = {
    name: 'web-deployment',
    ports: [80, 443, 8080]
  }
  const ports = getAvailablePorts(deployment, KUBERNETES_RESOURCE_KIND.Deployment)
  assert.deepEqual(ports, [80, 443, 8080])
})

test('getAvailablePorts returns empty array for Deployment without ports', () => {
  const deployment = { name: 'web-deployment' }
  const ports = getAvailablePorts(deployment, KUBERNETES_RESOURCE_KIND.Deployment)
  assert.deepEqual(ports, [])
})

test('getAvailablePorts returns ports for Pod with container ports string', () => {
  const pod = {
    name: 'web-pod',
    containers: [
      { name: 'nginx', ports: '80/TCP, 443/TCP' },
      { name: 'sidecar', ports: '8080/TCP' }
    ]
  }
  const ports = getAvailablePorts(pod, KUBERNETES_RESOURCE_KIND.Pod)
  assert.deepEqual(ports, [80, 443, 8080])
})

test('getAvailablePorts deduplicates ports for Pod with multiple identical ports', () => {
  const pod = {
    name: 'web-pod',
    containers: [
      { name: 'c1', ports: '80/TCP' },
      { name: 'c2', ports: '80/TCP' }
    ]
  }
  const ports = getAvailablePorts(pod, KUBERNETES_RESOURCE_KIND.Pod)
  assert.deepEqual(ports, [80])
})

test('getAvailablePorts returns ports for StatefulSet with ports array', () => {
  const statefulSet = {
    name: 'web-ss',
    ports: [6379]
  }
  const ports = getAvailablePorts(statefulSet, KUBERNETES_RESOURCE_KIND.StatefulSet)
  assert.deepEqual(ports, [6379])
})

test('getAvailablePorts returns empty array for unsupported resource kind or null row', () => {
  assert.deepEqual(getAvailablePorts(null, KUBERNETES_RESOURCE_KIND.Deployment), [])
  assert.deepEqual(getAvailablePorts(undefined, KUBERNETES_RESOURCE_KIND.Deployment), [])
  assert.deepEqual(getAvailablePorts({ name: 'pod-1' }, KUBERNETES_RESOURCE_KIND.Pod), [])
})
