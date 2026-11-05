import test from 'node:test'
import assert from 'node:assert/strict'
import { resolveServiceExternalUrl } from '../serviceEndpoints.ts'
import type { ServiceInfo } from '../../types/kubernetes.ts'

function createMockService(overrides: Partial<ServiceInfo> = {}): ServiceInfo {
  return {
    name: 'test-service',
    namespace: 'default',
    type: 'LoadBalancer',
    clusterIP: '10.96.0.1',
    externalIP: '-',
    ports: '80/TCP',
    endpoints: '10.244.0.5:80',
    age: '1d',
    sessionAffinity: 'None',
    created: '2026-01-01T00:00:00Z',
    uid: 'test-uid',
    selector: {},
    labels: {},
    portsList: [],
    endpointsList: [],
    events: [],
    ...overrides
  }
}

test('resolveServiceExternalUrl returns null for empty or dash externalIP', () => {
  assert.equal(resolveServiceExternalUrl(createMockService({ externalIP: '-' })), null)
  assert.equal(resolveServiceExternalUrl(createMockService({ externalIP: '' })), null)
  assert.equal(resolveServiceExternalUrl(createMockService({ externalIP: '   ' })), null)
})

test('resolveServiceExternalUrl returns URL as-is if externalIP already contains http:// or https://', () => {
  assert.equal(
    resolveServiceExternalUrl(createMockService({ externalIP: 'https://api.example.com' })),
    'https://api.example.com'
  )
  assert.equal(
    resolveServiceExternalUrl(createMockService({ externalIP: 'http://my-domain.org:8080' })),
    'http://my-domain.org:8080'
  )
})

test('resolveServiceExternalUrl handles port 443 as https without port suffix', () => {
  const serviceWithPortList = createMockService({
    externalIP: '203.0.113.10',
    portsList: [{ port: 443, name: 'https', protocol: 'TCP' }]
  })
  assert.equal(resolveServiceExternalUrl(serviceWithPortList), 'https://203.0.113.10')

  const serviceWithPortsString = createMockService({
    externalIP: '203.0.113.10',
    ports: '443:31443/TCP'
  })
  assert.equal(resolveServiceExternalUrl(serviceWithPortsString), 'https://203.0.113.10')
})

test('resolveServiceExternalUrl handles port 80 as http without port suffix', () => {
  const service = createMockService({
    externalIP: '203.0.113.10',
    portsList: [{ port: 80, name: 'http', protocol: 'TCP' }]
  })
  assert.equal(resolveServiceExternalUrl(service), 'http://203.0.113.10')
})

test('resolveServiceExternalUrl handles custom port with http://<ip>:<port>', () => {
  const service = createMockService({
    externalIP: '203.0.113.10',
    portsList: [{ port: 8080, name: 'http-alt', protocol: 'TCP' }]
  })
  assert.equal(resolveServiceExternalUrl(service), 'http://203.0.113.10:8080')
})

test('resolveServiceExternalUrl prioritizes port 443 if multiple ports exist', () => {
  const service = createMockService({
    externalIP: '203.0.113.10',
    portsList: [
      { port: 80, name: 'http', protocol: 'TCP' },
      { port: 443, name: 'https', protocol: 'TCP' }
    ]
  })
  assert.equal(resolveServiceExternalUrl(service), 'https://203.0.113.10')
})
