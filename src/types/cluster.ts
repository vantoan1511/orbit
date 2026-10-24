export interface DiscoveredContextInfo {
  name: string
  clusterName: string
  serverUrl?: string
  userName?: string
  authType: 'Certificate' | 'Token' | 'Exec Plugin' | 'OIDC' | 'Basic Auth' | 'None'
  isCurrentContext: boolean
}

export interface AddClusterCustomization {
  alias?: string | null
  color?: string | null
  tags?: string[]
  description?: string | null
}

export interface AddClusterPayload {
  filePath?: string
  kubeconfigContent?: string
  selectedContexts?: string[]
  metadata?: AddClusterCustomization
  switchContext?: boolean
}

export interface ManualEndpointConfig {
  contextName: string
  clusterName: string
  serverUrl: string
  authType: 'token' | 'certificate' | 'basic' | 'none'
  token?: string
  clientCertData?: string
  clientKeyData?: string
  username?: string
  password?: string
  insecureSkipTlsVerify: boolean
  caCertData?: string
}
