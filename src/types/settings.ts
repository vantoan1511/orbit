export type ConfigurationDataType = 'string' | 'number' | 'boolean'
export type ConfigurationCardinality = '0..1' | '0..*' | '1..*' | '1..1' | (string & {})

/**
 * Well-known configuration keys defined across backend and frontend.
 */
export const CONFIG_KEYS = {
  MAX_LOG_FILES: 'maxLogFiles',
  CUSTOM_KUBECONFIG_PATHS: 'customKubeconfigPaths',
  AUTO_CHECK_UPDATES: 'autoCheckUpdates',
  LAUNCH_ON_STARTUP: 'launchOnStartup',
  SHARE_TELEMETRY: 'shareTelemetry'
} as const

export type ConfigKey = (typeof CONFIG_KEYS)[keyof typeof CONFIG_KEYS]

export interface Configuration<T = unknown> {
  /** Unique identification key of config */
  key: string
  /** Display name on UI */
  name: string
  /** Description of what this config controls */
  description: string
  /** Data type used by UI to display the corresponding component */
  datatype: ConfigurationDataType
  /** Default value (optional) */
  defaultValue?: T
  /** Current effective value */
  value?: T
  /** Confidential flag (backend encodes/decodes, temporarily base64) */
  isConfidential: boolean
  /** Multiplicity / Cardinality: '0..1' | '0..*' | '1..*' | '1..1' */
  cardinality: ConfigurationCardinality
  /** Flag to control if a config is enabled / applicable */
  enable: boolean
  /** Timestamp when config was created */
  createdAt?: string
  /** Timestamp when config was last updated */
  lastUpdatedAt?: string
}

/**
 * Key-value mapping for direct overrides and backward compatibility.
 */
export type ConfigurationMap = Record<string, unknown>
