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
                enable: false, // Planned for future release
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
                key: "shareTelemetry".to_string(),
                name: "Share Telemetry".to_string(),
                description: "Anonymous crash reporting and usage telemetry to improve stability.".to_string(),
                datatype: "boolean".to_string(),
                default_value: Some(serde_json::json!(false)),
                value: None,
                is_confidential: false,
                cardinality: "1..1".to_string(),
                enable: false, // Disabled
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
                description: "Visual design aesthetic style (flat, glass, aero, liquidglass, ios).".to_string(),
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
                created_at: Some(created),
                last_updated_at: None,
            },
        ]
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orbit_config_default_and_getters() {
        let config = OrbitConfig::default();
        assert!(config.custom_kubeconfig_paths().is_empty());
        assert_eq!(config.max_log_files(), 10);
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
        // Save current env state
        let original_orbit_dev = std::env::var("ORBIT_DEV").ok();

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
            match original_orbit_dev {
                Some(val) => std::env::set_var("ORBIT_DEV", val),
                None => std::env::remove_var("ORBIT_DEV"),
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
}

