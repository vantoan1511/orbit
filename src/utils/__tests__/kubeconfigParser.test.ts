import assert from 'node:assert/strict'
import test from 'node:test'
import {
  parseKubeconfigYaml,
  filterKubeconfigContexts,
  buildKubeconfigFromEndpoint
} from '../kubeconfigParser.ts'
import type { ManualEndpointConfig } from '@/types/cluster'

const MULTI_CONTEXT_YAML = `
apiVersion: v1
kind: Config
current-context: staging-context
clusters:
- name: staging-cluster
  cluster:
    server: https://k8s-staging.example.com:6443
- name: prod-cluster
  cluster:
    server: https://k8s-prod.example.com:6443
- name: dev-cluster
  cluster:
    server: https://k8s-dev.example.com:6443
users:
- name: token-user
  user:
    token: secret-token-xyz
- name: cert-user
  user:
    client-certificate-data: LS0tLS...
    client-key-data: LS0tLS...
- name: exec-user
  user:
    exec:
      apiVersion: client.authentication.k8s.io/v1beta1
      command: aws
- name: oidc-user
  user:
    auth-provider:
      name: oidc
contexts:
- name: staging-context
  context:
    cluster: staging-cluster
    user: token-user
- name: prod-context
  context:
    cluster: prod-cluster
    user: cert-user
- name: exec-context
  context:
    cluster: dev-cluster
    user: exec-user
- name: oidc-context
  context:
    cluster: dev-cluster
    user: oidc-user
`

test('parseKubeconfigYaml correctly parses contexts, clusters, and auth types', () => {
  const contexts = parseKubeconfigYaml(MULTI_CONTEXT_YAML)
  assert.equal(contexts.length, 4)

  const staging = contexts.find((c) => c.name === 'staging-context')
  assert.ok(staging)
  assert.equal(staging.clusterName, 'staging-cluster')
  assert.equal(staging.serverUrl, 'https://k8s-staging.example.com:6443')
  assert.equal(staging.userName, 'token-user')
  assert.equal(staging.authType, 'Token')
  assert.equal(staging.isCurrentContext, true)

  const prod = contexts.find((c) => c.name === 'prod-context')
  assert.ok(prod)
  assert.equal(prod.authType, 'Certificate')
  assert.equal(prod.isCurrentContext, false)

  const exec = contexts.find((c) => c.name === 'exec-context')
  assert.ok(exec)
  assert.equal(exec.authType, 'Exec Plugin')

  const oidc = contexts.find((c) => c.name === 'oidc-context')
  assert.ok(oidc)
  assert.equal(oidc.authType, 'OIDC')
})

test('parseKubeconfigYaml handles empty or invalid YAML gracefully', () => {
  assert.deepEqual(parseKubeconfigYaml(''), [])
  assert.deepEqual(parseKubeconfigYaml('invalid: [}'), [])
  assert.deepEqual(parseKubeconfigYaml('just a plain string'), [])
  assert.deepEqual(parseKubeconfigYaml('apiVersion: v1\nkind: Config'), [])
})

test('filterKubeconfigContexts keeps only selected contexts, clusters, and users', () => {
  const filteredYaml = filterKubeconfigContexts(MULTI_CONTEXT_YAML, ['prod-context'])
  const parsed = parseKubeconfigYaml(filteredYaml)

  assert.equal(parsed.length, 1)
  assert.equal(parsed[0].name, 'prod-context')
  assert.equal(parsed[0].clusterName, 'prod-cluster')
  assert.equal(parsed[0].serverUrl, 'https://k8s-prod.example.com:6443')
  assert.equal(parsed[0].authType, 'Certificate')
  assert.equal(parsed[0].isCurrentContext, true)

  // Staging and dev clusters should be eliminated from YAML
  assert.ok(!filteredYaml.includes('staging-cluster'))
  assert.ok(!filteredYaml.includes('dev-cluster'))
  assert.ok(!filteredYaml.includes('token-user'))
  assert.ok(!filteredYaml.includes('exec-user'))
})

test('buildKubeconfigFromEndpoint generates valid Kubeconfig YAML for token auth', () => {
  const config: ManualEndpointConfig = {
    contextName: 'custom-dev-ctx',
    clusterName: 'custom-dev-cluster',
    serverUrl: 'https://192.168.1.100:6443',
    authType: 'token',
    token: 'my-bearer-token',
    insecureSkipTlsVerify: true
  }

  const yamlOutput = buildKubeconfigFromEndpoint(config)
  const parsed = parseKubeconfigYaml(yamlOutput)

  assert.equal(parsed.length, 1)
  assert.equal(parsed[0].name, 'custom-dev-ctx')
  assert.equal(parsed[0].clusterName, 'custom-dev-cluster')
  assert.equal(parsed[0].serverUrl, 'https://192.168.1.100:6443')
  assert.equal(parsed[0].authType, 'Token')
  assert.equal(parsed[0].isCurrentContext, true)
  assert.ok(yamlOutput.includes('insecure-skip-tls-verify: true'))
})

test('buildKubeconfigFromEndpoint generates valid Kubeconfig YAML for client certificate auth', () => {
  const config: ManualEndpointConfig = {
    contextName: 'cert-ctx',
    clusterName: 'cert-cluster',
    serverUrl: 'https://10.96.0.1:6443',
    authType: 'certificate',
    clientCertData: 'CERT_DATA_BASE64',
    clientKeyData: 'KEY_DATA_BASE64',
    insecureSkipTlsVerify: false,
    caCertData: 'CA_DATA_BASE64'
  }

  const yamlOutput = buildKubeconfigFromEndpoint(config)
  const parsed = parseKubeconfigYaml(yamlOutput)

  assert.equal(parsed.length, 1)
  assert.equal(parsed[0].name, 'cert-ctx')
  assert.equal(parsed[0].authType, 'Certificate')
  assert.ok(yamlOutput.includes('certificate-authority-data: CA_DATA_BASE64'))
  assert.ok(yamlOutput.includes('client-certificate-data: CERT_DATA_BASE64'))
  assert.ok(yamlOutput.includes('client-key-data: KEY_DATA_BASE64'))
})

test('buildKubeconfigFromEndpoint prefixes https:// when protocol is missing', () => {
  const config: ManualEndpointConfig = {
    contextName: 'no-proto-ctx',
    clusterName: 'no-proto-cluster',
    serverUrl: '192.168.1.50:6443',
    authType: 'token',
    token: 'test-token',
    insecureSkipTlsVerify: true
  }

  const yamlOutput = buildKubeconfigFromEndpoint(config)
  const parsed = parseKubeconfigYaml(yamlOutput)

  assert.equal(parsed.length, 1)
  assert.equal(parsed[0].serverUrl, 'https://192.168.1.50:6443')
})

test('buildKubeconfigFromEndpoint encodes raw PEM strings to base64 for cert and key fields', () => {
  const pemCertHeader = '-----BEGIN ' + 'CERTIFICATE-----'
  const pemKeyHeader = '-----BEGIN ' + 'RSA PRIVATE KEY-----'
  const rawPemCert = `${pemCertHeader}\nMIIDXTCCAkWgAwIBAgIJAP...\n-----END CERTIFICATE-----`
  const rawPemKey = `${pemKeyHeader}\nMIIEowIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----`
  const rawPemCa = `${pemCertHeader}\nMIIB...CA\n-----END CERTIFICATE-----`

  const config: ManualEndpointConfig = {
    contextName: 'pem-ctx',
    clusterName: 'pem-cluster',
    serverUrl: 'https://pem.k8s.local:6443',
    authType: 'certificate',
    clientCertData: rawPemCert,
    clientKeyData: rawPemKey,
    caCertData: rawPemCa,
    insecureSkipTlsVerify: false
  }

  const yamlOutput = buildKubeconfigFromEndpoint(config)
  assert.ok(!yamlOutput.includes(pemCertHeader))
  assert.ok(!yamlOutput.includes(pemKeyHeader))

  // Verify it is base64 encoded
  const expectedCertB64 = Buffer.from(rawPemCert).toString('base64')
  assert.ok(yamlOutput.includes(expectedCertB64))
})
