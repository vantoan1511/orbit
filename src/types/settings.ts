export type ConfigurationDataType = 'string' | 'number' | 'boolean'
export type ConfigurationCardinality = '0..1' | '0..*' | '1..*' | '1..1' | string

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
