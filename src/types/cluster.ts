export const COLOR_PRESETS = [
  { label: 'Red (Production)', hex: '#ef4444' },
  { label: 'Orange', hex: '#f97316' },
  { label: 'Amber (Staging)', hex: '#f59e0b' },
  { label: 'Emerald (Dev)', hex: '#10b981' },
  { label: 'Cyan (Test)', hex: '#06b6d4' },
  { label: 'Blue', hex: '#3b82f6' },
  { label: 'Purple', hex: '#8b5cf6' },
  { label: 'Slate (Neutral)', hex: '#64748b' }
] as const

export type ColorPreset = (typeof COLOR_PRESETS)[number]

export const AUTH_TYPE_OPTIONS: Array<{
  label: string
  value: 'token' | 'certificate' | 'basic' | 'none'
}> = [
  { label: 'Bearer Token', value: 'token' },
  { label: 'Client Certificate & Key', value: 'certificate' },
  { label: 'Basic Authentication', value: 'basic' },
  { label: 'None / Anonymous', value: 'none' }
]

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
