import * as yaml from 'yaml'
import type { DiscoveredContextInfo, ManualEndpointConfig } from '@/types/cluster'
import { encodeBase64 } from './text.ts'

interface RawKubeContext {
  name: string
  context?: {
    cluster?: string
    user?: string
    namespace?: string
  }
}

interface RawKubeCluster {
  name: string
  cluster?: {
    server?: string
    'certificate-authority-data'?: string
    'insecure-skip-tls-verify'?: boolean
  }
}

interface RawKubeUser {
  name: string
  user?: {
    token?: string
    tokenFile?: string
    'client-certificate-data'?: string
    'client-certificate'?: string
    'client-key-data'?: string
    'client-key'?: string
    exec?: Record<string, unknown>
    'auth-provider'?: Record<string, unknown>
    username?: string
    password?: string
  }
}

interface RawKubeconfig {
  apiVersion?: string
  kind?: string
  'current-context'?: string
  contexts?: RawKubeContext[]
  clusters?: RawKubeCluster[]
  users?: RawKubeUser[]
  preferences?: Record<string, unknown>
}

type AuthType = DiscoveredContextInfo['authType']

/**
 * Strategy-based detector for authentication type in Kubeconfig user object.
 */
function resolveAuthType(userObj: RawKubeUser['user'] | undefined): AuthType {
  if (!userObj) {
    return 'None'
  }
  if (userObj.token || userObj.tokenFile) {
    return 'Token'
  }
  if (userObj['client-certificate-data'] || userObj['client-certificate']) {
    return 'Certificate'
  }
  if (userObj.exec) {
    return 'Exec Plugin'
  }
  if (userObj['auth-provider']) {
    return 'OIDC'
  }
  if (userObj.username && userObj.password) {
    return 'Basic Auth'
  }
  return 'None'
}

/**
 * Parses raw YAML string into a list of discovered contexts with cluster and auth metadata.
 */
export function parseKubeconfigYaml(yamlContent: string): DiscoveredContextInfo[] {
  if (!yamlContent || typeof yamlContent !== 'string' || !yamlContent.trim()) {
    return []
  }

  try {
    const doc = yaml.parse(yamlContent) as RawKubeconfig | null
    if (!doc || typeof doc !== 'object' || !Array.isArray(doc.contexts)) {
      return []
    }

    const clustersMap = new Map<string, RawKubeCluster['cluster']>()
    if (Array.isArray(doc.clusters)) {
      for (const cl of doc.clusters) {
        if (cl?.name) {
          clustersMap.set(cl.name, cl.cluster)
        }
      }
    }

    const usersMap = new Map<string, RawKubeUser['user']>()
    if (Array.isArray(doc.users)) {
      for (const u of doc.users) {
        if (u?.name) {
          usersMap.set(u.name, u.user)
        }
      }
    }

    const currentContext = doc['current-context'] || ''

    return doc.contexts
      .filter((ctx): ctx is RawKubeContext => Boolean(ctx && ctx.name))
      .map((ctx) => {
        const clusterName = ctx.context?.cluster || ''
        const cluster = clustersMap.get(clusterName)
        const userName = ctx.context?.user || ''
        const user = usersMap.get(userName)

        return {
          name: ctx.name,
          clusterName,
          serverUrl: cluster?.server,
          userName: userName || undefined,
          authType: resolveAuthType(user),
          isCurrentContext: ctx.name === currentContext
        }
      })
  } catch {
    return []
  }
}

/**
 * Filters a multi-context Kubeconfig YAML to preserve only selected contexts,
 * pruning unreferenced clusters and users to produce a minimal, clean config.
 */
export function filterKubeconfigContexts(yamlContent: string, selectedContexts: string[]): string {
  if (!yamlContent || !selectedContexts.length) {
    return yamlContent
  }

  try {
    const doc = yaml.parse(yamlContent) as RawKubeconfig | null
    if (!doc || typeof doc !== 'object' || !Array.isArray(doc.contexts)) {
      return yamlContent
    }

    const selectedSet = new Set(selectedContexts)
    const keptContexts = doc.contexts.filter((c) => c && selectedSet.has(c.name))
    if (!keptContexts.length) {
      return yamlContent
    }

    const activeClusters = new Set(keptContexts.map((c) => c.context?.cluster).filter(Boolean))
    const activeUsers = new Set(keptContexts.map((c) => c.context?.user).filter(Boolean))

    const filteredClusters = Array.isArray(doc.clusters)
      ? doc.clusters.filter((cl) => cl && activeClusters.has(cl.name))
      : []

    const filteredUsers = Array.isArray(doc.users)
      ? doc.users.filter((u) => u && activeUsers.has(u.name))
      : []

    let currentContext = doc['current-context']
    if (!currentContext || !selectedSet.has(currentContext)) {
      currentContext = keptContexts[0]?.name ?? ''
    }

    const filteredDoc: RawKubeconfig = {
      apiVersion: doc.apiVersion || 'v1',
      kind: doc.kind || 'Config',
      preferences: doc.preferences || {},
      'current-context': currentContext,
      contexts: keptContexts,
      clusters: filteredClusters,
      users: filteredUsers
    }

    return yaml.stringify(filteredDoc)
  } catch {
    return yamlContent
  }
}

/**
 * Normalizes a Kubernetes API server URL by prepending 'https://' if the protocol scheme is omitted.
 */
export function normalizeServerUrl(rawUrl: string): string {
  const trimmed = rawUrl.trim()
  if (!trimmed) return ''
  if (/^https?:\/\//i.test(trimmed)) {
    return trimmed
  }
  return `https://${trimmed}`
}

/**
 * Normalizes certificate/key data by encoding raw PEM content to base64.
 * Returns existing base64 strings unchanged.
 */
export function normalizeCertData(data: string | undefined): string | undefined {
  if (!data) return undefined
  const trimmed = data.trim()
  if (!trimmed) return undefined
  if (trimmed.startsWith('-----BEGIN ')) {
    return encodeBase64(trimmed)
  }
  return trimmed
}

/**
 * Constructs a valid Kubeconfig YAML string from manual API endpoint configuration parameters.
 */
export function buildKubeconfigFromEndpoint(config: ManualEndpointConfig): string {
  const clusterName = config.clusterName.trim() || 'default-cluster'
  const contextName = config.contextName.trim() || 'default-context'
  const userName = `${clusterName}-user`

  const clusterConfig: NonNullable<RawKubeCluster['cluster']> = {
    server: normalizeServerUrl(config.serverUrl)
  }

  const normalizedCa = normalizeCertData(config.caCertData)
  if (config.insecureSkipTlsVerify) {
    clusterConfig['insecure-skip-tls-verify'] = true
  } else if (normalizedCa) {
    clusterConfig['certificate-authority-data'] = normalizedCa
  }

  const userConfig: NonNullable<RawKubeUser['user']> = {}

  if (config.authType === 'token' && config.token) {
    userConfig.token = config.token.trim()
  } else if (config.authType === 'certificate') {
    const clientCert = normalizeCertData(config.clientCertData)
    const clientKey = normalizeCertData(config.clientKeyData)
    if (clientCert) {
      userConfig['client-certificate-data'] = clientCert
    }
    if (clientKey) {
      userConfig['client-key-data'] = clientKey
    }
  } else if (config.authType === 'basic') {
    if (config.username?.trim()) {
      userConfig.username = config.username.trim()
    }
    if (config.password) {
      userConfig.password = config.password
    }
  }

  const doc: RawKubeconfig = {
    apiVersion: 'v1',
    kind: 'Config',
    preferences: {},
    'current-context': contextName,
    clusters: [
      {
        name: clusterName,
        cluster: clusterConfig
      }
    ],
    users: [
      {
        name: userName,
        user: userConfig
      }
    ],
    contexts: [
      {
        name: contextName,
        context: {
          cluster: clusterName,
          user: userName
        }
      }
    ]
  }

  return yaml.stringify(doc)
}
