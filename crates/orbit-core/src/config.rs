use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub fn default_max_log_files() -> usize {
    10
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    /// Unique identification key of config (e.g. "maxLogFiles")
    pub key: String,
    /// Display name on UI
    pub name: String,
    /// Detailed description of what this configuration controls
    pub description: String,
    /// Datatype indicating the UI component ("string", "number", "boolean")
    pub datatype: String,
    /// Default system value (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<serde_json::Value>,
    /// Effective or user-specified value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    /// Whether this configuration is confidential (backend will encode/decode, temporarily base64)
    pub is_confidential: bool,
    /// Multiplicity / cardinality: "0..1", "0..*", "1..*", "1..1"
    pub cardinality: String,
    /// Whether the configuration is enabled/active in the system
    pub enable: bool,
    /// ISO timestamp when configuration definition was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// ISO timestamp when configuration was last updated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated_at: Option<String>,
}

pub const SYSTEM_CONFIG_CREATED_AT: &str = "2026-01-01T00:00:00Z";

impl Configuration {
    /// Returns the system-defined list of application configurations.
    pub fn system_definitions() -> Vec<Configuration> {
        let created = SYSTEM_CONFIG_CREATED_AT.to_string();
        vec![
            Configuration {
                key: "maxLogFiles".to_string(),
                name: "Log Retention Limit".to_string(),
                description: "Maximum number of application log files to retain before automatic rotation and cleanup.".to_string(),
                datatype: "number".to_string(),
                default_value: Some(serde_json::json!(default_max_log_files())),
                value: None,
                is_confidential: false,
                cardinality: "1..1".to_string(),
                enable: true,
                created_at: Some(created.clone()),
                last_updated_at: None,
            },
            Configuration {
                key: "customKubeconfigPaths".to_string(),
                name: "Custom Kubeconfig Paths".to_string(),
                description: "Additional file paths used to discover and manage cluster contexts.".to_string(),
                datatype: "string".to_string(),
                default_value: Some(serde_json::json!(Vec::<String>::new())),
                value: None,
                is_confidential: false,
                cardinality: "0..*".to_string(),
                enable: true,
                created_at: Some(created.clone()),
                last_updated_at: None,
            },
            Configuration {
                key: "autoCheckUpdates".to_string(),
                name: "Automatically Check for Updates".to_string(),
                description: "Periodically check if newer versions of Orbit are available.".to_string(),
                datatype: "boolean".to_string(),
                default_value: Some(serde_json::json!(true)),
                value: None,
                is_confidential: false,
                cardinality: "1..1".to_string(),
                enable: true,
                created_at: Some(created.clone()),
                last_updated_at: None,
            },
            Configuration {
                key: "launchOnStartup".to_string(),
                name: "Launch on Startup".to_string(),
                description: "Automatically start Orbit minimized to the system tray when logging into the system.".to_string(),
                datatype: "boolean".to_string(),
                default_value: Some(serde_json::json!(false)),
                value: None,
                is_confidential: false,
                cardinality: "1..1".to_string(),
                enable: true,
                created_at: Some(created.clone()),
                last_updated_at: None,
            },
            Configuration {
                key: "closeToTray".to_string(),
                name: "Close to System Tray".to_string(),
                description: "Keep Orbit running in the system tray when closing the window.".to_string(),
                datatype: "boolean".to_string(),
                default_value: Some(serde_json::json!(true)),
                value: None,
                is_confidential: false,
                cardinality: "1..1".to_string(),
                enable: true,
                created_at: Some(created.clone()),
                last_updated_at: None,
            },
            Configuration {
                key: "defaultTerminalShell".to_string(),
                name: "Default Terminal Shell".to_string(),
                description: "Path to the default shell executable to use for local terminal sessions (e.g. pwsh.exe, /bin/zsh).".to_string(),
                datatype: "string".to_string(),
                default_value: Some(serde_json::json!("")),
                value: None,
                is_confidential: false,
                cardinality: "0..1".to_string(),
                enable: true,
                created_at: Some(created.clone()),
                last_updated_at: None,
            },
            Configuration {
                key: "keyboardShortcuts".to_string(),
                name: "Keyboard Shortcuts".to_string(),
                description: "Custom keyboard shortcut keybindings for rapid workspace navigation and actions.".to_string(),
                datatype: "string".to_string(),
                default_value: Some(serde_json::json!({})),
                value: None,
                is_confidential: false,
                cardinality: "0..1".to_string(),
                enable: true,
                created_at: Some(created.clone()),
                last_updated_at: None,
            },
            Configuration {
                key: "themeMode".to_string(),
                name: "Theme Mode".to_string(),
                description: "Application theme appearance mode (dark, light, or system).".to_string(),
                datatype: "string".to_string(),
                default_value: Some(serde_json::json!("dark")),
                value: None,
                is_confidential: false,
                cardinality: "1..1".to_string(),
                enable: true,
                created_at: Some(created.clone()),
                last_updated_at: None,
            },
            Configuration {
                key: "accentColor".to_string(),
                name: "Accent Color".to_string(),
                description: "Primary accent color for interactive UI elements and focus states.".to_string(),
                datatype: "string".to_string(),
                default_value: Some(serde_json::json!("blue")),
                value: None,
                is_confidential: false,
                cardinality: "1..1".to_string(),
                enable: true,
                created_at: Some(created.clone()),
                last_updated_at: None,
            },
            Configuration {
                key: "uiStyle".to_string(),
                name: "UI Style".to_string(),
                description: "Visual design aesthetic style (flat, glass, aero, liquidglass, ios, paper).".to_string(),
                datatype: "string".to_string(),
                default_value: Some(serde_json::json!("flat")),
                value: None,
                is_confidential: false,
                cardinality: "1..1".to_string(),
                enable: true,
                created_at: Some(created.clone()),
                last_updated_at: None,
            },
            Configuration {
                key: "semanticColors".to_string(),
                name: "Semantic Colors".to_string(),
                description: "Custom semantic status colors for Success, Warning, Danger, and Info.".to_string(),
                datatype: "string".to_string(),
                default_value: Some(serde_json::json!({
                    "preset": "default",
                    "success": "emerald",
                    "warning": "amber",
                    "danger": "rose",
                    "info": "blue"
                })),
                value: None,
                is_confidential: false,
                cardinality: "0..1".to_string(),
                enable: true,
                created_at: Some(created.clone()),
                last_updated_at: None,
            },
            Configuration {
                key: "clusterCustomizations".to_string(),
                name: "Cluster Customizations".to_string(),
                description: "User-defined custom cluster metadata such as aliases, labels, tags, and colors.".to_string(),
                datatype: "string".to_string(),
                default_value: Some(serde_json::json!({})),
                value: None,
                is_confidential: false,
                cardinality: "0..1".to_string(),
                enable: true,
                created_at: Some(created.clone()),
                last_updated_at: None,
            },
            Configuration {
                key: "notificationPreferences".to_string(),
                name: "Notification Preferences".to_string(),
                description: "Granular controls for UI toast popups and operational notifications.".to_string(),
                datatype: "string".to_string(),
                default_value: Some(serde_json::json!({
                    "enableToasts": true,
                    "notifyOnSuccess": true,
                    "notifyOnError": true,
                    "notifyOnUpdates": true
                })),
                value: None,
                is_confidential: false,
                cardinality: "0..1".to_string(),
                enable: true,
                created_at: Some(created.clone()),
                last_updated_at: None,
            },
            Configuration {
                key: "proxySettings".to_string(),
                name: "HTTP and HTTPS Proxy".to_string(),
                description: "Proxy configuration for outbound Kubernetes API requests and cluster connectivity.".to_string(),
                datatype: "string".to_string(),
                default_value: Some(serde_json::json!({
                    "enabled": false,
                    "httpProxy": "",
                    "httpsProxy": "",
                    "noProxy": ""
                })),
                value: None,
                is_confidential: false,
                cardinality: "0..1".to_string(),
                enable: true,
                created_at: Some(created.clone()),
                last_updated_at: None,
            },
            Configuration {
                key: "selectedNamespaces".to_string(),
                name: "Selected Namespaces".to_string(),
                description: "Per-cluster active namespace filter selections.".to_string(),
                datatype: "string".to_string(),
                default_value: Some(serde_json::json!({})),
                value: None,
                is_confidential: false,
                cardinality: "0..1".to_string(),
                enable: true,
                created_at: Some(created.clone()),
                last_updated_at: None,
            },
            Configuration {
                key: "workspaceTabs".to_string(),
                name: "Workspace Tabs".to_string(),
                description: "Per-cluster open workspace tabs and active tab state.".to_string(),
                datatype: "string".to_string(),
                default_value: Some(serde_json::json!({})),
                value: None,
                is_confidential: false,
                cardinality: "0..1".to_string(),
                enable: true,
                created_at: Some(created.clone()),
                last_updated_at: None,
            },
            Configuration {
                key: "tablePreferences".to_string(),
                name: "Table Preferences".to_string(),
                description: "Per-table column, sort, and page size settings plus per-cluster table filters.".to_string(),
                datatype: "string".to_string(),
                default_value: Some(serde_json::json!({})),
                value: None,
                is_confidential: false,
                cardinality: "0..1".to_string(),
                enable: true,
                created_at: Some(created.clone()),
                last_updated_at: None,
            },
            Configuration {
                key: "enableExperienceFeatures".to_string(),
                name: "Experience Features".to_string(),
                description: "Enable experimental preview features such as the native GPUI desktop shell.".to_string(),
                datatype: "boolean".to_string(),
                default_value: Some(serde_json::json!(false)),
                value: None,
                is_confidential: false,
                cardinality: "1..1".to_string(),
                enable: true,
                created_at: Some(created),
                last_updated_at: None,
            },
        ]
    }
}

pub type SelectedNamespacesMap = HashMap<String, Vec<String>>;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PersistedClusterTabs {
    pub open_tabs: Vec<String>,
    pub active_tab: Option<String>,
}

pub type WorkspaceTabsMap = HashMap<String, PersistedClusterTabs>;

pub const ALLOWED_TABLE_PAGE_SIZES: [usize; 4] = [25, 50, 100, 200];
pub const DEFAULT_TABLE_PAGE_SIZE: usize = 25;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PersistedSortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PersistedTableConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hidden_columns: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_direction: Option<PersistedSortDirection>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PersistedClusterTableFilters {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub search_query: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extra_filters: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PersistedTablePreferences {
    /// Per-table layout/config state (keyed by table_id, e.g. "pods", "deployments").
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tables: HashMap<String, PersistedTableConfig>,
    /// Per-cluster, per-table active filter state (cluster_id -> table_id -> filters).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub clusters: HashMap<String, HashMap<String, PersistedClusterTableFilters>>,
}

/// Persistent user configuration storing strictly key-value pairs in `~/.orbit/config.json`.
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(transparent)]
pub struct OrbitConfig {
    pub values: HashMap<String, serde_json::Value>,
}

impl OrbitConfig {
    /// Determines whether Orbit is running in development mode.
    /// Checked via `ORBIT_DEV` environment variable (if "1", "true", or "dev" -> dev mode; if "0" or "false" -> prod mode)
    /// falling back to debug assertion compilation flags `cfg!(debug_assertions)`.
    pub fn is_dev_mode() -> bool {
        if let Ok(val) = std::env::var("ORBIT_DEV") {
            let normalized = val.trim().to_lowercase();
            if normalized == "1" || normalized == "true" || normalized == "dev" {
                return true;
            }
            if normalized == "0" || normalized == "false" || normalized == "prod" {
                return false;
            }
        }
        cfg!(debug_assertions)
    }

    /// Returns the directory name for Orbit's user configuration (`.orbit-dev` vs `.orbit`).
    pub fn config_dir_name() -> &'static str {
        if Self::is_dev_mode() {
            ".orbit-dev"
        } else {
            ".orbit"
        }
    }

    /// Returns the path to the Orbit configuration directory (`~/.orbit` or `~/.orbit-dev`).
    pub fn config_dir() -> Option<PathBuf> {
        if let Some(custom) = std::env::var_os("ORBIT_CONFIG_DIR") {
            return Some(PathBuf::from(custom));
        }
        let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
        let mut path = PathBuf::from(home);
        path.push(Self::config_dir_name());
        Some(path)
    }

    /// Returns the path to the Orbit logs directory (`~/.orbit/logs`).
    pub fn logs_dir() -> Option<PathBuf> {
        let mut path = Self::config_dir()?;
        path.push("logs");
        Some(path)
    }

    /// Returns the path to the Orbit configuration file (`~/.orbit/config.json`).
    pub fn file_path() -> Option<PathBuf> {
        let mut path = Self::config_dir()?;
        path.push("config.json");
        Some(path)
    }

    /// Returns whether experimental experience features (e.g., GPUI preview shell) are enabled.
    pub fn enable_experience_features(&self) -> bool {
        self.values
            .get("enableExperienceFeatures")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// Load the configuration from disk, or return default if it does not exist or fails to parse.
    pub fn load() -> Self {
        let Some(path) = Self::file_path() else {
            return Self::default();
        };

        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<HashMap<String, serde_json::Value>>(&content) {
                Ok(raw_map) => {
                    let mut normalized = HashMap::new();
                    for (k, v) in raw_map {
                        // Normalize legacy aliases
                        match k.as_str() {
                            "custom_kubeconfig_paths" => {
                                normalized.insert("customKubeconfigPaths".to_string(), v);
                            }
                            "max_log_files" => {
                                normalized.insert("maxLogFiles".to_string(), v);
                            }
                            _ => {
                                normalized.insert(k, v);
                            }
                        }
                    }
                    Self { values: normalized }
                }
                Err(err) => {
                    tracing::warn!(path = ?path, error = %err, "Failed to parse Orbit configuration file");
                    Self::default()
                }
            },
            Err(_) => Self::default(),
        }
    }

    /// Save the configuration to disk at `~/.orbit/config.json`.
    pub fn save(&self) -> Result<(), String> {
        let config_dir = Self::config_dir()
            .ok_or_else(|| "Could not determine user home directory for config storage".to_string())?;

        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)
                .map_err(|e| format!("Failed to create config directory {:?}: {}", config_dir, e))?;
        }

        let file_path = Self::file_path()
            .ok_or_else(|| "Could not determine configuration file path".to_string())?;

        let content = serde_json::to_string_pretty(&self.values)
            .map_err(|e| format!("Failed to serialize Orbit configuration: {}", e))?;

        std::fs::write(&file_path, content)
            .map_err(|e| format!("Failed to write configuration file {:?}: {}", file_path, e))?;

        Ok(())
    }

    /// Returns the resolved log retention limit.
    pub fn max_log_files(&self) -> usize {
        self.values
            .get("maxLogFiles")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or_else(default_max_log_files)
    }

    /// Returns the custom kubeconfig paths.
    pub fn custom_kubeconfig_paths(&self) -> Vec<String> {
        self.values
            .get("customKubeconfigPaths")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns the configured default terminal shell path, if set and non-empty.
    pub fn default_terminal_shell(&self) -> Option<String> {
        self.values
            .get("defaultTerminalShell")
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    }

    /// Returns whether launch on startup is enabled. Defaults to false.
    pub fn launch_on_startup(&self) -> bool {
        self.values
            .get("launchOnStartup")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// Returns whether close to system tray is enabled. Defaults to true.
    pub fn close_to_tray(&self) -> bool {
        self.values
            .get("closeToTray")
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
    }

    /// Returns whether automatically checking for updates is enabled. Defaults to true.
    pub fn auto_check_updates(&self) -> bool {
        self.values
            .get("autoCheckUpdates")
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
    }

    /// Returns notification preferences configuration.
    pub fn notification_preferences(&self) -> serde_json::Value {
        self.values
            .get("notificationPreferences")
            .cloned()
            .unwrap_or_else(|| {
                serde_json::json!({
                    "enableToasts": true,
                    "notifyOnSuccess": true,
                    "notifyOnError": true,
                    "notifyOnUpdates": true
                })
            })
    }

    /// Returns HTTP and HTTPS proxy settings.
    pub fn proxy_settings(&self) -> serde_json::Value {
        self.values
            .get("proxySettings")
            .cloned()
            .unwrap_or_else(|| {
                serde_json::json!({
                    "enabled": false,
                    "httpProxy": "",
                    "httpsProxy": "",
                    "noProxy": ""
                })
            })
    }

    /// Returns custom keyboard shortcuts keybindings map.
    pub fn keyboard_shortcuts(&self) -> HashMap<String, String> {
        self.values
            .get("keyboardShortcuts")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    /// Returns configured theme mode ("dark", "light", or "system"). Defaults to "dark".
    pub fn theme_mode(&self) -> String {
        self.values
            .get("themeMode")
            .and_then(|v| v.as_str())
            .unwrap_or("dark")
            .to_string()
    }

    /// Returns configured accent color name. Defaults to "blue".
    pub fn accent_color(&self) -> String {
        self.values
            .get("accentColor")
            .and_then(|v| v.as_str())
            .unwrap_or("blue")
            .to_string()
    }

    /// Returns configured UI style name. Defaults to "flat".
    pub fn ui_style(&self) -> String {
        self.values
            .get("uiStyle")
            .and_then(|v| v.as_str())
            .unwrap_or("flat")
            .to_string()
    }

    fn set_validated_string_or_default(
        &mut self,
        key: &str,
        value: &str,
        valid_values: &[&str],
        default: &str,
    ) {
        let normalized = value.trim().to_ascii_lowercase();
        let resolved = if valid_values.contains(&normalized.as_str()) {
            normalized
        } else {
            default.to_string()
        };

        if resolved == default {
            self.values.remove(key);
        } else {
            self.values
                .insert(key.to_string(), serde_json::Value::String(resolved));
        }
    }

    /// Updates `themeMode` ("dark", "light", or "system"), omitting the default "dark".
    pub fn update_theme_mode(&mut self, mode: &str) {
        self.set_validated_string_or_default("themeMode", mode, &["dark", "light", "system"], "dark");
    }

    /// Updates `uiStyle` ("flat", "glass", "aero", "liquidglass", "ios", or "paper"), omitting the default "flat".
    pub fn update_ui_style(&mut self, style: &str) {
        self.set_validated_string_or_default(
            "uiStyle",
            style,
            &["flat", "glass", "aero", "liquidglass", "ios", "paper"],
            "flat",
        );
    }

    /// Updates `accentColor` ("blue", "emerald", "violet", "cyan", "amber", "rose", "slate"), omitting the default "blue".
    pub fn update_accent_color(&mut self, accent: &str) {
        self.set_validated_string_or_default(
            "accentColor",
            accent,
            &["blue", "emerald", "violet", "cyan", "amber", "rose", "slate"],
            "blue",
        );
    }

    /// Updates `enableExperienceFeatures`, omitting the default `false`.
    pub fn update_enable_experience_features(&mut self, enabled: bool) {
        if enabled {
            self.values
                .insert("enableExperienceFeatures".to_string(), serde_json::Value::Bool(true));
        } else {
            self.values.remove("enableExperienceFeatures");
        }
    }

    /// Resets `themeMode`, `uiStyle`, and `accentColor` back to system defaults.
    pub fn reset_appearance_preferences(&mut self) {
        self.values.remove("themeMode");
        self.values.remove("uiStyle");
        self.values.remove("accentColor");
    }

    /// Returns configured semantic colors. Defaults to default preset.
    pub fn semantic_colors(&self) -> serde_json::Value {
        self.values
            .get("semanticColors")
            .cloned()
            .unwrap_or_else(|| {
                serde_json::json!({
                    "preset": "default",
                    "success": "emerald",
                    "warning": "amber",
                    "danger": "rose",
                    "info": "blue"
                })
            })
    }

    /// Returns configured cluster customizations map.
    pub fn cluster_customizations(&self) -> crate::kubernetes::models::ClusterCustomizationMap {
        self.values
            .get("clusterCustomizations")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    /// Updates or removes cluster customization for a specific cluster ID.
    pub fn set_cluster_customization(
        &mut self,
        id: &str,
        customization: crate::kubernetes::models::ClusterCustomization,
    ) {
        let mut map = self.cluster_customizations();
        if customization.alias.as_deref().unwrap_or("").trim().is_empty()
            && customization.tags.is_empty()
            && customization.color.as_deref().unwrap_or("").trim().is_empty()
            && customization.description.as_deref().unwrap_or("").trim().is_empty()
        {
            map.remove(id);
        } else {
            let sanitized = crate::kubernetes::models::ClusterCustomization {
                alias: customization.alias.filter(|s| !s.trim().is_empty()),
                tags: customization.tags.into_iter().filter(|s| !s.trim().is_empty()).collect(),
                color: customization.color.filter(|s| !s.trim().is_empty()),
                description: customization.description.filter(|s| !s.trim().is_empty()),
            };
            map.insert(id.to_string(), sanitized);
        }
        if let Ok(v) = serde_json::to_value(map) {
            self.values.insert("clusterCustomizations".to_string(), v);
        }
    }

    /// Returns the per-cluster selected namespaces map.
    pub fn selected_namespaces_map(&self) -> SelectedNamespacesMap {
        self.values
            .get("selectedNamespaces")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    /// Returns the persisted namespace selection for a specific cluster ID.
    /// An empty vector represents "All Namespaces".
    pub fn selected_namespaces_for_cluster(&self, cluster_id: &str) -> Vec<String> {
        self.selected_namespaces_map()
            .get(cluster_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Updates or clears the persisted namespace selection for a specific cluster ID in memory.
    pub fn update_selected_namespaces_for_cluster(
        &mut self,
        cluster_id: &str,
        namespaces: Vec<String>,
    ) {
        let mut map = self.selected_namespaces_map();
        let mut seen = std::collections::HashSet::new();
        let cleaned: Vec<String> = namespaces
            .into_iter()
            .map(|ns| ns.trim().to_string())
            .filter(|ns| !ns.is_empty() && seen.insert(ns.clone()))
            .collect();

        if cleaned.is_empty() {
            map.remove(cluster_id);
        } else {
            map.insert(cluster_id.to_string(), cleaned);
        }

        if map.is_empty() {
            self.values.remove("selectedNamespaces");
        } else if let Ok(v) = serde_json::to_value(map) {
            self.values.insert("selectedNamespaces".to_string(), v);
        }
    }

    /// Updates and saves the persisted namespace selection for a specific cluster ID.
    pub fn set_selected_namespaces_for_cluster(
        &mut self,
        cluster_id: &str,
        namespaces: Vec<String>,
    ) -> Result<(), String> {
        self.update_selected_namespaces_for_cluster(cluster_id, namespaces);
        self.save()
    }

    /// Returns the per-cluster workspace tabs map.
    pub fn workspace_tabs_map(&self) -> WorkspaceTabsMap {
        self.values
            .get("workspaceTabs")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    /// Returns the persisted workspace tabs for a specific cluster ID, if visited.
    pub fn workspace_tabs_for_cluster(&self, cluster_id: &str) -> Option<PersistedClusterTabs> {
        self.workspace_tabs_map().get(cluster_id).cloned()
    }

    /// Updates the persisted workspace tabs for a specific cluster ID in memory.
    /// Preserves explicit empty `open_tabs` (`active_tab: None`) so a visited cluster
    /// with all tabs closed does not re-open a default tab on reconnect.
    pub fn update_workspace_tabs_for_cluster(
        &mut self,
        cluster_id: &str,
        open_tabs: Vec<String>,
        active_tab: Option<String>,
    ) {
        let mut map = self.workspace_tabs_map();
        let mut seen = std::collections::HashSet::new();
        let cleaned_tabs: Vec<String> = open_tabs
            .into_iter()
            .map(|id| id.trim().to_string())
            .filter(|id| !id.is_empty() && seen.insert(id.clone()))
            .collect();

        let resolved_active = if cleaned_tabs.is_empty() {
            None
        } else {
            active_tab
                .map(|id| id.trim().to_string())
                .filter(|id| !id.is_empty() && cleaned_tabs.iter().any(|t| t == id))
                .or_else(|| cleaned_tabs.first().cloned())
        };

        map.insert(
            cluster_id.to_string(),
            PersistedClusterTabs {
                open_tabs: cleaned_tabs,
                active_tab: resolved_active,
            },
        );

        if let Ok(v) = serde_json::to_value(map) {
            self.values.insert("workspaceTabs".to_string(), v);
        }
    }

    /// Updates and saves the persisted workspace tabs for a specific cluster ID.
    pub fn set_workspace_tabs_for_cluster(
        &mut self,
        cluster_id: &str,
        open_tabs: Vec<String>,
        active_tab: Option<String>,
    ) -> Result<(), String> {
        self.update_workspace_tabs_for_cluster(cluster_id, open_tabs, active_tab);
        self.save()
    }

    /// Returns the persisted DataTable preferences (per-table config + per-cluster filters).
    pub fn table_preferences(&self) -> PersistedTablePreferences {
        self.values
            .get("tablePreferences")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    fn write_table_preferences(&mut self, prefs: PersistedTablePreferences) {
        if prefs.tables.is_empty() && prefs.clusters.is_empty() {
            self.values.remove("tablePreferences");
        } else if let Ok(v) = serde_json::to_value(prefs) {
            self.values.insert("tablePreferences".to_string(), v);
        }
    }

    /// Updates the persisted layout/sort/page-size config for a specific table ID in memory.
    pub fn update_table_config(&mut self, table_id: &str, config: PersistedTableConfig) {
        let table_key = table_id.trim();
        if table_key.is_empty() {
            return;
        }

        let mut prefs = self.table_preferences();

        let page_size = config
            .page_size
            .filter(|size| ALLOWED_TABLE_PAGE_SIZES.contains(size));

        let mut seen = std::collections::HashSet::new();
        let mut hidden_columns: Vec<String> = config
            .hidden_columns
            .into_iter()
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty() && seen.insert(c.clone()))
            .collect();
        hidden_columns.sort();

        let sort_column = config
            .sort_column
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty());
        let sort_direction = if sort_column.is_some() {
            config.sort_direction
        } else {
            None
        };

        let normalized = PersistedTableConfig {
            page_size,
            hidden_columns,
            sort_column,
            sort_direction,
        };

        if normalized == PersistedTableConfig::default() {
            prefs.tables.remove(table_key);
        } else {
            prefs.tables.insert(table_key.to_string(), normalized);
        }

        self.write_table_preferences(prefs);
    }

    /// Updates the persisted filter state for a specific `(cluster_id, table_id)` in memory.
    pub fn update_cluster_table_filters(
        &mut self,
        cluster_id: &str,
        table_id: &str,
        filters: PersistedClusterTableFilters,
    ) {
        let cluster_key = cluster_id.trim();
        let table_key = table_id.trim();
        if cluster_key.is_empty() || table_key.is_empty() {
            return;
        }

        let mut prefs = self.table_preferences();

        let search_query = filters.search_query.trim().to_string();
        let extra_filters: HashMap<String, String> = filters
            .extra_filters
            .into_iter()
            .filter_map(|(k, v)| {
                let k = k.trim().to_string();
                let v = v.trim().to_string();
                if k.is_empty() || v.is_empty() {
                    None
                } else {
                    Some((k, v))
                }
            })
            .collect();

        let normalized = PersistedClusterTableFilters {
            search_query,
            extra_filters,
        };

        if normalized == PersistedClusterTableFilters::default() {
            if let Some(cluster_tables) = prefs.clusters.get_mut(cluster_key) {
                cluster_tables.remove(table_key);
                if cluster_tables.is_empty() {
                    prefs.clusters.remove(cluster_key);
                }
            }
        } else {
            prefs
                .clusters
                .entry(cluster_key.to_string())
                .or_default()
                .insert(table_key.to_string(), normalized);
        }

        self.write_table_preferences(prefs);
    }

    /// Generates the unified `Configuration` model list sent to the frontend.
    pub fn get_configurations(&self) -> Vec<Configuration> {
        let mut defs = Configuration::system_definitions();

        for config in &mut defs {
            let raw_val = self.values.get(&config.key).cloned().or_else(|| config.default_value.clone());
            if let Some(val) = raw_val {
                if config.is_confidential {
                    // Base64 encode string values when confidential
                    if let Some(s) = val.as_str() {
                        config.value = Some(serde_json::Value::String(BASE64_STANDARD.encode(s.as_bytes())));
                    } else {
                        let serialized = val.to_string();
                        config.value = Some(serde_json::Value::String(BASE64_STANDARD.encode(serialized.as_bytes())));
                    }
                } else {
                    config.value = Some(val);
                }
            }
        }
        defs
    }

    /// Updates configuration values from incoming unified `Configuration` models.
    pub fn update_from_configurations(&mut self, updated_configs: &[Configuration]) {
        let defs = Configuration::system_definitions();
        let def_map: HashMap<&str, &Configuration> = defs.iter().map(|d| (d.key.as_str(), d)).collect();

        for incoming in updated_configs {
            let Some(def) = def_map.get(incoming.key.as_str()) else {
                tracing::trace!(key = %incoming.key, "Ignoring unknown configuration key");
                continue;
            };

            let Some(mut val) = incoming.value.clone() else {
                // If value was omitted or null in payload, remove to revert to default
                self.values.remove(&incoming.key);
                continue;
            };

            if val.is_null() || def.default_value.as_ref() == Some(&val) {
                self.values.remove(&incoming.key);
                continue;
            }

            // If confidential, decode base64 before storing locally
            if def.is_confidential {
                val = val
                    .as_str()
                    .and_then(|s| BASE64_STANDARD.decode(s).ok())
                    .and_then(|b| String::from_utf8(b).ok())
                    .map(serde_json::Value::String)
                    .unwrap_or(val);
            }
            self.values.insert(incoming.key.clone(), val);
        }
    }

    /// Normalizes a file path string for cross-platform comparisons (canonicalizing slashes and casing).
    pub fn normalize_path(path_str: &str) -> String {
        if let Ok(canonical) = std::fs::canonicalize(path_str) {
            let s = canonical.to_string_lossy().to_string();
            let clean = if let Some(stripped) = s.strip_prefix(r"\\?\") {
                stripped.to_string()
            } else {
                s
            };
            clean.replace('\\', "/")
        } else {
            path_str.replace('\\', "/")
        }
    }

    /// Add a custom kubeconfig file path to the configuration and save it.
    /// If the path is already present, it will not be duplicated.
    pub fn add_kubeconfig_path(&mut self, path: &str) -> Result<(), String> {
        let norm_path = Self::normalize_path(path);
        let mut paths = self.custom_kubeconfig_paths();
        if !paths.iter().any(|p| Self::normalize_path(p).eq_ignore_ascii_case(&norm_path)) {
            paths.push(norm_path);
            self.values.insert("customKubeconfigPaths".to_string(), serde_json::json!(paths));
            self.save()?;
        }
        Ok(())
    }
}

#[cfg(any(test, feature = "test-utils"))]
static TEST_CONFIG_DIR_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(any(test, feature = "test-utils"))]
pub struct TestConfigDirGuard {
    pub temp_dir: PathBuf,
    original_env: Option<std::ffi::OsString>,
    _lock: std::sync::MutexGuard<'static, ()>,
}

#[cfg(any(test, feature = "test-utils"))]
impl TestConfigDirGuard {
    pub fn new(test_name: &str) -> Self {
        let lock = TEST_CONFIG_DIR_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir = std::env::temp_dir().join(format!("orbit_{}_{}", test_name, uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let original_env = std::env::var_os("ORBIT_CONFIG_DIR");
        unsafe {
            std::env::set_var("ORBIT_CONFIG_DIR", &temp_dir);
        }
        Self {
            temp_dir,
            original_env,
            _lock: lock,
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl Drop for TestConfigDirGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.original_env {
                Some(val) => std::env::set_var("ORBIT_CONFIG_DIR", val),
                None => std::env::remove_var("ORBIT_CONFIG_DIR"),
            }
        }
        let _ = std::fs::remove_dir_all(&self.temp_dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_orbit_config_default_and_getters() {
        let config = OrbitConfig::default();
        assert!(config.custom_kubeconfig_paths().is_empty());
        assert_eq!(config.max_log_files(), 10);
    }

    #[test]
    fn test_enable_experience_features_in_system_definitions() {
        let defs = Configuration::system_definitions();
        let entry = defs
            .iter()
            .find(|c| c.key == "enableExperienceFeatures")
            .expect("enableExperienceFeatures should be defined in system_definitions");
        assert_eq!(entry.datatype, "boolean");
        assert_eq!(entry.default_value, Some(serde_json::json!(false)));
        assert!(entry.enable);
        assert_eq!(entry.cardinality, "1..1");
        assert!(!OrbitConfig::default().enable_experience_features());
    }

    #[test]
    fn test_orbit_config_serde_key_value_store() {
        let json = r#"{"custom_kubeconfig_paths": ["/path/to/kubeconfig.yaml"], "max_log_files": 5}"#;
        let mut raw_map: HashMap<String, serde_json::Value> = serde_json::from_str(json).unwrap();
        // Test normalization logic
        let mut normalized = HashMap::new();
        for (k, v) in raw_map.drain() {
            if k == "custom_kubeconfig_paths" {
                normalized.insert("customKubeconfigPaths".to_string(), v);
            } else if k == "max_log_files" {
                normalized.insert("maxLogFiles".to_string(), v);
            } else {
                normalized.insert(k, v);
            }
        }
        let config = OrbitConfig { values: normalized };
        assert_eq!(config.custom_kubeconfig_paths().len(), 1);
        assert_eq!(config.custom_kubeconfig_paths()[0], "/path/to/kubeconfig.yaml");
        assert_eq!(config.max_log_files(), 5);

        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("/path/to/kubeconfig.yaml"));
        assert!(serialized.contains("\"maxLogFiles\":5"));
    }

    #[test]
    fn test_unified_configuration_model_generation() {
        let mut config = OrbitConfig::default();
        config.values.insert("maxLogFiles".to_string(), serde_json::json!(25));

        let configs = config.get_configurations();
        assert!(!configs.is_empty());

        let max_log_cfg = configs.iter().find(|c| c.key == "maxLogFiles").unwrap();
        assert_eq!(max_log_cfg.name, "Log Retention Limit");
        assert_eq!(max_log_cfg.datatype, "number");
        assert_eq!(max_log_cfg.value, Some(serde_json::json!(25)));
        assert_eq!(max_log_cfg.default_value, Some(serde_json::json!(10)));
        assert_eq!(max_log_cfg.cardinality, "1..1");
        assert!(max_log_cfg.enable);
    }

    #[test]
    fn test_confidential_encoding_and_decoding() {
        let mut test_def = Configuration {
            key: "apiKey".to_string(),
            name: "API Key".to_string(),
            description: "Confidential key".to_string(),
            datatype: "string".to_string(),
            default_value: None,
            value: Some(serde_json::json!("super-secret-token")),
            is_confidential: true,
            cardinality: "0..1".to_string(),
            enable: true,
            created_at: None,
            last_updated_at: None,
        };

        // Test base64 encode
        let raw_secret = "my-secret-value";
        let encoded = BASE64_STANDARD.encode(raw_secret.as_bytes());
        test_def.value = Some(serde_json::json!(encoded));

        let decoded_bytes = BASE64_STANDARD.decode(test_def.value.as_ref().unwrap().as_str().unwrap()).unwrap();
        let decoded_str = String::from_utf8(decoded_bytes).unwrap();
        assert_eq!(decoded_str, raw_secret);
    }

    #[test]
    fn test_update_from_configurations() {
        let mut config = OrbitConfig::default();
        let updated = vec![
            Configuration {
                key: "maxLogFiles".to_string(),
                name: "Log Retention Limit".to_string(),
                description: "Test".to_string(),
                datatype: "number".to_string(),
                default_value: Some(serde_json::json!(10)),
                value: Some(serde_json::json!(50)),
                is_confidential: false,
                cardinality: "1..1".to_string(),
                enable: true,
                created_at: None,
                last_updated_at: None,
            }
        ];

        config.update_from_configurations(&updated);
        assert_eq!(config.max_log_files(), 50);

        // Test resetting to default via matching default value (e.g. 10)
        let reset_with_default = vec![
            Configuration {
                key: "maxLogFiles".to_string(),
                name: "Log Retention Limit".to_string(),
                description: "Test".to_string(),
                datatype: "number".to_string(),
                default_value: Some(serde_json::json!(10)),
                value: Some(serde_json::json!(10)),
                is_confidential: false,
                cardinality: "1..1".to_string(),
                enable: true,
                created_at: None,
                last_updated_at: None,
            }
        ];
        config.update_from_configurations(&reset_with_default);
        assert_eq!(config.max_log_files(), 10);
        assert!(!config.values.contains_key("maxLogFiles"));

        // Re-set and test resetting to default via None
        config.values.insert("maxLogFiles".to_string(), serde_json::json!(50));
        let reset_with_none = vec![
            Configuration {
                key: "maxLogFiles".to_string(),
                name: "Log Retention Limit".to_string(),
                description: "Test".to_string(),
                datatype: "number".to_string(),
                default_value: Some(serde_json::json!(10)),
                value: None,
                is_confidential: false,
                cardinality: "1..1".to_string(),
                enable: true,
                created_at: None,
                last_updated_at: None,
            }
        ];
        config.update_from_configurations(&reset_with_none);
        assert_eq!(config.max_log_files(), 10);
        assert!(!config.values.contains_key("maxLogFiles"));
    }

    #[test]
    fn test_default_terminal_shell_config() {
        let mut config = OrbitConfig::default();
        assert_eq!(config.default_terminal_shell(), None);

        config.values.insert("defaultTerminalShell".to_string(), serde_json::json!("pwsh.exe"));
        assert_eq!(config.default_terminal_shell(), Some("pwsh.exe".to_string()));

        // Empty string should be treated as None
        config.values.insert("defaultTerminalShell".to_string(), serde_json::json!("   "));
        assert_eq!(config.default_terminal_shell(), None);
    }

    #[test]
    fn test_launch_on_startup_is_enabled_in_system_definitions() {
        let defs = Configuration::system_definitions();
        let launch_cfg = defs.iter().find(|c| c.key == "launchOnStartup").expect("launchOnStartup must exist");
        assert!(launch_cfg.enable, "launchOnStartup must be enabled");
        assert_eq!(launch_cfg.datatype, "boolean");
        assert_eq!(launch_cfg.default_value, Some(serde_json::json!(false)));
        assert!(launch_cfg.description.contains("minimized to the system tray"));
    }

    #[test]
    fn test_launch_on_startup_default_and_custom() {
        let mut config = OrbitConfig::default();
        assert!(!config.launch_on_startup());

        config.values.insert("launchOnStartup".to_string(), serde_json::json!(true));
        assert!(config.launch_on_startup());

        config.values.insert("launchOnStartup".to_string(), serde_json::json!(false));
        assert!(!config.launch_on_startup());
    }

    #[test]
    fn test_close_to_tray_in_system_definitions() {
        let defs = Configuration::system_definitions();
        let close_cfg = defs.iter().find(|c| c.key == "closeToTray").expect("closeToTray must exist");
        assert!(close_cfg.enable, "closeToTray must be enabled");
        assert_eq!(close_cfg.datatype, "boolean");
        assert_eq!(close_cfg.default_value, Some(serde_json::json!(true)));
        assert!(close_cfg.description.contains("system tray"));
    }

    #[test]
    fn test_close_to_tray_default_and_custom() {
        let mut config = OrbitConfig::default();
        assert!(config.close_to_tray());

        config.values.insert("closeToTray".to_string(), serde_json::json!(false));
        assert!(!config.close_to_tray());

        config.values.insert("closeToTray".to_string(), serde_json::json!(true));
        assert!(config.close_to_tray());
    }

    #[test]
    fn test_keyboard_shortcuts_in_system_definitions() {
        let defs = Configuration::system_definitions();
        let cfg = defs.iter().find(|c| c.key == "keyboardShortcuts").expect("keyboardShortcuts must exist");
        assert!(cfg.enable, "keyboardShortcuts must be enabled");
        assert_eq!(cfg.datatype, "string");
        assert_eq!(cfg.default_value, Some(serde_json::json!({})));
    }

    #[test]
    fn test_keyboard_shortcuts_config() {
        let mut config = OrbitConfig::default();
        assert!(config.keyboard_shortcuts().is_empty());

        let mut map = std::collections::HashMap::new();
        map.insert("quickOpen".to_string(), "Ctrl+K".to_string());
        config.values.insert("keyboardShortcuts".to_string(), serde_json::json!(map));

        let shortcuts = config.keyboard_shortcuts();
        assert_eq!(shortcuts.get("quickOpen"), Some(&"Ctrl+K".to_string()));
    }

    #[test]
    fn test_appearance_configs_in_system_definitions() {
        let defs = Configuration::system_definitions();
        let theme_cfg = defs.iter().find(|c| c.key == "themeMode").expect("themeMode must exist");
        assert!(theme_cfg.enable);
        assert_eq!(theme_cfg.default_value, Some(serde_json::json!("dark")));

        let accent_cfg = defs.iter().find(|c| c.key == "accentColor").expect("accentColor must exist");
        assert!(accent_cfg.enable);
        assert_eq!(accent_cfg.default_value, Some(serde_json::json!("blue")));

        let style_cfg = defs.iter().find(|c| c.key == "uiStyle").expect("uiStyle must exist");
        assert!(style_cfg.enable);
        assert_eq!(style_cfg.default_value, Some(serde_json::json!("flat")));

        let semantic_cfg = defs.iter().find(|c| c.key == "semanticColors").expect("semanticColors must exist");
        assert!(semantic_cfg.enable);
        assert_eq!(
            semantic_cfg.default_value,
            Some(serde_json::json!({
                "preset": "default",
                "success": "emerald",
                "warning": "amber",
                "danger": "rose",
                "info": "blue"
            }))
        );
    }

    #[test]
    fn test_appearance_configs_defaults_and_custom() {
        let mut config = OrbitConfig::default();
        assert_eq!(config.theme_mode(), "dark");
        assert_eq!(config.accent_color(), "blue");
        assert_eq!(config.ui_style(), "flat");
        assert_eq!(
            config.semantic_colors(),
            serde_json::json!({
                "preset": "default",
                "success": "emerald",
                "warning": "amber",
                "danger": "rose",
                "info": "blue"
            })
        );

        config.values.insert("themeMode".to_string(), serde_json::json!("light"));
        config.values.insert("accentColor".to_string(), serde_json::json!("emerald"));
        config.values.insert("uiStyle".to_string(), serde_json::json!("glass"));
        config.values.insert(
            "semanticColors".to_string(),
            serde_json::json!({
                "preset": "colorblind",
                "success": "teal",
                "warning": "amber",
                "danger": "crimson",
                "info": "sky"
            }),
        );

        assert_eq!(config.theme_mode(), "light");
        assert_eq!(config.accent_color(), "emerald");
        assert_eq!(config.ui_style(), "glass");
        assert_eq!(
            config.semantic_colors()["preset"],
            "colorblind"
        );
    }

    #[test]
    fn test_orbit_config_dev_mode_directory_resolution() {
        let _lock = ENV_LOCK.lock().unwrap();
        // Save current env state
        let original_orbit_dev = std::env::var("ORBIT_DEV").ok();
        let original_config_dir = std::env::var_os("ORBIT_CONFIG_DIR");

        unsafe {
            std::env::remove_var("ORBIT_CONFIG_DIR");
        }

        // 1. Explicit Dev mode via ORBIT_DEV=1
        unsafe {
            std::env::set_var("ORBIT_DEV", "1");
        }
        assert!(OrbitConfig::is_dev_mode());
        assert_eq!(OrbitConfig::config_dir_name(), ".orbit-dev");
        let dev_dir = OrbitConfig::config_dir().expect("config_dir should be resolved");
        assert!(
            dev_dir.ends_with(".orbit-dev"),
            "dev config_dir should end with .orbit-dev, got {:?}",
            dev_dir
        );
        let dev_logs = OrbitConfig::logs_dir().expect("logs_dir should be resolved");
        assert!(
            dev_logs.ends_with(".orbit-dev\\logs") || dev_logs.ends_with(".orbit-dev/logs"),
            "dev logs_dir should end with .orbit-dev/logs, got {:?}",
            dev_logs
        );

        // 2. Explicit Prod mode via ORBIT_DEV=0
        unsafe {
            std::env::set_var("ORBIT_DEV", "0");
        }
        assert!(!OrbitConfig::is_dev_mode());
        assert_eq!(OrbitConfig::config_dir_name(), ".orbit");
        let prod_dir = OrbitConfig::config_dir().expect("config_dir should be resolved");
        assert!(
            prod_dir.ends_with(".orbit"),
            "prod config_dir should end with .orbit, got {:?}",
            prod_dir
        );

        // Restore env
        unsafe {
            match original_config_dir {
                Some(val) => std::env::set_var("ORBIT_CONFIG_DIR", val),
                None => std::env::remove_var("ORBIT_CONFIG_DIR"),
            }
            match original_orbit_dev {
                Some(val) => std::env::set_var("ORBIT_DEV", val),
                None => std::env::remove_var("ORBIT_DEV"),
            }
        }
    }

    #[test]
    fn test_orbit_config_dir_respects_custom_env_override() {
        let _lock = ENV_LOCK.lock().unwrap();
        let original_custom = std::env::var_os("ORBIT_CONFIG_DIR");
        let test_custom_path = PathBuf::from("/custom/test/orbit_config_path");

        unsafe {
            std::env::set_var("ORBIT_CONFIG_DIR", &test_custom_path);
        }

        let resolved = OrbitConfig::config_dir();
        assert_eq!(resolved, Some(test_custom_path));

        unsafe {
            match original_custom {
                Some(val) => std::env::set_var("ORBIT_CONFIG_DIR", val),
                None => std::env::remove_var("ORBIT_CONFIG_DIR"),
            }
        }
    }

    #[test]
    fn test_cluster_customizations_in_system_definitions() {
        let defs = Configuration::system_definitions();
        let cfg = defs
            .iter()
            .find(|c| c.key == "clusterCustomizations")
            .expect("clusterCustomizations must exist");
        assert_eq!(cfg.datatype, "string");
        assert_eq!(cfg.cardinality, "0..1");
    }

    #[test]
    fn test_cluster_customization_get_and_set() {
        use crate::kubernetes::models::ClusterCustomization;
        let mut config = OrbitConfig::default();
        assert!(config.cluster_customizations().is_empty());

        let customization = ClusterCustomization {
            alias: Some("Production EKS".to_string()),
            tags: vec!["prod".to_string(), "aws".to_string()],
            color: Some("#ef4444".to_string()),
            description: Some("Main payment cluster".to_string()),
        };

        config.set_cluster_customization("ctx-1", customization.clone());
        let map = config.cluster_customizations();
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("ctx-1"), Some(&customization));

        // Test clearing customization when empty
        config.set_cluster_customization("ctx-1", ClusterCustomization::default());
        assert!(config.cluster_customizations().is_empty());
    }

    #[test]
    fn test_auto_check_updates_in_system_definitions() {
        let defs = Configuration::system_definitions();
        let cfg = defs
            .iter()
            .find(|c| c.key == "autoCheckUpdates")
            .expect("autoCheckUpdates must exist in system definitions");
        assert!(cfg.enable, "autoCheckUpdates must be enabled");
        assert_eq!(cfg.datatype, "boolean");
        assert_eq!(cfg.default_value, Some(serde_json::json!(true)));
    }

    #[test]
    fn test_auto_check_updates_getter() {
        let mut config = OrbitConfig::default();
        assert!(config.auto_check_updates(), "Default auto_check_updates should be true");

        config.values.insert("autoCheckUpdates".to_string(), serde_json::json!(false));
        assert!(!config.auto_check_updates());

        config.values.insert("autoCheckUpdates".to_string(), serde_json::json!(true));
        assert!(config.auto_check_updates());
    }

    #[test]
    fn test_share_telemetry_absent_from_system_definitions() {
        let defs = Configuration::system_definitions();
        assert!(
            defs.iter().find(|c| c.key == "shareTelemetry").is_none(),
            "shareTelemetry should be removed from system definitions"
        );
    }

    #[test]
    fn test_notification_preferences_in_system_definitions() {
        let defs = Configuration::system_definitions();
        let cfg = defs
            .iter()
            .find(|c| c.key == "notificationPreferences")
            .expect("notificationPreferences must exist");
        assert!(cfg.enable);
        assert_eq!(cfg.datatype, "string");
        let default_val = cfg.default_value.as_ref().expect("default value must exist");
        assert_eq!(default_val["enableToasts"], true);
        assert_eq!(default_val["notifyOnSuccess"], true);
        assert_eq!(default_val["notifyOnError"], true);
        assert_eq!(default_val["notifyOnUpdates"], true);

        let mut config = OrbitConfig::default();
        let notif = config.notification_preferences();
        assert_eq!(notif["enableToasts"], true);

        config.values.insert(
            "notificationPreferences".to_string(),
            serde_json::json!({
                "enableToasts": false,
                "notifyOnSuccess": false,
                "notifyOnError": true,
                "notifyOnUpdates": false
            }),
        );
        let updated_notif = config.notification_preferences();
        assert_eq!(updated_notif["enableToasts"], false);
    }

    #[test]
    fn test_proxy_settings_in_system_definitions() {
        let defs = Configuration::system_definitions();
        let cfg = defs
            .iter()
            .find(|c| c.key == "proxySettings")
            .expect("proxySettings must exist");
        assert!(cfg.enable);
        assert_eq!(cfg.datatype, "string");
        let default_val = cfg.default_value.as_ref().expect("default value must exist");
        assert_eq!(default_val["enabled"], false);
        assert_eq!(default_val["httpProxy"], "");
        assert_eq!(default_val["httpsProxy"], "");
        assert_eq!(default_val["noProxy"], "");

        let mut config = OrbitConfig::default();
        let proxy = config.proxy_settings();
        assert_eq!(proxy["enabled"], false);

        config.values.insert(
            "proxySettings".to_string(),
            serde_json::json!({
                "enabled": true,
                "httpProxy": "http://proxy.corp.internal:8080",
                "httpsProxy": "http://proxy.corp.internal:8080",
                "noProxy": "localhost,127.0.0.1,.internal"
            }),
        );
        let updated_proxy = config.proxy_settings();
        assert_eq!(updated_proxy["enabled"], true);
        assert_eq!(updated_proxy["httpProxy"], "http://proxy.corp.internal:8080");
    }

    #[test]
    fn test_selected_namespaces_in_system_definitions_and_per_cluster_roundtrip() {
        let _guard = TestConfigDirGuard::new("selected_namespaces");
        let defs = Configuration::system_definitions();
        let cfg = defs
            .iter()
            .find(|c| c.key == "selectedNamespaces")
            .expect("selectedNamespaces must exist");
        assert!(cfg.enable);
        assert_eq!(cfg.datatype, "string");
        assert_eq!(cfg.cardinality, "0..1");
        assert_eq!(cfg.default_value, Some(serde_json::json!({})));

        let mut config = OrbitConfig::default();
        config
            .set_selected_namespaces_for_cluster(
                "prod-ctx",
                vec![
                    "payments".to_string(),
                    "default".to_string(),
                    "payments".to_string(),
                    "  ".to_string(),
                ],
            )
            .expect("save should succeed");

        let reloaded = OrbitConfig::load();
        assert_eq!(
            reloaded.selected_namespaces_for_cluster("prod-ctx"),
            vec!["payments".to_string(), "default".to_string()]
        );
        assert!(reloaded
            .selected_namespaces_for_cluster("dev-ctx")
            .is_empty());

        let mut cleared = reloaded;
        cleared
            .set_selected_namespaces_for_cluster("prod-ctx", Vec::new())
            .expect("clear should succeed");
        let reloaded_after_clear = OrbitConfig::load();
        assert!(reloaded_after_clear
            .selected_namespaces_for_cluster("prod-ctx")
            .is_empty());
        assert!(!reloaded_after_clear
            .values
            .contains_key("selectedNamespaces"));
    }

    #[test]
    fn test_workspace_tabs_in_system_definitions_and_per_cluster_roundtrip() {
        let _guard = TestConfigDirGuard::new("workspace_tabs");
        let defs = Configuration::system_definitions();
        let cfg = defs
            .iter()
            .find(|c| c.key == "workspaceTabs")
            .expect("workspaceTabs must exist");
        assert!(cfg.enable);
        assert_eq!(cfg.datatype, "string");
        assert_eq!(cfg.cardinality, "0..1");
        assert_eq!(cfg.default_value, Some(serde_json::json!({})));

        let mut config = OrbitConfig::default();
        config
            .set_workspace_tabs_for_cluster(
                "prod-ctx",
                vec![
                    "pods".to_string(),
                    "deployments".to_string(),
                    "pods".to_string(),
                    "  ".to_string(),
                ],
                Some("missing-tab".to_string()),
            )
            .expect("save should succeed");

        let reloaded = OrbitConfig::load();
        assert_eq!(
            reloaded.workspace_tabs_for_cluster("prod-ctx"),
            Some(PersistedClusterTabs {
                open_tabs: vec!["pods".to_string(), "deployments".to_string()],
                active_tab: Some("pods".to_string()),
            })
        );
        assert_eq!(reloaded.workspace_tabs_for_cluster("dev-ctx"), None);

        let mut emptied = reloaded;
        emptied
            .set_workspace_tabs_for_cluster("prod-ctx", Vec::new(), Some("pods".to_string()))
            .expect("save empty tabs should succeed");
        let reloaded_after_empty = OrbitConfig::load();
        assert_eq!(
            reloaded_after_empty.workspace_tabs_for_cluster("prod-ctx"),
            Some(PersistedClusterTabs {
                open_tabs: Vec::new(),
                active_tab: None,
            })
        );
    }

    #[test]
    fn test_appearance_and_experience_config_setters_and_persistence() {
        let _guard = TestConfigDirGuard::new("appearance_and_experience_setters");
        let mut config = OrbitConfig::default();

        config.update_theme_mode("  LIGHT ");
        config.update_ui_style("LiquidGlass");
        config.update_accent_color("EMERALD");
        config.update_enable_experience_features(true);

        assert_eq!(config.theme_mode(), "light");
        assert_eq!(config.ui_style(), "liquidglass");
        assert_eq!(config.accent_color(), "emerald");
        assert!(config.enable_experience_features());

        config.save().expect("save should succeed");
        let reloaded = OrbitConfig::load();
        assert_eq!(reloaded.theme_mode(), "light");
        assert_eq!(reloaded.ui_style(), "liquidglass");
        assert_eq!(reloaded.accent_color(), "emerald");
        assert!(reloaded.enable_experience_features());

        let mut reset = reloaded;
        reset.update_theme_mode("dark");
        reset.update_ui_style("flat");
        reset.update_accent_color("blue");
        reset.update_enable_experience_features(false);

        assert!(!reset.values.contains_key("themeMode"));
        assert!(!reset.values.contains_key("uiStyle"));
        assert!(!reset.values.contains_key("accentColor"));
        assert!(!reset.values.contains_key("enableExperienceFeatures"));

        // Invalid values fall back to canonical defaults and stay omitted
        reset.update_theme_mode("unknown-mode");
        reset.update_ui_style("unknown-style");
        reset.update_accent_color("unknown-accent");
        assert_eq!(reset.theme_mode(), "dark");
        assert_eq!(reset.ui_style(), "flat");
        assert_eq!(reset.accent_color(), "blue");
        assert!(!reset.values.contains_key("themeMode"));
        assert!(!reset.values.contains_key("uiStyle"));
        assert!(!reset.values.contains_key("accentColor"));

        // reset_appearance_preferences clears all appearance keys while keeping experience flag
        reset.update_theme_mode("system");
        reset.update_ui_style("paper");
        reset.update_accent_color("rose");
        reset.update_enable_experience_features(true);
        reset.reset_appearance_preferences();
        assert!(!reset.values.contains_key("themeMode"));
        assert!(!reset.values.contains_key("uiStyle"));
        assert!(!reset.values.contains_key("accentColor"));
        assert!(reset.enable_experience_features());
    }

    #[test]
    fn test_table_preferences_system_definition_exists() {
        let defs = Configuration::system_definitions();
        let cfg = defs
            .iter()
            .find(|c| c.key == "tablePreferences")
            .expect("tablePreferences must exist");
        assert!(cfg.enable);
        assert_eq!(cfg.datatype, "string");
        assert_eq!(cfg.cardinality, "0..1");
        assert_eq!(cfg.default_value, Some(serde_json::json!({})));
    }

    #[test]
    fn test_orbit_config_table_config_roundtrip_and_validation() {
        let _guard = TestConfigDirGuard::new("table_config_roundtrip");
        let mut config = OrbitConfig::default();

        config.update_table_config(
            "pods",
            PersistedTableConfig {
                page_size: Some(50),
                hidden_columns: vec![
                    "node".to_string(),
                    "restarts".to_string(),
                    "node".to_string(),
                    "  ".to_string(),
                ],
                sort_column: Some("name".to_string()),
                sort_direction: Some(PersistedSortDirection::Asc),
            },
        );
        config.save().expect("save should succeed");

        let reloaded = OrbitConfig::load();
        let prefs = reloaded.table_preferences();
        assert_eq!(
            prefs.tables.get("pods"),
            Some(&PersistedTableConfig {
                page_size: Some(50),
                hidden_columns: vec!["node".to_string(), "restarts".to_string()],
                sort_column: Some("name".to_string()),
                sort_direction: Some(PersistedSortDirection::Asc),
            })
        );

        // Invalid page_size (e.g. 13) is ignored/normalized to None
        let mut next = reloaded;
        next.update_table_config(
            "pods",
            PersistedTableConfig {
                page_size: Some(13),
                hidden_columns: vec![],
                sort_column: Some("age".to_string()),
                sort_direction: Some(PersistedSortDirection::Desc),
            },
        );
        assert_eq!(
            next.table_preferences().tables.get("pods"),
            Some(&PersistedTableConfig {
                page_size: None,
                hidden_columns: vec![],
                sort_column: Some("age".to_string()),
                sort_direction: Some(PersistedSortDirection::Desc),
            })
        );

        // Empty config prunes the table entry and removes tablePreferences if empty
        next.update_table_config("pods", PersistedTableConfig::default());
        assert!(!next.table_preferences().tables.contains_key("pods"));
        assert!(!next.values.contains_key("tablePreferences"));
    }

    #[test]
    fn test_orbit_config_cluster_table_filters_isolation_and_pruning() {
        let _guard = TestConfigDirGuard::new("cluster_table_filters");
        let mut config = OrbitConfig::default();

        let mut extra = HashMap::new();
        extra.insert("status".to_string(), "Running".to_string());
        extra.insert("empty".to_string(), "  ".to_string());

        config.update_cluster_table_filters(
            "prod-ctx",
            "pods",
            PersistedClusterTableFilters {
                search_query: "  nginx ".to_string(),
                extra_filters: extra,
            },
        );
        config.save().expect("save should succeed");

        let reloaded = OrbitConfig::load();
        let prefs = reloaded.table_preferences();
        let prod_pods = prefs
            .clusters
            .get("prod-ctx")
            .and_then(|t| t.get("pods"))
            .expect("prod-ctx pods filters must exist");
        assert_eq!(prod_pods.search_query, "nginx");
        assert_eq!(
            prod_pods.extra_filters.get("status").map(String::as_str),
            Some("Running")
        );
        assert!(!prod_pods.extra_filters.contains_key("empty"));
        assert!(!prefs.clusters.contains_key("dev-ctx"));

        // Clearing search + extra filters prunes the empty entry
        let mut cleared = reloaded;
        cleared.update_cluster_table_filters(
            "prod-ctx",
            "pods",
            PersistedClusterTableFilters::default(),
        );
        assert!(cleared.table_preferences().clusters.is_empty());
        assert!(!cleared.values.contains_key("tablePreferences"));
    }
}

