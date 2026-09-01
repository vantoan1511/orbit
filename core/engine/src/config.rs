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
                description: "Automatically start Orbit when logging into the system.".to_string(),
                datatype: "boolean".to_string(),
                default_value: Some(serde_json::json!(false)),
                value: None,
                is_confidential: false,
                cardinality: "1..1".to_string(),
                enable: false, // Planned for future release
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
    /// Returns the path to the Orbit configuration directory (`~/.orbit`).
    pub fn config_dir() -> Option<PathBuf> {
        let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
        let mut path = PathBuf::from(home);
        path.push(".orbit");
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
                tracing::trace!(key = %incoming.key, "Skipped config update: value is None");
                continue;
            };

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
    }
}
