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
  CLOSE_TO_TRAY: 'closeToTray',
  DEFAULT_TERMINAL_SHELL: 'defaultTerminalShell',
  KEYBOARD_SHORTCUTS: 'keyboardShortcuts',
  THEME_MODE: 'themeMode',
  ACCENT_COLOR: 'accentColor',
  UI_STYLE: 'uiStyle',
  SEMANTIC_COLORS: 'semanticColors',
  CLUSTER_CUSTOMIZATIONS: 'clusterCustomizations',
  NOTIFICATION_PREFERENCES: 'notificationPreferences',
  PROXY_SETTINGS: 'proxySettings'
} as const

export type ConfigKey = (typeof CONFIG_KEYS)[keyof typeof CONFIG_KEYS]

/**
 * Configurations that are managed in specialized settings tabs
 * (e.g. Preferences, Appearance) rather than the General form.
 */
export const SPECIALIZED_CONFIG_KEYS: readonly string[] = [
  CONFIG_KEYS.KEYBOARD_SHORTCUTS,
  CONFIG_KEYS.THEME_MODE,
  CONFIG_KEYS.ACCENT_COLOR,
  CONFIG_KEYS.UI_STYLE,
  CONFIG_KEYS.SEMANTIC_COLORS,
  CONFIG_KEYS.CLUSTER_CUSTOMIZATIONS,
  CONFIG_KEYS.NOTIFICATION_PREFERENCES,
  CONFIG_KEYS.PROXY_SETTINGS
] as const

export function isGeneralConfigKey(key: string): boolean {
  return !(SPECIALIZED_CONFIG_KEYS as readonly string[]).includes(key)
}

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
 * Granular notification preferences stored under CONFIG_KEYS.NOTIFICATION_PREFERENCES.
 */
export interface NotificationPreferences {
  enableToasts: boolean
  notifyOnSuccess: boolean
  notifyOnError: boolean
  notifyOnUpdates: boolean
}

export const DEFAULT_NOTIFICATION_PREFERENCES: NotificationPreferences = {
  enableToasts: true,
  notifyOnSuccess: true,
  notifyOnError: true,
  notifyOnUpdates: true
}

/**
 * Proxy configuration stored under CONFIG_KEYS.PROXY_SETTINGS.
 */
export interface ProxySettings {
  enabled: boolean
  httpProxy: string
  httpsProxy: string
  noProxy: string
}

export const DEFAULT_PROXY_SETTINGS: ProxySettings = {
  enabled: false,
  httpProxy: '',
  httpsProxy: '',
  noProxy: ''
}

/**
 * Key-value mapping for direct overrides and backward compatibility.
 */
export type ConfigurationMap = Record<string, unknown>

export type SettingsSubTab =
  'general' | 'clusters' | 'preferences' | 'appearance' | 'notifications' | 'proxy' | 'about'

export const VALID_SETTINGS_TABS: readonly SettingsSubTab[] = [
  'general',
  'clusters',
  'preferences',
  'appearance',
  'notifications',
  'proxy',
  'about'
] as const

export const ENABLED_SETTINGS_TABS: readonly SettingsSubTab[] = [
  'general',
  'clusters',
  'preferences',
  'appearance',
  'notifications',
  'proxy',
  'about'
] as const
